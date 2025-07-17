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

type DefaultMetric = <LatMem as RuntimeMeasurement>::M;
type DefaultMetricSetup = Phased<Prepared<DefaultMetric>>;

#[derive(Clone, Debug)]
pub struct Phased<Current, Prev = ()> {
    pub current: Current,
    prev: Prev,
}

impl<Current, Prev> Phased<Current, Prev> {
    pub fn cdr(&self) -> &Prev {
        &self.prev
    }
}

impl<P1, P2, P3> Phased<P3, Phased<P2, Phased<P1>>> {
    pub fn phase1(&self) -> &P1 {
        &self.prev.prev.current
    }
    pub fn phase2(&self) -> &P2 {
        &self.prev.current
    }
    pub fn phase3(&self) -> &P3 {
        &self.current
    }
}

impl<Current, Prev> std::ops::Deref for Phased<Current, Prev> {
    type Target = Current;

    fn deref(&self) -> &Self::Target {
        &self.current
    }
}

impl<D: RuntimeMetric, D2> Phased<Prepared<D, D2>> {
    fn prepare() -> Phased<Prepared<D, ()>> {
        Phased {
            current: Prepared::<D>::prepare(),
            prev: (),
        }
    }
}
impl<P1, P2> Phased<P1, P2> {
    fn map<P3>(self, f: impl Fn(P1) -> P3) -> Phased<P3, P2> {
        Phased {
            current: f(self.current),
            prev: self.prev,
        }
    }
}

impl<D: RuntimeMetric, P> Phased<Prepared<D, ()>, P> {
    fn start(self) -> Phased<Prepared<D::M, D>, P> {
        self.map(|prepared| prepared.start())
    }
}

impl<D: RuntimeMetric, P> Phased<Prepared<D::M, D>, P> {
    fn stop(self) -> Phased<Prepared<D::M>, P> {
        self.map(|prepared| prepared.stop())
    }
    fn next_p<T>(self, f: impl Fn() -> T) -> Phased<T, Phased<Prepared<D::M>, P>> {
        Phased {
            current: f(),
            prev: Phased {
                current: self.current.stop(),
                prev: self.prev,
            },
        }
    }
    fn stop_then_prepare(self) -> Phased<Prepared<D, ()>, Phased<Prepared<D::M>, P>> {
        self.next_p(|| Prepared::<D>::prepare())
    }
    fn stop_then_skip_prepare(self) -> Phased<Prepared<D::M, D>, Phased<Prepared<D::M>, P>> {
        self.next_p(|| Prepared::<D>::nothing())
    }
}

impl<P1: RuntimeMeasurement, P2: RuntimeMeasurement> Phased<P1, P2> {
    pub fn sum<T: 'static + Clone + std::ops::Add<Output = T>>(&self) -> Option<T> {
        let (a, b) = self.current.sum::<T>().zip(self.prev.sum::<T>())?;
        Some(a.clone() + b.clone())
    }
}
impl<P1: RuntimeMeasurement, P2: RuntimeMeasurement> RuntimeMeasurement for Phased<P1, P2> {
    type M = Prepared<P1::M, P2::M>;
    fn display(&self) -> impl std::fmt::Display {
        format!("{} + {}", self.current.display(), self.prev.display())
    }
    fn sum<T: 'static + Clone + std::ops::Add<Output = T>>(&self) -> Option<T> {
        let (a, b) = self.current.sum::<T>().zip(self.prev.sum::<T>())?;
        Some(a.clone() + b.clone())
    }
}

#[derive(Clone, Debug)]
pub struct Prepared<D, D2 = D> {
    pub prep: D,
    pub mapping: D2,
}

impl<D: RuntimeMetric, D2> Prepared<D, D2> {
    fn prepare() -> Prepared<D, ()> {
        Prepared {
            prep: D::start(),
            mapping: (),
        }
    }
    fn nothing() -> Prepared<D::M, D> {
        Prepared {
            prep: D::nothing(),
            mapping: D::start(),
        }
    }
}

impl<D: RuntimeMetric> Prepared<D, ()> {
    fn start(self) -> Prepared<D::M, D> {
        Prepared {
            prep: self.prep.stop(),
            mapping: D::start(),
        }
    }
}

impl<D: RuntimeMetric> Prepared<D::M, D> {
    fn stop(self) -> Prepared<D::M, D::M> {
        Prepared {
            prep: self.prep,
            mapping: self.mapping.stop(),
        }
    }
}

