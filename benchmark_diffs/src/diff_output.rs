use std::ops::Range;

use hyper_diff::tree::tree_path::CompressedTreePath;
use hyperast::{
    position::{Position, compute_position},
    types::{self, LabelStore, Labeled, NodeStore, WithSerialization},
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct F<T> {
    pub times: Vec<usize>,
    pub matches: Vec<Match<T>>,
    pub actions: Option<Vec<Act<T>>>,
}

#[derive(Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Match<T> {
    pub src: T,
    pub dest: T,
}

#[derive(Deserialize, PartialEq, Eq, Hash, Debug)]
pub struct Act<T> {
    pub action: Kind,
    pub tree: T,
    pub parent: Option<T>,
    pub at: Option<usize>,
    pub label: Option<String>,
}

#[derive(Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Tree {
    pub r#type: String,
    pub label: Option<String>,
    pub file: String,
    pub start: usize,
    pub end: usize,
}

#[derive(Deserialize, PartialEq, Eq, Hash, Debug, Clone)]
pub struct Path(pub Vec<u32>);

#[derive(Deserialize, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    #[serde(rename = "update-node")]
    Upd,
    #[serde(rename = "move-tree")]
    Move,
    #[serde(rename = "insert-node")]
    Ins,
    #[serde(rename = "delete-node")]
    Del,
}

impl Tree {
    pub fn from_pos<HAST>(stores: HAST, (pos, x): (Position, HAST::IdN)) -> Self
    where
        HAST: types::HyperAST,
        for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::Tree,
    {
        let Range { start, end } = pos.range();
        let file = pos.file().to_string_lossy().to_string();
        let r = stores.node_store().resolve(&x);
        let t = stores.resolve_type(&x);
        Tree {
            r#type: t.to_string(),
            label: r
                .try_get_label()
                .map(|x| stores.label_store().resolve(&x).to_string())
                .filter(|x| !x.is_empty()),
            file,
            start,
            end,
        }
    }
}

impl<'a, HAST> From<(HAST, HAST::IdN, &CompressedTreePath<HAST::Idx>)> for Tree
where
    HAST: types::HyperAST + Copy,
    HAST::IdN: types::NodeId<IdN = HAST::IdN>,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::Tree + WithSerialization,
{
    fn from((stores, ori, p): (HAST, HAST::IdN, &CompressedTreePath<HAST::Idx>)) -> Self {
        (stores, compute_position(ori, &mut p.iter(), stores)).into()
    }
}

impl<HAST> From<(HAST, (Position, HAST::IdN))> for Tree
where
    HAST: types::HyperAST,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::Tree,
{
    fn from((stores, (pos, x)): (HAST, (Position, HAST::IdN))) -> Self {
        let Range { start, end } = pos.range();
        let file = pos.file().to_string_lossy().to_string();
        let r = stores.node_store().resolve(&x);
        let t = stores.resolve_type(&x);
        Tree {
            r#type: t.to_string(),
            label: r
                .try_get_label()
                .map(|x| stores.label_store().resolve(&x).to_string())
                .filter(|x| !x.is_empty()),
            file,
            start,
            end,
        }
    }
}
