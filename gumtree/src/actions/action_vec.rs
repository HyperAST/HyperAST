use std::fmt::Debug;

use num_traits::ToPrimitive;

use hyper_ast::{
    position::compute_range,
    store::{
        defaults::{LabelIdentifier, NodeIdentifier},
        SimpleStores,
    },
    types::{
        Children, IterableChildren, LabelStore, NodeStore, NodeStoreExt, Tree, Typed, WithChildren,
    },
};

use crate::tree::tree_path::TreePath;

use super::{
    script_generator2::{Act, SimpleAction},
    Actions,
};

pub struct ActionsVec<A>(pub Vec<A>);

impl<A: Debug> Debug for ActionsVec<A> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("ActionsVec").field(&self.0).finish()
    }
}
impl<A> Default for ActionsVec<A> {
    fn default() -> Self {
        Self(Default::default())
    }
}

pub fn actions_vec_f<P: TreePath<Item = u16>>(
    v: &ActionsVec<SimpleAction<LabelIdentifier, P, NodeIdentifier>>,
    stores: &SimpleStores,
    ori: NodeIdentifier,
) {
    v.iter().for_each(|a| print_action(ori, stores, a));
}

fn format_action_pos<P: TreePath<Item = u16>>(
    ori: NodeIdentifier,
    stores: &SimpleStores,
    a: &SimpleAction<LabelIdentifier, P, NodeIdentifier>,
) -> String {
    // TODO make whole thing more specific to a path in a tree
    let mut end = None;
    // struct ItLast<T, It: Iterator<Item = T>> {
    //     tmp: Option<T>,
    //     it: It,
    // }

    // impl<T, It: Iterator<Item = T>> ItLast<T, It> {
    //     fn new(it: It) -> Self {
    //         Self { it, tmp: None }
    //     }
    //     fn end(self) -> Option<T> {
    //         self.tmp
    //     }
    // }

    // impl<T, It: Iterator<Item = T>> Iterator for ItLast<T, It> {
    //     type Item = T;

    //     fn next(&mut self) -> Option<Self::Item> {
    //         todo!()
    //     }
    // }

    struct A<'a, T, It: Iterator<Item = T>> {
        curr: &'a mut Option<T>,
        it: It,
    }
    impl<'a, T: Clone, It: Iterator<Item = T>> Iterator for A<'a, T, It> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            if self.curr.is_none() {
                let x = self.it.next()?;
                self.curr.replace(x.clone());
            }
            let x = self.it.next()?;
            self.curr.replace(x.clone())
        }
    }
    let mut it = A {
        curr: &mut end,
        it: a.path.ori.iter(),
    };
    let p = compute_range(ori, &mut it, stores);
    format!(
        "{:?} at {:?}",
        p,
        it.it
            .chain(vec![end.unwrap()].into_iter())
            .collect::<Vec<_>>()
    )
}

fn print_action<P: TreePath<Item = u16>>(
    ori: NodeIdentifier,
    stores: &SimpleStores,
    a: &SimpleAction<LabelIdentifier, P, NodeIdentifier>,
) {
    match &a.action {
        Act::Delete {} => println!(
            "Del {:?}",
            compute_range(ori, &mut a.path.ori.iter(), stores)
        ),
        Act::Update { new } => println!(
            "Upd {:?} {:?}",
            stores.label_store.resolve(new),
            compute_range(ori, &mut a.path.ori.iter(), stores)
        ),
        Act::Insert { sub } => println!(
            "Ins {:?} {}",
            {
                let node = stores.node_store.resolve(*sub);
                node.get_type()
            },
            format_action_pos(ori, stores, a)
        ),
        Act::Move { from } => println!(
            "Mov {:?} {:?} {}",
            {
                let mut node = stores.node_store.resolve(ori);
                for x in from.ori.iter() {
                    let e = node.child(&x).unwrap();
                    node = stores.node_store.resolve(e);
                }
                node.get_type()
            },
            compute_range(ori, &mut from.ori.iter(), stores),
            format_action_pos(ori, stores, a)
        ),
        Act::MovUpd { from, new } => println!(
            "MovUpd {:?} {:?} {:?} {}",
            {
                let mut node = stores.node_store.resolve(ori);
                for x in from.ori.iter() {
                    let e = node.child(&x).unwrap();
                    node = stores.node_store.resolve(e);
                }
                node.get_type()
            },
            stores.label_store.resolve(new),
            compute_range(ori, &mut from.ori.iter(), stores),
            format_action_pos(ori, stores, a)
        ),
    }
}

impl<A> Actions for ActionsVec<A> {
    fn len(&self) -> usize {
        self.0.len()
    }
}
impl<A> ActionsVec<A> {
    pub fn iter(&self) -> impl Iterator<Item = &A> + '_ {
        self.0.iter()
    }
}

pub trait TestActions<A> {
    fn has_actions(&self, items: &[A]) -> bool;
}

impl<A: Eq> TestActions<A> for ActionsVec<A> {
    fn has_actions(&self, items: &[A]) -> bool {
        items.iter().all(|x| self.0.contains(x))
    }
}

impl<L: Debug, P: TreePath, I: Debug> ActionsVec<SimpleAction<L, P, I>> {
    pub(crate) fn push(&mut self, action: SimpleAction<L, P, I>) {
        self.0.push(action)
    }
    pub(crate) fn get(&self, i: usize) -> Option<&SimpleAction<L, P, I>> {
        self.0.get(i)
    }

    pub(crate) fn new() -> Self {
        Self(Default::default())
    }
}

// // pub trait BuildableTree<T: Tree> {
// //     fn build(t: T::Type, l: T::Label, cs: Vec<T::TreeId>) -> T;
// // }

