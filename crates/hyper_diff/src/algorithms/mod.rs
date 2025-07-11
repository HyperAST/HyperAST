use crate::actions::action_vec::ActionsVec;
use crate::actions::script_generator2::SimpleAction;
use crate::decompressed_tree_store::ShallowDecompressedTreeStore;
use crate::matchers::{Mapper, mapping_store::VecStore};

pub mod change_distiller;
pub mod change_distiller_lazy;
pub mod change_distiller_partial_lazy;
pub mod gumtree;
pub mod gumtree_hybrid;
pub mod gumtree_hybrid_lazy;
pub mod gumtree_hybrid_partial_lazy;
pub mod gumtree_lazy;
pub mod gumtree_partial_lazy;
pub mod gumtree_simple;
pub mod gumtree_simple_lazy;
pub mod gumtree_stable;
pub mod gumtree_stable_lazy;
pub mod xy;

#[derive(Debug, Clone)]
pub struct MappingDurations<const N: usize, D>(pub [D; N]);

#[derive(Debug, Clone)]
pub struct PreparedMappingDurations<const N: usize, D> {
    pub mappings: MappingDurations<N, D>,
    pub preparation: [D; N],
}

#[derive(Debug, Clone)]
pub struct MappingMemoryUsages<const N: usize> {
    pub memory: [usize; N],
}

impl<const N: usize, D: std::iter::Sum + std::ops::Add<Output = D> + Copy> ComputeTime
    for PreparedMappingDurations<N, D>
{
    type T = D;
    fn time(&self) -> D {
        self.preparation.iter().copied().sum::<D>() + self.mappings.0.iter().copied().sum::<D>()
    }
}

impl<const N: usize, D: Copy> From<PreparedMappingDurations<N, D>> for MappingDurations<N, D> {
    fn from(value: PreparedMappingDurations<N, D>) -> Self {
        value.mappings
    }
}

impl<const N: usize, D> From<[D; N]> for MappingDurations<N, D> {
    fn from(value: [D; N]) -> Self {
        Self(value)
    }
}

pub struct DiffResult<A, M, MD, D> {
    pub mapping_durations: MD,
    pub mapping_memory_usages: MappingMemoryUsages<2>, // todo: use templates
    pub mapper: M,
    pub actions: Option<ActionsVec<A>>,
    pub prepare_gen_t: D,
    pub gen_t: D,
}

#[allow(type_alias_bounds)]
type DiffRes<HAST: HyperASTShared> = DiffResult<
    SimpleAction<HAST::Label, CompressedTreePath<HAST::Idx>, HAST::IdN>,
    Mapper<HAST, CDS<HAST>, CDS<HAST>, VecStore<u32>>,
    PreparedMappingDurations<2, std::time::Duration>,
    std::time::Duration,
>;

#[derive(Debug)]
pub struct ResultsSummary<MD, D> {
    pub mapping_durations: MD,
    pub mapping_memory_usages: MappingMemoryUsages<2>,
    pub mappings: usize,
    pub actions: Option<usize>,
    pub prepare_gen_t: D,
    pub gen_t: D,
}

impl<A, MD: Clone, HAST, DS, DD, D: Clone>
    DiffResult<A, Mapper<HAST, DS, DD, VecStore<u32>>, MD, D>
{
    pub fn summarize(&self) -> ResultsSummary<MD, D> {
        use crate::actions::Actions;
        use crate::matchers::mapping_store::MappingStore;
        ResultsSummary {
            mapping_durations: self.mapping_durations.clone(),
            mappings: self.mapper.mapping.mappings.len(),
            actions: self.actions.as_ref().map(|x| x.len()),
            prepare_gen_t: self.prepare_gen_t.clone(),
            gen_t: self.gen_t.clone(),
            mapping_memory_usages: self.mapping_memory_usages.clone(),
        }
    }
}

pub trait ComputeTime {
    type T: std::ops::Add<Output = Self::T> + Copy;
    fn time(&self) -> Self::T;
}

impl<'a, MD, D> ResultsSummary<MD, D> {
    pub fn compare_results(&self, other: &Self) -> bool {
        self.mappings == other.mappings && self.actions == other.actions
    }
}

impl<'a, MD: ComputeTime> ComputeTime for ResultsSummary<MD, MD::T> {
    type T = MD::T;
    fn time(&self) -> Self::T {
        self.gen_t + self.prepare_gen_t + self.mapping_durations.time()
    }
}

impl<'a, A, M, MD: ComputeTime> ComputeTime for DiffResult<A, M, MD, MD::T> {
    type T = MD::T;
    fn time(&self) -> Self::T {
        self.gen_t + self.prepare_gen_t + self.mapping_durations.time()
    }
}

