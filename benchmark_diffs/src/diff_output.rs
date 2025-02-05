use std::ops::Range;

use hyperast::{
    position::{compute_position, Position},
    types::{self, LabelStore, Labeled, NodeStore, WithSerialization},
};
use hyper_diff::tree::tree_path::CompressedTreePath;
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

impl<'a, HAST> From<(&'a HAST, HAST::IdN, &CompressedTreePath<HAST::Idx>)> for Tree
where
    HAST: types::HyperAST<'a>,
    HAST::T: types::Tree + WithSerialization,
{
    fn from((stores, ori, p): (&'a HAST, HAST::IdN, &CompressedTreePath<HAST::Idx>)) -> Self {
        (stores, compute_position(ori, &mut p.iter(), stores)).into()
    }
}

impl<'a, HAST> From<(&'a HAST, (Position, HAST::IdN))> for Tree
where
    HAST: types::HyperAST<'a>,
    HAST::T: types::Tree,
{
    fn from((stores, (pos, x)): (&'a HAST, (Position, HAST::IdN))) -> Self {
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