/// WARN for now it is very inneficient because it completly apply actions every times,
/// most likely it would need a temporary structure.
/// Also actions are applied in order, thus there is a single way of applying actions.
/// It might not have enough info to it flexibly, action_tree could definetly be more flexible.
// pub fn apply_actions<S: for<'b> NodeStoreMut<'b, <T as Stored>::TreeId, &'b T>>(
pub fn apply_actions<T, S, P>(
    actions: ActionsVec<SimpleAction<T::Label, P, T::TreeId>>,
    root: &mut Vec<T::TreeId>,
    node_store: &mut S,
) where
    P: TreePath<Item = T::ChildIdx> + Debug,
    T: hyper_ast::types::Tree,
    T::Type: Debug + Copy,
    T::Label: Debug + Copy,
    T::TreeId: Debug + Copy,
    T::ChildIdx: Debug + Copy,
    S: NodeStoreExt<T> + NodeStore<T::TreeId>, //NodeStoreExt<'a, T, R>,
    for<'d> S::R<'d>: hyper_ast::types::Tree<
        TreeId = T::TreeId,
        Type = T::Type,
        Label = T::Label,
        ChildIdx = T::ChildIdx,
    >,
{
    for a in actions.iter() {
        // *log::debug!(
        //     "mid tree:\n{:?}",
        //     DisplayTree::new(&label_store, &node_store, *root.last().unwrap())
        // );
        apply_action(a, root, node_store);
    }
}