// WIP
impl<HAST, Dsrc, Ddst, M, MD> std::fmt::Display
    for DiffResult<
        crate::actions::script_generator2::SimpleAction<
            HAST::Label,
            crate::tree::tree_path::CompressedTreePath<HAST::Idx>,
            HAST::IdN,
        >,
        Mapper<HAST, Dsrc, Ddst, M>,
        MD,
        MD::T,
    >
where
    Dsrc: ShallowDecompressedTreeStore<HAST, u32>,
    HAST: types::HyperAST + Copy,
    MD: ComputeTime,
    MD::T: std::fmt::Debug,
    for<'t> <HAST as types::AstLending<'t>>::RT: types::WithSerialization,
    for<'t> <HAST as types::AstLending<'t>>::RT: types::WithStats,
    HAST::IdN: Copy + types::NodeId<IdN = HAST::IdN> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "structural diff {:?}s", self.time())?;
        let ori = self
            .mapper
            .src_arena
            .original(&self.mapper.src_arena.root());
        let Some(actions) = &self.actions else {
            return Ok(());
        };
        crate::actions::action_vec::actions_vec_f(f, actions, self.mapper.hyperast, ori)
    }
}

fn get_allocated_memory() -> usize {
    use jemalloc_ctl::{epoch, stats};
    epoch::advance().unwrap();
    stats::allocated::read().unwrap()
}

use crate::decompressed_tree_store;
use crate::matchers;
use crate::tree::tree_path::CompressedTreePath;

#[allow(type_alias_bounds)]
type DS<HAST: types::HyperASTShared> = matchers::Decompressible<
    HAST,
    decompressed_tree_store::lazy_post_order::LazyPostOrder<HAST::IdN, u32>,
>;

#[allow(type_alias_bounds)]
type CDS<HAST: types::HyperASTShared> =
    matchers::Decompressible<HAST, decompressed_tree_store::CompletePostOrder<HAST::IdN, u32>>;

fn check_oneshot_decompressed_against_lazy<HAST: types::HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
    mapper: &Mapper<HAST, CDS<HAST>, CDS<HAST>, VecStore<u32>>,
) where
    HAST::IdN: Clone + std::fmt::Debug + Eq,
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: std::fmt::Debug + Clone + Copy + Eq,
    for<'t> <HAST as types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let mapper = mapper.src_arena.decomp.deref();
    let mapper = mapper.deref();
    log::trace!(
        "naive.ids:\t{:?}",
        &mapper.iter().take(20).collect::<Vec<_>>()
    );
    log::trace!(
        "naive:\t{:?}",
        &mapper.llds.iter().take(20).collect::<Vec<_>>()
    );
    use matchers::Decompressible;
    use types::HyperASTShared;
    #[allow(type_alias_bounds)]
    type DS<HAST: HyperASTShared> = Decompressible<
        HAST,
        crate::decompressed_tree_store::lazy_post_order::LazyPostOrder<HAST::IdN, u32>,
    >;
    let _mapper: (HAST, (DS<HAST>, DS<HAST>)) = hyperast.decompress_pair(src, dst);
    let mut _mapper_owned: Mapper<_, DS<HAST>, DS<HAST>, VecStore<u32>> = _mapper.into();
    let _mapper = Mapper {
        hyperast,
        mapping: crate::matchers::Mapping {
            mappings: _mapper_owned.mapping.mappings,
            src_arena: _mapper_owned.mapping.src_arena,
            dst_arena: _mapper_owned.mapping.dst_arena,
        },
    };
    use decompressed_tree_store::CompletePostOrder;
    let _mapper = _mapper.map(
        |src_arena| {
            Decompressible::<HAST, CompletePostOrder<HAST::IdN, _>>::from(
                src_arena.map(|x| x.complete(hyperast)),
            )
        },
        |dst_arena| {
            Decompressible::<HAST, CompletePostOrder<HAST::IdN, _>>::from(
                dst_arena.map(|x| x.complete(hyperast)),
            )
        },
    );
    use std::ops::Deref;
    let _mapper = _mapper.src_arena.decomp.deref();
    let _mapper = _mapper.deref();
    log::trace!(
        "lazy:\t{:?}",
        &_mapper.llds.iter().take(20).collect::<Vec<_>>()
    );
    log::trace!(
        "lazy.ids:\t{:?}",
        &_mapper.iter().take(20).collect::<Vec<_>>()
    );
    assert_eq!(_mapper.llds, mapper.llds);
}

macro_rules! tr {
    ($($val:ident),*) => {
        $(
            log::trace!("{}={:?}", stringify!($val), $val);
        )*
    };
}
use hyperast::types::{self, HyperASTShared};
pub(self) use tr;
