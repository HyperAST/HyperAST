use hyper_gumtree::{
    actions::{action_vec::ActionsVec, script_generator2::SimpleAction},
    matchers::mapping_store::VecStore,
};

pub mod gumtree;
pub mod gumtree_lazy;
pub mod gumtree_partial_lazy;

trait MD {
    // const N: usize;
    // fn mappings(&self) -> [f64;Self::N];
}

#[derive(Debug)]
pub struct MappingDurations<const N: usize> (pub [f64; N]);

#[derive(Debug)]
pub struct PreparedMappingDurations<const N: usize> {
    pub mappings: MappingDurations<N>,
    pub preparation: [f64; N],
}

impl<const N: usize> From<PreparedMappingDurations<N>> for MappingDurations<N> {
    fn from(value: PreparedMappingDurations<N>) -> Self {
        value.mappings
    }
}

impl<const N: usize> From<[f64;N]> for MappingDurations<N> {
    fn from(value: [f64;N]) -> Self {
        Self(value)
    }
}

pub struct DiffResult<IdN, IdL, P, M, MD> {
    pub mapping_durations: MD,
    pub mapper: M,
    pub actions: Option<ActionsVec<SimpleAction<IdL, P, IdN>>>,
    pub prepare_gen_t: f64,
    pub gen_t: f64,
}