pub fn apply_action<T, S, P>(
    a: &SimpleAction<T::Label, P, T::TreeId>,
    root: &'_ mut Vec<T::TreeId>,
    s: &'_ mut S,
) where
    P: TreePath<Item = T::ChildIdx> + Debug,
    T: hyper_ast::types::Tree,
    T::Type: Debug + Copy,
    T::Label: Debug + Copy,
    T::TreeId: Debug + Copy,
    T::ChildIdx: Debug + Copy,
    S: NodeStoreExt<T> + NodeStore<T::TreeId>, //NodeStoreExt<'a, T, R>,
    for<'d> S::R<'d>: hyper_ast::types::Tree<
        TreeId = T::TreeId,
        Type = T::Type,
        Label = T::Label,
        ChildIdx = T::ChildIdx,
    >,
{
    let fun_name = |s: &mut S, x: &T::TreeId| -> (T::Type, Option<T::Label>) {
        let node = s.resolve(x);
        let t = node.get_type().to_owned();
        let l = node.try_get_label().cloned();
        (t, l)
    };
    let a = a;
    let roots: &mut Vec<_> = root;
    log::trace!("{:?}", a);
    let SimpleAction { path, action } = a;

    let from = match action {
        Act::Move { from } => Some(from),
        Act::MovUpd { from, .. } => Some(from),
        _ => None,
    };

    let sub = if let Some(from) = from {
        // apply remove
        log::trace!("sub path {:?}", from.mid.iter().collect::<Vec<_>>());
        let mut path = from.mid.iter();
        let fp = path.next().unwrap().to_usize().unwrap();
        // dbg!(&fp);
        let r = &mut roots[fp];
        let mut x = *r;
        let mut parents: Vec<(T::TreeId, T::ChildIdx, Vec<T::TreeId>)> = vec![];
        while let Some(p) = path.next() {
            // dbg!(&p);
            let node = s.resolve(&x);
            let cs = node.children().unwrap();
            parents.push((x, p, cs.iter_children().cloned().collect()));
            let i = p;
            // dbg!(cs.len());
            if i < cs.child_count() {
                x = cs[i].clone();
            } else {
                assert!(path.next().is_none());
                break;
            }
        }
        log::trace!("parents {:?}", parents);
        let (node, sub) = if let Some((x, i, cs)) = parents.pop() {
            let mut children = Vec::with_capacity(cs.len() - 1);
            children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
            children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
            let (t, l) = fun_name(s, &x);
            let node = s.build_then_insert(x, t, l, children);
            (node, cs[i.to_usize().unwrap()].clone())
        } else {
            // let mut children = Vec::with_capacity(cs.len() - 1);
            // children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
            // children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
            // let node = s.resolve(&x);
            // let node = Self::build(node.get_type(), node.get_label().clone(), children);
            // s.get_or_insert(node)
            (r.clone(), r.clone())
        };
        let mut node: T::TreeId = node;
        for (x, i, cs) in parents.into_iter().rev() {
            let mut children = Vec::with_capacity(cs.len() - 1);
            children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
            children.push(node.clone());
            children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
            let (t, l) = fun_name(s, &x);
            node = s.build_then_insert(x, t, l, children);
            // let n: T = BuildableTree::build(n.get_type().to_owned(), n.get_label().clone(), children);
            // node = s.get_or_insert(n);
        }
        *r = node;
        Some(sub)
    } else {
        None
    };

    let mut parents: Vec<(T::TreeId, T::ChildIdx, Vec<T::TreeId>)> = vec![];
    log::trace!("{:?}", path.mid.iter().collect::<Vec<_>>());
    // dbg!(path.mid.iter().collect::<Vec<_>>());
    let mut path = path.mid.iter();
    let fp = path.next().unwrap().to_usize().unwrap();
    // dbg!(&fp);
    let r = if roots.len() > fp {
        &mut roots[fp]
    } else if roots.len() == fp {
        roots.push(roots[fp - 1].clone());
        &mut roots[fp]
    } else {
        panic!()
    };
    let mut x: T::TreeId = *r;
    while let Some(p) = path.next() {
        // dbg!(&p);
        let node = s.resolve(&x);
        let cs = node.children();
        let i = p;
        // TODO use pattern match
        if cs.is_some() && i < cs.unwrap().child_count() {
            let tmp = cs.unwrap()[i].clone();
            parents.push((x, p, cs.unwrap().iter_children().cloned().collect()));
            x = tmp;
        } else {
            if cs.is_some() && !cs.unwrap().is_empty() {
                parents.push((x, p, cs.unwrap().iter_children().cloned().collect()));
                log::error!("{:?} > {:?}", i, cs.unwrap().child_count());
            } else {
                parents.push((x, p, Default::default()));
                log::error!("{:?} > {:?}", i, 0);
            }
            assert_eq!(path.next(), None);
            break;
        }
    }

    let node = match action {
        Act::Delete {} => {
            let (x, i, cs) = parents.pop().unwrap();
            let mut children = Vec::with_capacity(cs.len() - 1);
            children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
            children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
            let (t, l) = fun_name(s, &x);
            s.build_then_insert(x, t, l, children)
        }
        Act::Insert { sub } => {
            if let Some((x, i, cs)) = parents.pop() {
                let mut children = Vec::with_capacity(cs.len());
                children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
                let sub = {
                    let (t, l) = fun_name(s, sub);
                    s.build_then_insert(*sub, t, l, vec![])
                };
                children.push(sub);
                if i.to_usize().unwrap() < cs.len() {
                    children.extend_from_slice(&cs[i.to_usize().unwrap()..]);
                }
                let (t, l) = fun_name(s, &x);
                s.build_then_insert(x, t, l, children)
            } else {
                let sub = {
                    let (t, l) = fun_name(s, sub);
                    s.build_then_insert(*sub, t, l, vec![])
                };
                // *r = sub.clone();
                sub
            }
        }
        Act::Update { new } => {
            if let Some((x, i, cs)) = parents.pop() {
                let mut children = Vec::with_capacity(cs.len());
                children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
                let sub = {
                    let x = cs[i.to_usize().unwrap()].clone();
                    // let node = s.resolve(&x);
                    // s.build_then_insert(
                    //     node.get_type(),
                    //     new.clone(),
                    //     node.get_children().to_vec(),
                    // )

                    let (t, cs) = {
                        let node = s.resolve(&x);
                        let t = node.get_type().to_owned();
                        let cs = node.children();
                        let cs = cs.map(|cs| cs.iter_children().cloned().collect());
                        (t, cs)
                    };
                    s.build_then_insert(x, t, Some(new.clone()), cs.unwrap_or_default())
                };
                children.push(sub);
                children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
                let (t, l) = fun_name(s, &x);
                s.build_then_insert(x, t, l, children)
            } else {
                let (t, cs) = {
                    let node = s.resolve(&x);
                    let t = node.get_type().to_owned();
                    let cs = node.children();
                    let cs = cs.map(|cs| cs.iter_children().cloned().collect());
                    (t, cs)
                };
                s.build_then_insert(x, t, Some(new.clone()), cs.unwrap_or_default())
            }
        }

        Act::Move { .. } => {
            // apply insert
            let (x, i, cs) = parents.pop().unwrap();
            let mut children = Vec::with_capacity(cs.len());
            children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
            let sub = {
                // let node = s.resolve(&sub.unwrap());
                // let node = BuildableTree::build(node.get_type(), node.get_label().clone(), vec![]);
                // s.get_or_insert(node)
                sub.unwrap()
            };
            children.push(sub);
            if i.to_usize().unwrap() < cs.len() {
                children.extend_from_slice(&cs[i.to_usize().unwrap()..]);
            }
            // dbg!(children.len());
            let (t, l) = fun_name(s, &x);
            s.build_then_insert(x, t, l, children)
        }
        Act::MovUpd { new, .. } => {
            // apply insert
            let (x, i, cs) = parents.pop().unwrap();
            let mut children = Vec::with_capacity(cs.len());
            children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
            let sub = {
                // let node = s.resolve(&sub.unwrap());
                // let node = BuildableTree::build(node.get_type(), node.get_label().clone(), vec![]);
                // s.get_or_insert(node)
                sub.unwrap()
            };
            children.push(sub);
            if i.to_usize().unwrap() < cs.len() {
                children.extend_from_slice(&cs[i.to_usize().unwrap()..]);
            }
            let t = {
                let node = s.resolve(&x);
                let t = node.get_type().to_owned();
                t
            };
            s.build_then_insert(x, t, Some(new.clone()), children)
        }
    };
    let mut node = node;
    for (x, i, cs) in parents.into_iter().rev() {
        let mut children = Vec::with_capacity(cs.len() - 1);
        children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
        children.push(node.clone());
        children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
        let (t, l) = fun_name(s, &x);
        node = s.build_then_insert(x, t, l, children);
    }
    *r = node;
}

// pub trait ActionApplier<T>
// where
//     T: hyper_ast::types::Tree,
//     T::Type: Debug + Copy,
//     T::Label: Debug + Copy,
//     T::TreeId: Debug + Copy,
//     T::ChildIdx: Debug + Copy,
// {
//     type S<'d>: NodeStoreExt2<T> + NodeStore2<T::TreeId, R<'d> = Self::R<'d>>
//     where
//         Self: 'd,
//         <Self as ActionApplier<T>>::S<'d>: 'd;
//     // where
//     //     for<'d> S::R<'d>: Self::R<'d>;
//     type R<'d>: 'd
//         + hyper_ast::types::Tree<
//             TreeId = T::TreeId,
//             Type = T::Type,
//             Label = T::Label,
//             ChildIdx = T::ChildIdx,
//         >
//     where
//         Self: 'd;
//     // for<'d> S::R<'d>: hyper_ast::types::Tree<
//     //     TreeId = T::TreeId,
//     //     Type = T::Type,
//     //     Label = T::Label,
//     //     ChildIdx = T::ChildIdx,
//     // >,

