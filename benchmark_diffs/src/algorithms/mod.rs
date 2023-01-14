use hyper_gumtree::{
    actions::{action_vec::ActionsVec},
    matchers::{mapping_store::VecStore, Mapper},
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

impl<'a, A, MD: Clone, HAST, DS, DD> DiffResult<A, Mapper<'a, HAST, DS, DD, VecStore<u32>>, MD> {
    pub fn summarize(&self) -> ResultsSummary<MD> {
        use hyper_gumtree::actions::Actions;
        use hyper_gumtree::matchers::mapping_store::MappingStore;
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
