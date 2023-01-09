use std::ops::Range;

use hyper_ast::{
    position::{compute_position, Position},
    types::{self, Tree as _, Typed},
};
use hyper_gumtree::tree::tree_path::{CompressedTreePath, TreePath};
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

impl<'a, IdN: Clone, NS: types::NodeStore<IdN>, LS: types::LabelStore<str>>
    From<(
        (&'a NS, &'a LS),
        IdN,
        &CompressedTreePath<<NS::R<'a> as types::WithChildren>::ChildIdx>,
    )> for Tree
where
    NS::R<'a>:
        types::Tree<TreeId = IdN, Type = types::Type, Label = LS::I> + types::WithSerialization,
{
    fn from(
        ((node_store, label_store), ori, p): (
            (&'a NS, &'a LS),
            IdN,
            &CompressedTreePath<<NS::R<'a> as types::WithChildren>::ChildIdx>,
        ),
    ) -> Self {
        (
            (node_store, label_store),
            compute_position(ori, &mut p.iter(), node_store, label_store),
        )
            .into()
    }
}

impl<'a, IdN, NS: 'a + types::NodeStore<IdN>, LS: types::LabelStore<str>>
    From<((&'a NS, &'a LS), (Position, IdN))> for Tree
where
    NS::R<'a>: types::Tree<TreeId = IdN, Type = types::Type, Label = LS::I>,
{
    fn from(((node_store, label_store), (pos, x)): ((&'a NS, &'a LS), (Position, IdN))) -> Self {
        let Range { start, end } = pos.range();
        let file = pos.file().to_string_lossy().to_string();
        let r = node_store.resolve(&x);
        Tree {
            r#type: r.get_type().to_string(),
            label: r
                .try_get_label()
                .map(|x| label_store.resolve(x).to_string())
                .filter(|x| !x.is_empty()),
            file,
            start,
            end,
        }
    }
}

// impl
//     From<(
//         &NodeIdentifier,
//         &SimpleStores,
//         &SimpleAction<LabelIdentifier, u16, NodeIdentifier>,
//     )> for Act
// {
//     fn from(
//         (ori, stores, a): (
//             &NodeIdentifier,
//             &SimpleStores,
//             &SimpleAction<LabelIdentifier, u16, NodeIdentifier>,
//         ),
//     ) -> Self {
//         let f = |p: &SimpleAction<LabelIdentifier, u16, NodeIdentifier>| {
//             // TODO make whole thing more specific to a path in a tree
//             let mut curr = None;
//             struct ItLast<T, It: Iterator<Item = T>> {
//                 tmp: Option<T>,
//                 it: It,
//             }

//             impl<T, It: Iterator<Item = T>> ItLast<T, It> {
//                 fn new(it: It) -> Self {
//                     Self { it, tmp: None }
//                 }
//                 fn end(self) -> Option<T> {
//                     self.tmp
//                 }
//             }

//             impl<T, It: Iterator<Item = T>> Iterator for ItLast<T, It> {
//                 type Item = T;

//                 fn next(&mut self) -> Option<Self::Item> {
//                     todo!()
//                 }
//             }

//             struct A<'a, T, It: Iterator<Item = T>> {
//                 curr: &'a mut Option<T>,
//                 it: It,
//             }
//             impl<'a, T: Clone, It: Iterator<Item = T>> Iterator for A<'a, T, It> {
//                 type Item = T;

//                 fn next(&mut self) -> Option<Self::Item> {
//                     if self.curr.is_none() {
//                         let x = self.it.next()?;
//                         self.curr.replace(x.clone());
//                     }
//                     let x = self.it.next()?;
//                     self.curr.replace(x.clone())
//                 }
//             }
//             let mut it = A {
//                 curr: &mut curr,
//                 it: a.path.ori.iter(),
//             };
//             let (pos, x) = compute_position(*ori, &mut it, stores);
//             let Range { start, end } = pos.range().into();
//             let file = pos.file().to_string_lossy().to_string();
//             let r = stores.node_store.resolve(x);
//             let p = Tree {
//                 r#type: r.get_type().to_string(),
//                 label: r
//                     .try_get_label()
//                     .map(|x| stores.label_store.resolve(x).to_string()),
//                 file,
//                 start,
//                 end,
//             };
//             (
//                 it.it
//                     .chain(vec![curr.unwrap()].into_iter())
//                     .collect::<Vec<_>>(),
//                 p,
//             )
//         };
//         match &a.action {
//             script_generator2::Act::Delete {} => Act {
//                 action: Kind::Del,
//                 tree: (ori, stores, &a.path.ori).into(),
//                 parent: None,
//                 at: None,
//                 label: None,
//             },
//             script_generator2::Act::Update { new } => Act {
//                 action: Kind::Upd,
//                 tree: (ori, stores, &a.path.ori).into(),
//                 parent: None,
//                 at: None,
//                 label: Some(stores.label_store.resolve(&new).to_string()),
//             },
//             script_generator2::Act::Insert { sub } => {
//                 let (at, parent) = f(a);
//                 Act {
//                     action: Kind::Ins,
//                     tree: (ori, stores, &a.path.ori).into(),
//                     parent: Some(parent),
//                     at: Some(at[0] as usize),
//                     label: None,
//                 }
//             }
//             // println!(
//             //     "Ins {:?} {}",
//             //     {
//             //         let node = stores.node_store.resolve(*sub);
//             //         node.get_type()
//             //     },
//             //     format_action_pos(ori, stores, a)
//             // ),
//             script_generator2::Act::Move { from } => {
//                 let (at, parent) = f(a);
//                 Act {
//                     action: Kind::Ins,
//                     tree: (ori, stores, &a.path.ori).into(),
//                     parent: Some(parent),
//                     at: Some(at[0] as usize),
//                     label: None,
//                 }
//             }
//             // println!(
//             //     "Mov {:?} {:?} {}",
//             //     {
//             //         let mut node = stores.node_store.resolve(ori);
//             //         for x in from.ori.iter() {
//             //             let e = node.get_child(&x);
//             //             node = stores.node_store.resolve(e);
//             //         }
//             //         node.get_type()
//             //     },
//             //     make_position(ori, &mut from.ori.iter(), stores),
//             //     format_action_pos(ori, stores, a)
//             // ),
//             script_generator2::Act::MovUpd { from, new } => todo!(),
//         }
//     }
// }