//     fn store(&mut self) -> &mut Self::S<'_>;

//     fn apply_action(
//         &mut self,
//         a: &SimpleAction<T::Label, T::ChildIdx, T::TreeId>,
//         root: &mut Vec<T::TreeId>,
//     ) {
//         let fun_name = |s: &mut Self::S<'_>, x: &T::TreeId| -> (T::Type, Option<T::Label>) {
//             let node = s.resolve(x);
//             let t = node.get_type().to_owned();
//             let l = node.try_get_label().cloned();
//             (t, l)
//         };
//         let a = a;
//         let roots: &mut Vec<_> = root;
//         log::trace!("{:?}", a);
//         let SimpleAction { path, action } = a;

//         let from = match action {
//             Act::Move { from } => Some(from),
//             Act::MovUpd { from, .. } => Some(from),
//             _ => None,
//         };
//         let s = self.store();
//         let sub = if let Some(from) = from {
//             // apply remove
//             log::trace!("sub path {:?}", from.mid.iter().collect::<Vec<_>>());
//             let mut path = from.mid.iter();
//             let r = &mut roots[path.next().unwrap().to_usize().unwrap()];
//             let mut x = *r;
//             let mut parents: Vec<(T::TreeId, T::ChildIdx, Vec<T::TreeId>)> = vec![];
//             while let Some(p) = path.next() {
//                 let node = s.resolve(&x);
//                 let cs = node.get_children().to_vec();
//                 parents.push((x, p, cs.iter().cloned().collect()));
//                 let i = p.to_usize().unwrap();
//                 if i < cs.len() {
//                     x = cs[i].clone();
//                 } else {
//                     assert!(path.next().is_none());
//                     break;
//                 }
//             }
//             log::trace!("parents {:?}", parents);
//             let (node, sub) = if let Some((x, i, cs)) = parents.pop() {
//                 let mut children = Vec::with_capacity(cs.len() - 1);
//                 children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                 children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                 let (t, l) = fun_name(s, &x);
//                 let node = s.build_then_insert(t, l, children);
//                 (node, cs[i.to_usize().unwrap()].clone())
//             } else {
//                 // let mut children = Vec::with_capacity(cs.len() - 1);
//                 // children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                 // children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                 // let node = s.resolve(&x);
//                 // let node = Self::build(node.get_type(), node.get_label().clone(), children);
//                 // s.get_or_insert(node)
//                 (r.clone(), r.clone())
//             };
//             let mut node: T::TreeId = node;
//             for (x, i, cs) in parents.into_iter().rev() {
//                 let mut children = Vec::with_capacity(cs.len() - 1);
//                 children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                 children.push(node.clone());
//                 children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                 let (t, l) = fun_name(s, &x);
//                 node = s.build_then_insert(t, l, children);
//                 // let n: T = BuildableTree::build(n.get_type().to_owned(), n.get_label().clone(), children);
//                 // node = s.get_or_insert(n);
//             }
//             *r = node;
//             Some(sub)
//         } else {
//             None
//         };

//         let mut parents: Vec<(T::TreeId, T::ChildIdx, Vec<T::TreeId>)> = vec![];
//         log::trace!("{:?}", path.mid.iter().collect::<Vec<_>>());
//         let mut path = path.mid.iter();
//         let fp = path.next().unwrap().to_usize().unwrap();
//         let r = if roots.len() > fp {
//             &mut roots[fp]
//         } else if roots.len() == fp {
//             roots.push(roots[fp - 1].clone());
//             &mut roots[fp]
//         } else {
//             panic!()
//         };
//         let mut x: T::TreeId = *r;
//         while let Some(p) = path.next() {
//             let node = s.resolve(&x);
//             let cs = node.get_children().to_vec();
//             parents.push((x, p, cs.clone()));
//             let i = p.to_usize().unwrap();
//             if i < cs.len() {
//                 x = cs[i].clone();
//             } else {
//                 log::error!("{:?} > {:?}", i, cs.len());
//                 assert_eq!(path.next(), None);
//                 break;
//             }
//         }

//         let node = match action {
//             Act::Delete {} => {
//                 let (x, i, cs) = parents.pop().unwrap();
//                 let mut children = Vec::with_capacity(cs.len() - 1);
//                 children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                 children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                 let (t, l) = fun_name(s, &x);
//                 s.build_then_insert(t, l, children)
//             }
//             Act::Insert { sub } => {
//                 if let Some((x, i, cs)) = parents.pop() {
//                     let mut children = Vec::with_capacity(cs.len());
//                     children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                     let sub = {
//                         let (t, l) = fun_name(s, sub);
//                         s.build_then_insert(t, l, vec![])
//                     };
//                     children.push(sub);
//                     if i.to_usize().unwrap() < cs.len() {
//                         children.extend_from_slice(&cs[i.to_usize().unwrap()..]);
//                     }
//                     let (t, l) = fun_name(s, &x);
//                     s.build_then_insert(t, l, children)
//                 } else {
//                     let sub = {
//                         let (t, l) = fun_name(s, sub);
//                         s.build_then_insert(t, l, vec![])
//                     };
//                     // *r = sub.clone();
//                     sub
//                 }
//             }
//             Act::Update { new } => {
//                 if let Some((x, i, cs)) = parents.pop() {
//                     let mut children = Vec::with_capacity(cs.len());
//                     children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                     let sub = {
//                         let x = cs[i.to_usize().unwrap()].clone();
//                         // let node = s.resolve(&x);
//                         // s.build_then_insert(
//                         //     node.get_type(),
//                         //     new.clone(),
//                         //     node.get_children().to_vec(),
//                         // )