impl<D1: RuntimeMeasurement, D2: RuntimeMeasurement> RuntimeMeasurement for Prepared<D1, D2> {
    type M = Prepared<D1::M, D2::M>;
    fn display(&self) -> impl std::fmt::Display {
        format!("{} + {}", self.prep.display(), self.mapping.display())
    }
    fn sum<D: 'static + Clone + std::ops::Add<Output = D>>(&self) -> Option<D> {
        let (a, b) = self.prep.sum::<D>().zip(self.mapping.sum::<D>())?;
        Some(a.clone() + b.clone())
    }
}

pub type LatMem = (std::time::Duration, AllocatedMemory);
pub type PreparedPhased3<D> = Phased<Prepared<D>, Phased<Prepared<D>, Phased<Prepared<D>>>>;

pub trait RuntimeMetric {
    fn start() -> Self;
    type M: RuntimeMeasurement<M = Self>;
    fn stop(self) -> Self::M;
    fn nothing() -> Self::M;
}

pub trait RuntimeMeasurement {
    type M;
    fn display(&self) -> impl std::fmt::Display;
    fn sum<D: 'static + Clone + std::ops::Add<Output = D>>(&self) -> Option<D>;
}

impl<M1: RuntimeMetric, M2: RuntimeMetric> RuntimeMetric for (M1, M2) {
    type M = (M1::M, M2::M);
    fn start() -> Self {
        (M1::start(), M2::start())
    }
    fn stop(self) -> Self::M {
        (self.0.stop(), self.1.stop())
    }
    fn nothing() -> Self::M {
        (M1::nothing(), M2::nothing())
    }
}

impl<M1: RuntimeMeasurement, M2: RuntimeMeasurement> RuntimeMeasurement for (M1, M2) {
    type M = (M1::M, M2::M);
    fn display(&self) -> impl std::fmt::Display {
        struct DisplayTuple<'a, T>(&'a T);
        impl<M1: RuntimeMeasurement, M2: RuntimeMeasurement> std::fmt::Display
            for DisplayTuple<'_, (M1, M2)>
        {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                self.0.0.display().fmt(f)?;
                self.0.1.display().fmt(f)?;
                Ok(())
            }
        }
        DisplayTuple(self)
    }

    fn sum<D: 'static + Clone + std::ops::Add<Output = D>>(&self) -> Option<D> {
        let (a, b) = self.0.sum::<D>().zip(self.0.sum::<D>())?;
        Some(a.clone() + b.clone())
    }
}

impl RuntimeMeasurement for () {
    type M = ();
    fn display(&self) -> impl std::fmt::Display {
        struct DisplayEmpty();
        impl std::fmt::Display for DisplayEmpty {
            fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                Ok(())
            }
        }
        DisplayEmpty()
    }

    fn sum<D: 'static + Clone + std::ops::Add<Output = D>>(&self) -> Option<D> {
        None
    }
}

impl RuntimeMetric for std::time::Instant {
    type M = std::time::Duration;
    fn start() -> Self {
        std::time::Instant::now()
    }
    fn stop(self) -> Self::M {
        self.elapsed()
    }
    fn nothing() -> Self::M {
        std::time::Duration::ZERO
    }
}

impl RuntimeMeasurement for std::time::Duration {
    type M = std::time::Instant;
    fn display(&self) -> impl std::fmt::Display {
        struct Disp(std::time::Duration);
        impl std::fmt::Display for Disp {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let secs = self.0.as_secs_f64();
                write!(f, "{:.3}s", secs)
            }
        }
        Disp(*self)
    }

    fn sum<D: 'static + Clone + std::ops::Add<Output = D>>(&self) -> Option<D> {
        use std::any::Any;
        let x: &dyn Any = self;
        <dyn Any>::downcast_ref::<D>(x).cloned()
    }
}

#[derive(Clone, Copy)]
pub struct AllocatedMemory(pub isize);

impl RuntimeMetric for AllocatedMemory {
    fn start() -> Self {
        use jemalloc_ctl::{epoch, stats};
        epoch::advance().unwrap();
        Self(stats::allocated::read().unwrap() as isize)
    }

    type M = AllocatedMemory;

    fn stop(self) -> Self::M {
        use jemalloc_ctl::{epoch, stats};
        epoch::advance().unwrap();
        let mem = stats::allocated::read().unwrap() as isize;
        Self(mem - self.0)
    }
    fn nothing() -> Self::M {
        Self(0)
    }
}

