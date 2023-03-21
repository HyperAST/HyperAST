use std::hash::Hash;

pub type NodeId = u64;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FetchedHyperAST {
    pub(super) label_list: Vec<String>,
    pub(super) type_sys: TypeSys,
    pub(super) labeled: ViewLabeled,
    pub(super) children: ViewChildren,
    pub(super) both: ViewBoth,
    pub(super) typed: ViewTyped,
}

impl Hash for FetchedHyperAST {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.label_list.len().hash(state);
        self.type_sys.0.len().hash(state);
        self.labeled.ids.len().hash(state);
        self.children.ids.len().hash(state);
        self.both.ids.len().hash(state);
        self.typed.ids.len().hash(state);
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ViewLabeled {
    pub(super) ids: Vec<NodeId>,
    pub(super) kinds: Vec<u16>,
    pub(super) labels: Vec<u32>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ViewTyped {
    pub(super) ids: Vec<NodeId>,
    pub(super) kinds: Vec<u16>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ViewChildren {
    pub(super) ids: Vec<NodeId>,
    pub(super) kinds: Vec<u16>,
    pub(super) cs_ofs: Vec<u32>,
    pub(super) cs_lens: Vec<u32>,
    pub(super) children: Vec<NodeId>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ViewBoth {
    pub(super) ids: Vec<NodeId>,
    pub(super) kinds: Vec<u16>,
    pub(super) labels: Vec<u32>,
    pub(super) cs_ofs: Vec<u32>,
    pub(super) cs_lens: Vec<u32>,
    pub(super) children: Vec<NodeId>,
}

#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub(crate) struct TypeSys(pub(super) Vec<String>);