//                         let (t, cs) = {
//                             let node = s.resolve(&x);
//                             let t = node.get_type().to_owned();
//                             let cs = node.get_children().to_vec();
//                             (t, cs)
//                         };
//                         s.build_then_insert(t, Some(new.clone()), cs)
//                     };
//                     children.push(sub);
//                     children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                     let (t, l) = fun_name(s, &x);
//                     s.build_then_insert(t, l, children)
//                 } else {
//                     let (t, cs) = {
//                         let node = s.resolve(&x);
//                         let t = node.get_type().to_owned();
//                         let cs = node.get_children().to_vec();
//                         (t, cs)
//                     };
//                     let mut children = Vec::with_capacity(cs.len());
//                     children.extend_from_slice(&cs[..]);
//                     s.build_then_insert(t, Some(new.clone()), children)
//                 }
//             }

//             Act::Move { .. } => {
//                 // apply insert
//                 let (x, i, cs) = parents.pop().unwrap();
//                 let mut children = Vec::with_capacity(cs.len());
//                 children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                 let sub = {
//                     // let node = s.resolve(&sub.unwrap());
//                     // let node = BuildableTree::build(node.get_type(), node.get_label().clone(), vec![]);
//                     // s.get_or_insert(node)
//                     sub.unwrap()
//                 };
//                 children.push(sub);
//                 if i.to_usize().unwrap() < cs.len() {
//                     children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                 }
//                 let (t, l) = fun_name(s, &x);
//                 s.build_then_insert(t, l, children)
//             }
//             Act::MovUpd { new, .. } => {
//                 // apply insert
//                 let (x, i, cs) = parents.pop().unwrap();
//                 let mut children = Vec::with_capacity(cs.len());
//                 children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                 let sub = {
//                     // let node = s.resolve(&sub.unwrap());
//                     // let node = BuildableTree::build(node.get_type(), node.get_label().clone(), vec![]);
//                     // s.get_or_insert(node)
//                     sub.unwrap()
//                 };
//                 children.push(sub);
//                 if i.to_usize().unwrap() < cs.len() {
//                     children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                 }
//                 let t = {
//                     let node = s.resolve(&x);
//                     let t = node.get_type().to_owned();
//                     t
//                 };
//                 s.build_then_insert(t, Some(new.clone()), children)
//             }
//         };
//         let mut node = node;
//         for (x, i, cs) in parents.into_iter().rev() {
//             let mut children = Vec::with_capacity(cs.len() - 1);
//             children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//             children.push(node.clone());
//             children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//             let (t, l) = fun_name(s, &x);
//             node = s.build_then_insert(t, l, children);
//         }
//         *r = node;
//     }
// }

// pub fn apply_actions<'b,'d, T, S, It>(
//     actions: It,
//     r: T::TreeId,
//     s: &'b mut S,
// ) -> <T as Stored>::TreeId
// where
//     T: 'b + hyper_ast::types::Tree + Clone,
//     T::Type: Debug + Copy + Default,
//     T::Label: Debug + Copy + Default,
//     T::TreeId: Debug + Copy + Default,
//     T::ChildIdx: Debug + Copy + Default,
//     S: 'd+NodeStoreExt2<T>+NodeStore2< T>, //NodeStoreExt<'a, T, R>,
//     S::R<'d>: hyper_ast::types::Tree<
//             TreeId = T::TreeId,
//             Type = T::Type,
//             Label = T::Label,
//             ChildIdx = T::ChildIdx,
//         > + Clone,
//     It: Iterator<Item = &'b SimpleAction<T::Label, T::ChildIdx, T::TreeId>>,
// {
//     let mut roots = vec![r.clone()];
//     for a in actions {
//         apply_action(a, &mut roots, s)
//     }
//     r
// }

// pub fn apply_action<'d,T, S>(
//     a: &SimpleAction<T::Label, T::ChildIdx, T::TreeId>,
//     roots: &mut Vec<T::TreeId>,
//     s: &mut S,
// ) where
//     T: hyper_ast::types::Tree + Clone,
//     T::Type: Debug + Copy + Default,
//     T::Label: Debug + Copy + Default,
//     T::TreeId: Debug + Copy + Default,
//     T::ChildIdx: Debug + Copy + Default,
//     S:'d+ NodeStoreExt2<T>+NodeStore2< T>, //NodeStoreExt<'a, T, R>,
//     S::R<'d>: hyper_ast::types::Tree<
//             TreeId = T::TreeId,
//             Type = T::Type,
//             Label = T::Label,
//             ChildIdx = T::ChildIdx,
//         > + Clone,
// {
//     // log::trace!("{:?}", a);
//     // let SimpleAction { path, action } = a;

//     // let from = match action {
//     //     Act::Move { from } => Some(from),
//     //     Act::MovUpd { from, .. } => Some(from),
//     //     _ => None,
//     // };

