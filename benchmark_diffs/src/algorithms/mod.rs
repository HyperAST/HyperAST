use hyper_ast::store::defaults::{LabelIdentifier, NodeIdentifier};
use hyper_gumtree::{
    actions::{action_vec::ActionsVec, script_generator2::SimpleAction},
    matchers::mapping_store::VecStore,
};

pub mod gumtree;

// type IdD = u32;
// type DS<'store> = CompletePostOrder<HashedNodeRef<'store>, IdD>;

pub struct DiffResult<IdD, DS1, DS2, const M: usize> {
    pub mapping_durations: [f64; M],
    pub src_arena: DS1,
    pub dst_arena: DS2,
    pub mappings: VecStore<IdD>,
    pub actions: ActionsVec<SimpleAction<LabelIdentifier, u16, NodeIdentifier>>,
    pub gen_t: f64,
}
