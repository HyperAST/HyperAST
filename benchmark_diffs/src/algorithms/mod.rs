use hyper_ast::store::defaults::{NodeIdentifier, LabelIdentifier};
use hyper_gumtree::{decompressed_tree_store::CompletePostOrder, matchers::mapping_store::VecStore, actions::{action_vec::ActionsVec, script_generator2::SimpleAction}};

pub mod gumtree;


type IdD = u32;
type DS = CompletePostOrder<NodeIdentifier, IdD>;


pub struct DiffResult<const M:usize> {
    pub mapping_durations: [f64;M],
    pub src_arena: DS,
    pub dst_arena: DS,
    pub mappings: VecStore<IdD>,
    pub actions: ActionsVec<SimpleAction<LabelIdentifier, u16, NodeIdentifier>>,
    pub gen_t: f64,
}