//     // let sub = if let Some(from) = from {
//     //     // apply remove
//     //     log::trace!("sub path {:?}", from.mid.iter().collect::<Vec<_>>());
//     //     let mut path = from.mid.iter();
//     //     let r = &mut roots[path.next().unwrap().to_usize().unwrap()];
//     //     let mut x: T::TreeId = r.to_owned();
//     //     let mut parents = vec![];
//     //     while let Some(p) = path.next() {
//     //         let node = s.resolve(&x);
//     //         let cs = node.get_children().to_vec();
//     //         parents.push((x, p, cs.clone()));
//     //         let i = p.to_usize().unwrap();
//     //         if i < cs.len() {
//     //             x = cs[i].clone();
//     //         } else {
//     //             assert!(path.next().is_none());
//     //             break;
//     //         }
//     //     }
//     //     log::trace!("parents {:?}", parents);
//     //     let (node, sub) = if let Some((x, i, cs)) = parents.pop() {
//     //         let mut children = Vec::with_capacity(cs.len() - 1);
//     //         children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//     //         children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//     //         // let n = s.resolve(&x);
//     //         // let t = n.get_type().clone();
//     //         // let l = n.get_label().clone();
//     //         // let node = BuildableTree::<T>::build(t, l, children);
//     //         // (s.get_or_insert(node), cs[i.to_usize().unwrap()].clone())
//     //         todo!()
//     //     } else {
//     //         // let mut children = Vec::with_capacity(cs.len() - 1);
//     //         // children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//     //         // children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//     //         // let node = s.resolve(&x);
//     //         // let node = Self::build(node.get_type(), node.get_label().clone(), children);
//     //         // s.get_or_insert(node)
//     //         (r.clone(), r.clone())
//     //     };
//     //     let mut node: T::TreeId = node;
//     //     for (x, i, cs) in parents.into_iter().rev() {
//     //         let mut children = Vec::with_capacity(cs.len() - 1);
//     //         children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//     //         children.push(node.clone());
//     //         children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//     //         let n = s.resolve(&x);
//     //         let t = n.get_type().to_owned();
//     //         let l = n.get_label().clone();
//     //         drop(n);
//     //         node = s.build_then_insert(t, l, children);
//     //         // let n: T = BuildableTree::build(n.get_type().to_owned(), n.get_label().clone(), children);
//     //         // node = s.get_or_insert(n);
//     //     }
//     //     *r = node;
//     //     Some(sub)
//     // } else {
//     //     None
//     // };

//     // let mut parents = vec![];
//     // log::trace!("{:?}", path.mid.iter().collect::<Vec<_>>());
//     // let mut path = path.mid.iter();
//     // let fp = path.next().unwrap().to_usize().unwrap();
//     // let r = if roots.len() > fp {
//     //     &mut roots[fp]
//     // } else if roots.len() == fp {
//     //     roots.push(roots[fp - 1].clone());
//     //     &mut roots[fp]
//     // } else {
//     //     panic!()
//     // };
//     // let mut x = *r;
//     // while let Some(p) = path.next() {
//     //     let node = s.resolve(&x);
//     //     let cs = node.get_children().to_vec();
//     //     parents.push((x, p, cs.clone()));
//     //     let i = p.to_usize().unwrap();
//     //     if i < cs.len() {
//     //         x = cs[i].clone();
//     //     } else {
//     //         log::error!("{:?} > {:?}", i, cs.len());
//     //         assert_eq!(path.next(), None);
//     //         break;
//     //     }
//     // }

//     // let node = match action {
//     //     Act::Delete {} => {
//     //         let (x, i, cs) = parents.pop().unwrap();
//     //         let mut children = Vec::with_capacity(cs.len() - 1);
//     //         children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//     //         children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//     //         let node = s.resolve(&x);
//     //         s.build_then_insert(node.get_type(), node.get_label().clone(), children)
//     //     }
//     //     Act::Insert { sub } => {
//     //         if let Some((x, i, cs)) = parents.pop() {
//     //             let mut children = Vec::with_capacity(cs.len());
//     //             children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//     //             let sub = {
//     //                 let node = s.resolve(sub);
//     //                 s.build_then_insert(node.get_type(), node.get_label().clone(), vec![])
//     //             };
//     //             children.push(sub);
//     //             if i.to_usize().unwrap() < cs.len() {
//     //                 children.extend_from_slice(&cs[i.to_usize().unwrap()..]);
//     //             }
//     //             let node = s.resolve(&x);
//     //             s.build_then_insert(node.get_type(), node.get_label().clone(), children)
//     //         } else {
//     //             let sub = {
//     //                 let node = s.resolve(sub);
//     //                 s.build_then_insert(node.get_type(), node.get_label().clone(), vec![])
//     //             };
//     //             // *r = sub.clone();
//     //             sub
//     //         }
//     //     }
//     //     Act::Update { new } => {
//     //         if let Some((x, i, cs)) = parents.pop() {
//     //             let mut children = Vec::with_capacity(cs.len());
//     //             children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//     //             let sub = {
//     //                 let x = cs[i.to_usize().unwrap()].clone();
//     //                 let node = s.resolve(&x);
//     //                 s.build_then_insert(node.get_type(), new.clone(), node.get_children().to_vec())
//     //             };
//     //             children.push(sub);
//     //             children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//     //             let node = s.resolve(&x);
//     //             s.build_then_insert(node.get_type(), node.get_label().clone(), children)
//     //         } else {
//     //             let node = s.resolve(&r);
//     //             let cs = node.get_children().to_vec();
//     //             let mut children = Vec::with_capacity(cs.len());
//     //             children.extend_from_slice(&cs[..]);
//     //             s.build_then_insert(node.get_type(), new.clone(), children)
//     //         }
//     //     }

