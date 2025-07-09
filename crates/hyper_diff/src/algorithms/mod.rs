use crate::{
    actions::action_vec::ActionsVec,
    decompressed_tree_store::ShallowDecompressedTreeStore,
    matchers::{Mapper, mapping_store::VecStore},
};
use jemalloc_ctl::{epoch, stats};

pub mod gumtree;
pub mod gumtree_hybrid;
pub mod gumtree_hybrid_lazy;
pub mod gumtree_hybrid_partial_lazy;
pub mod gumtree_lazy;
pub mod gumtree_partial_lazy;

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
    HAST: hyperast::types::HyperAST + Copy,
    MD: ComputeTime,
    MD::T: std::fmt::Debug,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: hyperast::types::WithSerialization,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: hyperast::types::WithStats,
    HAST::IdN: Copy + hyperast::types::NodeId<IdN = HAST::IdN> + std::fmt::Debug,
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
    epoch::advance().unwrap();
    stats::allocated::read().unwrap()
}

macro_rules! tr {
    ($($val:ident),*) => {
        $(
            log::trace!("{}={:?}", stringify!($val), $val);
        )*
    };
}
pub(self) use tr;