impl RuntimeMeasurement for AllocatedMemory {
    type M = AllocatedMemory;
    fn display(&self) -> impl std::fmt::Display {
        self
    }

    fn sum<D: 'static + Clone + std::ops::Add<Output = D>>(&self) -> Option<D> {
        use std::any::Any;
        let x: &dyn Any = self;
        <dyn Any>::downcast_ref::<D>(x).cloned()
    }
}

impl std::fmt::Display for AllocatedMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} bytes", self.0)
    }
}

impl std::fmt::Debug for AllocatedMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AllocatedMemory")
            .field("bytes", &self.0)
            .finish()
    }
}

impl std::ops::Add for AllocatedMemory {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}

// #[derive(Debug, Clone)]
// struct MappingDurations<const N: usize, D>(pub [D; N]);

// #[derive(Debug, Clone)]
// pub struct PreparedMappingDurations<const N: usize, D> {
//     pub mappings: MappingDurations<N, D>,
//     pub preparation: [D; N],
// }

// #[derive(Debug, Clone)]
// pub struct MappingMemoryUsages<const N: usize> {
//     pub memory: [usize; N],
// }

// impl<const N: usize, D: std::iter::Sum + std::ops::Add<Output = D> + Copy> ComputeTime
//     for PreparedMappingDurations<N, D>
// {
//     type T = D;
//     fn time(&self) -> D {
//         self.preparation.iter().copied().sum::<D>() + self.mappings.0.iter().copied().sum::<D>()
//     }
// }

// impl<const N: usize, D: Copy> From<PreparedMappingDurations<N, D>> for MappingDurations<N, D> {
//     fn from(value: PreparedMappingDurations<N, D>) -> Self {
//         value.mappings
//     }
// }

// impl<const N: usize, D> From<[D; N]> for MappingDurations<N, D> {
//     fn from(value: [D; N]) -> Self {
//         Self(value)
//     }
// }

pub struct DiffResult<A, M, MD> {
    pub mapper: M,
    pub actions: Option<ActionsVec<A>>,

    // measures during preparation, mapping and ED generation, such as time and memory usage
    pub exec_data: MD,
    // pub mapping_durations: MD,
    // pub mapping_memory_usages: MappingMemoryUsages<2>, // todo: use templates
    // pub prepare_gen_t: D,
    // pub gen_t: D,
}

#[allow(type_alias_bounds)]
type DiffRes<HAST: HyperASTShared> = DiffResult<
    SimpleAction<HAST::Label, CompressedTreePath<HAST::Idx>, HAST::IdN>,
    Mapper<HAST, CDS<HAST>, CDS<HAST>, VecStore<u32>>,
    PreparedPhased3<LatMem>,
>;

#[derive(Debug)]
pub struct ResultsSummary<MD> {
    pub mappings: usize,
    pub actions: Option<usize>,
    pub exec_data: MD,
    // pub mapping_durations: MD,
    // pub mapping_memory_usages: MappingMemoryUsages<2>,
    // pub prepare_gen_t: D,
    // pub gen_t: D,
}

impl<A, MD: Clone, HAST, DS, DD> DiffResult<A, Mapper<HAST, DS, DD, VecStore<u32>>, MD> {
    pub fn summarize(&self) -> ResultsSummary<MD> {
        use crate::actions::Actions;
        use crate::matchers::mapping_store::MappingStore;
        ResultsSummary {
            // mapping_durations: self.mapping_durations.clone(),
            mappings: self.mapper.mapping.mappings.len(),
            actions: self.actions.as_ref().map(|x| x.len()),
            exec_data: self.exec_data.clone(),
            // prepare_gen_t: self.prepare_gen_t.clone(),
            // gen_t: self.gen_t.clone(),
            // mapping_memory_usages: self.mapping_memory_usages.clone(),
        }
    }
}

impl<'a, MD> ResultsSummary<MD> {
    pub fn compare_results(&self, other: &Self) -> bool {
        self.mappings == other.mappings && self.actions == other.actions
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
    >
where
    Dsrc: ShallowDecompressedTreeStore<HAST, u32>,
    HAST: types::HyperAST + Copy,
    // MD: ComputeTime,
    // MD::T: std::fmt::Debug,
    for<'t> <HAST as types::AstLending<'t>>::RT: types::WithSerialization,
    for<'t> <HAST as types::AstLending<'t>>::RT: types::WithStats,
    HAST::IdN: Copy + types::NodeId<IdN = HAST::IdN> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // writeln!(f, "structural diff {:?}s", self.time())?;
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