//     //     Act::Move { .. } => {
//     //         // apply insert
//     //         let (x, i, cs) = parents.pop().unwrap();
//     //         let mut children = Vec::with_capacity(cs.len());
//     //         children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//     //         let sub = {
//     //             // let node = s.resolve(&sub.unwrap());
//     //             // let node = BuildableTree::build(node.get_type(), node.get_label().clone(), vec![]);
//     //             // s.get_or_insert(node)
//     //             sub.unwrap()
//     //         };
//     //         children.push(sub);
//     //         if i.to_usize().unwrap() < cs.len() {
//     //             children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//     //         }
//     //         let node = s.resolve(&x);
//     //         s.build_then_insert(node.get_type(), node.get_label().clone(), children)
//     //     }
//     //     Act::MovUpd { new, .. } => {
//     //         // apply insert
//     //         let (x, i, cs) = parents.pop().unwrap();
//     //         let mut children = Vec::with_capacity(cs.len());
//     //         children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//     //         let sub = {
//     //             // let node = s.resolve(&sub.unwrap());
//     //             // let node = BuildableTree::build(node.get_type(), node.get_label().clone(), vec![]);
//     //             // s.get_or_insert(node)
//     //             sub.unwrap()
//     //         };
//     //         children.push(sub);
//     //         if i.to_usize().unwrap() < cs.len() {
//     //             children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//     //         }
//     //         let node = s.resolve(&x);
//     //         s.build_then_insert(node.get_type(), new.clone(), children)
//     //     }
//     // };
//     // let mut node = node;
//     // for (x, i, cs) in parents.into_iter().rev() {
//     //     let mut children = Vec::with_capacity(cs.len() - 1);
//     //     children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//     //     children.push(node.clone());
//     //     children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//     //     let n = s.resolve(&x);
//     //     node = s.build_then_insert(n.get_type(), n.get_label().clone(), children);
//     // }
//     // *r = node;
//     // ss
// }

// pub trait ApplicableActions<T, R>
// where
//     T: for<'a> AsTreeRef<'a, R> + Stored + Typed + Labeled + WithChildren + std::cmp::PartialEq,
//     R: Stored<TreeId = T::TreeId> + Typed + Labeled + WithChildren + std::cmp::PartialEq,
//     T::Label: Clone + Debug,
//     T::ChildIdx: Clone + Debug,
//     T::TreeId: Debug + Clone,
// {
//     /// WARN for now it is very inneficient because it completly apply actions every times,
//     /// most likely it would need a temporary structure.
//     /// Also actions are applied in order, thus there is a single way of applying actions.
//     /// It might not have enough info to it flexibly, action_tree could definetly be more flexible.
//     // pub fn apply_actions<S: for<'b> NodeStoreMut<'b, <T as Stored>::TreeId, &'b T>>(
//     fn apply_actions<'a, S: NodeStoreMut<'a, T, &'a T>, It: Iterator<Item = &'a SimpleAction<T::Label,T::ChildIdx,T::TreeId>>>(
//         actions: It,
//         r: T::TreeId,
//         s: &'a mut S,
//     ) -> <T as Stored>::TreeId
//     where
//         T: 'a,
//     {
//         let mut roots = vec![r.clone()];
//         for a in actions {
//             // Self::apply_action(a, &mut roots, s)
//         }
//         r
//     }

//     fn apply_action<'a, S: 'a + NodeStoreMut<'a, R, T>>(
//         a: &SimpleAction<T::Label,T::ChildIdx,T::TreeId>,
//         roots: &mut Vec<T::TreeId>,
//         mut s: S,
//     ) -> S {
//         let (r_type, r_label) = {
//             let n = s.resolve(&roots[0]);
//             (n.get_type().clone(), n.get_label().clone())
//         };

//         log::trace!("{:?}", a);
//         let SimpleAction { path, action } = a;

//         let from = match action {
//             Act::Move { from } => Some(from),
//             Act::MovUpd { from, .. } => Some(from),
//             _ => None,
//         };

//         let sub = if let Some(from) = from {
//             // apply remove
//             log::trace!("sub path {:?}", from.mid.iter().collect::<Vec<_>>());
//             let mut path = from.mid.iter();
//             let r = &mut roots[path.next().unwrap().to_usize().unwrap()];
//             let mut x = r.clone();
//             let mut parents = vec![];
//             while let Some(p) = path.next() {
//                 let node = s.resolve(&x);
//                 let cs = node.get_children().to_vec();
//                 parents.push((x, p, cs.clone()));
//                 let i = p.to_usize().unwrap();
//                 if i < cs.len() {
//                     x = cs[i].clone();
//                 } else {
//                     assert!(path.next().is_none());
//                     break;
//                 }
//             }
//             log::trace!("parents {:?}", parents);
//             let (node, sub) = if let Some((x, i, cs)) = parents.pop() {
//                 let mut children = Vec::with_capacity(cs.len() - 1);
//                 children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                 children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                 let node = {
//                     let node = s.resolve(&x);
//                     Self::build(node.get_type().clone(), node.get_label().clone(), children)
//                 };
//                 (s.get_or_insert(node), cs[i.to_usize().unwrap()].clone())
//             } else {
//                 // let mut children = Vec::with_capacity(cs.len() - 1);
//                 // children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                 // children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                 // let node = s.resolve(&x);
//                 // let node = Self::build(node.get_type(), node.get_label().clone(), children);
//                 // s.get_or_insert(node)
//                 (r.clone(), r.clone())
//             };
//             let mut node = node;
//             for (x, i, cs) in parents.into_iter().rev() {
//                 let mut children = Vec::with_capacity(cs.len() - 1);
//                 children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                 children.push(node.clone());
//                 children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                 let n = s.resolve(&x);
//                 let n = Self::build(n.get_type(), n.get_label().clone(), children);
//                 node = s.get_or_insert(n);
//             }
//             *r = node;
//             Some(sub)
//         } else {
//             None
//         };

