use crate::{
    actions::action_vec::ActionsVec,
    decompressed_tree_store::ShallowDecompressedTreeStore,
    matchers::{Mapper, mapping_store::VecStore},
};

pub mod gumtree;
pub mod gumtree_lazy;
pub mod gumtree_partial_lazy;

#[derive(Debug, Clone)]
pub struct MappingDurations<const N: usize>(pub [f64; N]);

#[derive(Debug, Clone)]
pub struct PreparedMappingDurations<const N: usize> {
    pub mappings: MappingDurations<N>,
    pub preparation: [f64; N],
}

impl<const N: usize> ComputeTime for PreparedMappingDurations<N> {
    fn time(&self) -> f64 {
        self.preparation.iter().sum::<f64>() + self.mappings.0.iter().sum::<f64>()
    }
}

impl<const N: usize> From<PreparedMappingDurations<N>> for MappingDurations<N> {
    fn from(value: PreparedMappingDurations<N>) -> Self {
        value.mappings
    }
}

impl<const N: usize> From<[f64; N]> for MappingDurations<N> {
    fn from(value: [f64; N]) -> Self {
        Self(value)
    }
}

pub struct DiffResult<A, M, MD> {
    pub mapping_durations: MD,
    pub mapper: M,
    pub actions: Option<ActionsVec<A>>,
    pub prepare_gen_t: f64,
    pub gen_t: f64,
}

#[derive(Debug)]
pub struct ResultsSummary<MD> {
    pub mapping_durations: MD,
    pub mappings: usize,
    pub actions: Option<usize>,
    pub prepare_gen_t: f64,
    pub gen_t: f64,
}

impl<A, MD: Clone, HAST, DS, DD> DiffResult<A, Mapper<HAST, DS, DD, VecStore<u32>>, MD> {
    pub fn summarize(&self) -> ResultsSummary<MD> {
        use crate::actions::Actions;
        use crate::matchers::mapping_store::MappingStore;
        ResultsSummary {
            mapping_durations: self.mapping_durations.clone(),
            mappings: self.mapper.mapping.mappings.len(),
            actions: self.actions.as_ref().map(|x| x.len()),
            prepare_gen_t: self.prepare_gen_t,
            gen_t: self.gen_t,
        }
    }
}
pub trait ComputeTime {
    fn time(&self) -> f64;
}

impl<'a, MD> ResultsSummary<MD> {
    pub fn compare_results(&self, other: &Self) -> bool {
        self.mappings == other.mappings && self.actions == other.actions
    }
}

impl<'a, MD: ComputeTime> ComputeTime for ResultsSummary<MD> {
    fn time(&self) -> f64 {
        self.gen_t + self.prepare_gen_t + self.mapping_durations.time()
    }
}

impl<'a, A, M, MD: ComputeTime> ComputeTime for DiffResult<A, M, MD> {
    fn time(&self) -> f64 {
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
    >
where
    Dsrc: ShallowDecompressedTreeStore<HAST, u32>,
    HAST: hyperast::types::HyperAST + Copy,
    MD: ComputeTime,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: hyperast::types::WithSerialization,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: hyperast::types::WithStats,
    HAST::IdN: Copy + hyperast::types::NodeId<IdN = HAST::IdN> + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "structural diff {}s", self.time())?;
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

// #[macro_use]
macro_rules! tr {
    ($($val:ident),*) => {
        $(
            log::trace!("{}={}", stringify!($val), $val);
        )*
    };
}
pub(self) use tr;
