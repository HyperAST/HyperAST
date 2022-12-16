use hyper_ast::store::defaults::{LabelIdentifier};
use hyper_gumtree::{
    actions::{action_vec::ActionsVec, script_generator2::SimpleAction},
    matchers::mapping_store::VecStore,
};

pub mod gumtree;

// type IdD = u32;
// type DS<'store> = CompletePostOrder<HashedNodeRef<'store>, IdD>;

pub struct DiffResult<IdN, IdL, IdX, IdD, DS1, DS2, const M: usize> {
    pub mapping_durations: [f64; M],
    pub src_arena: DS1,
    pub dst_arena: DS2,
    pub mappings: VecStore<IdD>,
    pub actions: ActionsVec<SimpleAction<IdL, IdX, IdN>>,
    pub gen_t: f64,
}