//         let mut parents = vec![];
//         log::trace!("{:?}", path.mid.iter().collect::<Vec<_>>());
//         let mut path = path.mid.iter();
//         let fp = path.next().unwrap().to_usize().unwrap();
//         let r = if roots.len() > fp {
//             &mut roots[fp]
//         } else if roots.len() == fp {
//             roots.push(roots[fp - 1].clone());
//             &mut roots[fp]
//         } else {
//             panic!()
//         };
//         let mut x = r.clone();
//         while let Some(p) = path.next() {
//             let node = s.resolve(&x);
//             let cs = node.get_children().to_vec();
//             parents.push((x, p, cs.clone()));
//             let i = p.to_usize().unwrap();
//             if i < cs.len() {
//                 x = cs[i].clone();
//             } else {
//                 log::error!("{:?} > {:?}", i, cs.len());
//                 assert_eq!(path.next(), None);
//                 break;
//             }
//         }

//         let node = match action {
//             Act::Delete {} => {
//                 let (x, i, cs) = parents.pop().unwrap();
//                 let mut children = Vec::with_capacity(cs.len() - 1);
//                 children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                 children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                 // let node = s.resolve(&x);
//                 // let node = Self::build(node.get_type(), node.get_label().clone(), children);

//                 // todo!()
//                 s.get_or_insert(Self::build(r_type, r_label, vec![]))
//                 // s.get_or_insert(node)
//             }
//             Act::Insert { sub } => {
//                 if let Some((x, i, cs)) = parents.pop() {
//                     let mut children = Vec::with_capacity(cs.len());
//                     children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                     let sub = {
//                         let node = s.resolve(sub);
//                         let node = Self::build(node.get_type(), node.get_label().clone(), vec![]);
//                         s.get_or_insert(node)
//                     };
//                     children.push(sub);
//                     if i.to_usize().unwrap() < cs.len() {
//                         children.extend_from_slice(&cs[i.to_usize().unwrap()..]);
//                     }
//                     let node = s.resolve(&x);
//                     let node = Self::build(node.get_type(), node.get_label().clone(), children);
//                     s.get_or_insert(node)
//                 } else {
//                     let sub = {
//                         let node = s.resolve(sub);
//                         let node = Self::build(node.get_type(), node.get_label().clone(), vec![]);
//                         s.get_or_insert(node)
//                     };
//                     // *r = sub.clone();
//                     sub
//                 }
//             }
//             Act::Update { new } => {
//                 if let Some((x, i, cs)) = parents.pop() {
//                     let mut children = Vec::with_capacity(cs.len());
//                     children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                     let sub = {
//                         let x = cs[i.to_usize().unwrap()].clone();
//                         let node = s.resolve(&x);
//                         let node =
//                             Self::build(node.get_type(), new.clone(), node.get_children().to_vec());
//                         s.get_or_insert(node)
//                     };
//                     children.push(sub);
//                     children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                     let node = s.resolve(&x);
//                     let node = Self::build(node.get_type(), node.get_label().clone(), children);
//                     s.get_or_insert(node)
//                 } else {
//                     let node = s.resolve(&r);
//                     let cs = node.get_children().to_vec();
//                     let mut children = Vec::with_capacity(cs.len());
//                     children.extend_from_slice(&cs[..]);
//                     let node = Self::build(node.get_type(), new.clone(), children);
//                     s.get_or_insert(node)
//                 }
//             }

//             Act::Move { .. } => {
//                 // apply insert
//                 let (x, i, cs) = parents.pop().unwrap();
//                 let mut children = Vec::with_capacity(cs.len());
//                 children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                 let sub = {
//                     // let node = s.resolve(&sub.unwrap());
//                     // let node = Self::build(node.get_type(), node.get_label().clone(), vec![]);
//                     // s.get_or_insert(node)
//                     sub.unwrap()
//                 };
//                 children.push(sub);
//                 if i.to_usize().unwrap() < cs.len() {
//                     children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                 }
//                 let node = s.resolve(&x);
//                 let node = Self::build(node.get_type(), node.get_label().clone(), children);
//                 s.get_or_insert(node)
//             }
//             Act::MovUpd { new, .. } => {
//                 // apply insert
//                 let (x, i, cs) = parents.pop().unwrap();
//                 let mut children = Vec::with_capacity(cs.len());
//                 children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//                 let sub = {
//                     // let node = s.resolve(&sub.unwrap());
//                     // let node = Self::build(node.get_type(), node.get_label().clone(), vec![]);
//                     // s.get_or_insert(node)
//                     sub.unwrap()
//                 };
//                 children.push(sub);
//                 if i.to_usize().unwrap() < cs.len() {
//                     children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//                 }
//                 let node = s.resolve(&x);
//                 let node = Self::build(node.get_type(), new.clone(), children);
//                 s.get_or_insert(node)
//             }
//         };
//         let mut node = node;
//         for (x, i, cs) in parents.into_iter().rev() {
//             let mut children = Vec::with_capacity(cs.len() - 1);
//             children.extend_from_slice(&cs[..i.to_usize().unwrap()]);
//             children.push(node.clone());
//             children.extend_from_slice(&cs[i.to_usize().unwrap() + 1..]);
//             let n = s.resolve(&x);
//             let n = Self::build(n.get_type(), n.get_label().clone(), children);
//             node = s.get_or_insert(n);
//         }
//         *r = node;
//         return s;
//     }

//     fn build<'a>(t: T::Type, l: T::Label, cs: Vec<T::TreeId>) -> T
//     where
//         T: 'a;
// }
