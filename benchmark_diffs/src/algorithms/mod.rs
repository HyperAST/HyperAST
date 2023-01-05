use hyper_gumtree::{
    actions::{action_vec::ActionsVec, script_generator2::SimpleAction},
    matchers::mapping_store::VecStore,
};

pub mod gumtree;

pub struct DiffResult<IdN, IdL, P, IdD, DS1, DS2, const M: usize> {
    pub mapping_durations: [f64; M],
    pub src_arena: DS1,
    pub dst_arena: DS2,
    pub mappings: VecStore<IdD>,
    pub actions: ActionsVec<SimpleAction<IdL, P, IdN>>,
    pub gen_t: f64,
}
