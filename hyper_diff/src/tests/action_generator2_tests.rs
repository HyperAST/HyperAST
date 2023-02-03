use crate::actions::action_vec::{apply_action, apply_actions};
use crate::decompressed_tree_store::bfs_wrapper::SimpleBfsMapper;
use crate::tree::simple_tree::Tree;
use crate::tree::tree_path::CompressedTreePath;
use crate::tree::tree_path::TreePath;
use crate::{
    actions::{
        action_vec::{ActionsVec, TestActions},
        script_generator2::{Act, ApplicablePath, ScriptGenerator, SimpleAction},
        Actions,
    },
    decompressed_tree_store::{CompletePostOrder, ShallowDecompressedTreeStore},
    matchers::mapping_store::{DefaultMappingStore, MappingStore},
    tests::examples::{example_action, example_action2, example_gt_java_code},
    tree::simple_tree::{vpair_to_stores, DisplayTree, TreeRef, NS},
};
use hyper_ast::types::{
    LabelStore, Labeled, NodeStore, NodeStoreExt, Stored, Tree as _, Typed, WithChildren, DecompressedSubtree,
};
use std::fmt;

type IdD = u16;

pub struct Fmt<F>(pub F)
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result;

impl<F> fmt::Debug for Fmt<F>
where
    F: Fn(&mut fmt::Formatter) -> fmt::Result,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (self.0)(f)
    }
}

#[test]
fn test_with_action_example() {
    let (label_store, mut node_store, src, dst) = vpair_to_stores(example_action());
    log::debug!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, src)
    );
    log::debug!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, dst)
    );
    let mut ms = DefaultMappingStore::default();
    let src_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &src);
    let dst_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &dst);
    let dst_arena2 = SimpleBfsMapper::from(&node_store, &dst_arena);
    let actions = {
        let src = &(src_arena.root());
        let dst = &(dst_arena.root());
        ms.topit(src_arena.len(), dst_arena.len());
        let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
        let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
        ms.link(from_src(&[]), from_dst(&[]));
        ms.link(from_src(&[1]), from_dst(&[0]));
        ms.link(from_src(&[1, 0]), from_dst(&[0, 0]));
        ms.link(from_src(&[1, 1]), from_dst(&[0, 1]));
        ms.link(from_src(&[0]), from_dst(&[1, 0]));
        ms.link(from_src(&[0, 0]), from_dst(&[1, 0, 0]));
        ms.link(from_src(&[4]), from_dst(&[3]));
        ms.link(from_src(&[4, 0]), from_dst(&[3, 0, 0, 0]));

        let g = |x: &u16| -> String {
            let n = node_store.resolve(x);
            let x = n.get_label();
            label_store.resolve(x).to_string()
        };

        log::debug!(
            "#src\n{:?}",
            Fmt(|f| {
                src_arena
                    .iter()
                    .enumerate()
                    .for_each(|(i, x)| write!(f, "[{}]: {}\n", i, g(x)).unwrap());
                write!(f, "")
            })
        );

        log::debug!(
            "#dst\n{:?}",
            Fmt(|f| {
                dst_arena
                    .iter()
                    .enumerate()
                    .for_each(|(i, x)| write!(f, "[{}]: {}\n", i, g(x)).unwrap());
                write!(f, "")
            })
        );
        let actions: ActionsVec<SimpleAction<u16, CompressedTreePath<u8>, u16>> =
            ScriptGenerator::<
                _,
                TreeRef<Tree>,
                _,
                SimpleBfsMapper<_, _, CompletePostOrder<_, IdD>, _>,
                NS<Tree>,
                _,
                _,
            >::_compute_actions(&node_store, &src_arena, &dst_arena2, &ms).unwrap();

        log::debug!("{:?}", actions);

        macro_rules! test_action {
            ( ins $at:expr, $to:expr ) => {{
                let a = make_insert::<Tree, CompressedTreePath<_>>(dst_arena.original(&from_dst(&$at)), (&$at, &$to));
                log::debug!("{:?}", a);
                assert!(actions.has_actions(&[a]));
            }};
            ( del $at:expr, $to:expr ) => {{
                let a = make_delete::<Tree, CompressedTreePath<_>>((&$at, &$to));
                log::debug!("{:?}", a);
                assert!(actions.has_actions(&[a]));
            }};
            ( upd $lab:expr; $at:expr, $to:expr ) => {{
                let a = make_update::<Tree, CompressedTreePath<_>>(label_store.get($lab).unwrap(), (&$at, &$to));
                log::debug!("{:?}", a);
                assert!(actions.has_actions(&[a]));
            }};
            ( mov $from:expr, $m_from:expr => $to:expr, $m_to:expr ) => {{
                let a = make_move::<Tree, CompressedTreePath<_>>((&$from, &$m_from), (&$to, &$m_to));
                log::debug!("{:?}", a);
                assert!(actions.has_actions(&[a]));
            }};
        }
        test_action!(upd "Z"; [], [0]); // root renamed

        test_action!(ins[1], [0, 2]); // h at a.2

        test_action!(ins[2], [0, 3]); // x at a.3

        test_action!(mov [0], [0, 0] => [1, 0], [0, 1, 0]); // e to h.0

        test_action!(ins [3, 0], [0, 5, 0]); // ins u at j.0

        test_action!(upd "y"; [0, 0], [0, 1, 0, 0]); // upd f to y

        test_action!(ins [3, 0, 0], [0, 5, 0, 0]); // ins u at v.0

        test_action!(mov [4, 0], [0, 5, 1] => [3, 0, 0, 0], [0, 5, 0, 0, 0]); // mov k to v.0

        test_action!(del[2], [0, 3]); // del g

        test_action!(del[3], [0, 3]); // del i

        assert_eq!(12, actions.len());
        actions
    };

    let mut root = vec![src];
    {
        let node = node_store.resolve(&root[0]);
        let t = node.get_type();
        let l = node.try_get_label().cloned();
        drop(node);
        node_store.build_then_insert(root[0], t, l, vec![]);
    }
    apply_actions::<_, NS<Tree>, _>(actions, &mut root, &mut node_store);
    let then = root; //ActionsVec::apply_actions(actions.iter(), *src, &mut node_store);
    assert_eq!(*then.last().unwrap(), dst);
}
// use aaa::*;
// mod aaa {
//     use super::*;
//     use std::fmt::Debug;
//     pub(crate) fn apply_actions<T, S>(
//         actions: ActionsVec<SimpleAction<T::Label, T::ChildIdx, T::TreeId>>,
//         root: &mut Vec<T::TreeId>,
//         node_store: &mut S,
//     ) where
//         T: hyper_ast::types::Tree,
//         T::Type: Debug + Copy + Default,
//         T::Label: Debug + Copy + Default,
//         T::TreeId: Debug + Copy + Default,
//         T::ChildIdx: Debug + Copy + Default,
//         S: NodeStoreExt2<T> + NodeStore2<T::TreeId>, //NodeStoreExt<'a, T, R>,
//         for<'d> S::R<'d>: hyper_ast::types::Tree<
//             TreeId = T::TreeId,
//             Type = T::Type,
//             Label = T::Label,
//             ChildIdx = T::ChildIdx,
//         >,
//     {
//         for a in actions.iter() {
//             // *log::debug!(
//             //     "mid tree:\n{:?}",
//             //     DisplayTree::new(&label_store, &node_store, *root.last().unwrap())
//             // );
//             // apply_action(a, root, node_store);
//         }
//     }

//     pub(crate) fn apply_action<T, S>(
//         a: &SimpleAction<T::Label, T::ChildIdx, T::TreeId>,
//         root: &mut Vec<T::TreeId>,
//         s: &mut S,
//     ) where
//         T: hyper_ast::types::Tree,
//         T::Type: Debug + Copy + Default,
//         T::Label: Debug + Copy + Default,
//         T::TreeId: Debug + Copy + Default,
//         T::ChildIdx: Debug + Copy + Default,
//         S: NodeStoreExt2<T> + NodeStore2<T::TreeId>, //NodeStoreExt<'a, T, R>,
//         for<'d> S::R<'d>: hyper_ast::types::Tree<
//             TreeId = T::TreeId,
//             Type = T::Type,
//             Label = T::Label,
//             ChildIdx = T::ChildIdx,
//         >,
//     {
//         let fun_name = |s: &mut S, x: &T::TreeId| -> (T::Type, Option<T::Label>) {
//             // let node = s.resolve(x);
//             // let t = node.get_type().to_owned();
//             // let l = node.get_label().to_owned();
//             // (t, l)
//             todo!()
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

#[test]
fn test_with_action_example2() {
    // env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
    //     .is_test(true)
    //     .init();
    let (label_store, mut node_store, src, dst) = vpair_to_stores(example_action2());
    log::debug!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, src)
    );
    log::debug!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, dst)
    );
    let mut ms = DefaultMappingStore::default();
    let src_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &src);
    let dst_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &dst);
    let dst_arena2 = SimpleBfsMapper::from(&node_store, &dst_arena);

    let actions = {
        let src = &(src_arena.root());
        let dst = &(dst_arena.root());
        ms.topit(src_arena.len(), dst_arena.len());
        let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
        let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
        ms.link(from_src(&[]), from_dst(&[]));
        ms.link(from_src(&[1]), from_dst(&[0]));
        ms.link(from_src(&[1, 0]), from_dst(&[0, 0]));
        ms.link(from_src(&[1, 1]), from_dst(&[0, 1]));
        ms.link(from_src(&[0]), from_dst(&[1, 0]));
        ms.link(from_src(&[0, 0]), from_dst(&[1, 0, 0]));
        ms.link(from_src(&[5]), from_dst(&[3]));
        ms.link(from_src(&[5, 0]), from_dst(&[3, 0, 0, 0]));

        let g = |x: &u16| -> String {
            let n = node_store.resolve(x);
            let x = n.get_label();
            label_store.resolve(x).to_string()
        };

        log::debug!(
            "#src\n{:?}",
            Fmt(|f| {
                src_arena
                    .iter()
                    .enumerate()
                    .for_each(|(i, x)| write!(f, "[{}]: {}\n", i, g(x)).unwrap());
                write!(f, "")
            })
        );

        log::debug!(
            "#dst\n{:?}",
            Fmt(|f| {
                dst_arena
                    .iter()
                    .enumerate()
                    .for_each(|(i, x)| write!(f, "[{}]: {}\n", i, g(x)).unwrap());
                write!(f, "")
            })
        );

        let actions = ScriptGenerator::<
            _,
            TreeRef<Tree>,
            _,
            SimpleBfsMapper<_, _, CompletePostOrder<_, IdD>, _>,
            NS<Tree>,
            _,
            _,
        >::_compute_actions(&node_store, &src_arena, &dst_arena2, &ms).unwrap();

        log::debug!("{:?}", actions);

        macro_rules! test_action {
            ( ins $at:expr, $to:expr ) => {{
                let a = make_insert::<Tree, CompressedTreePath<_>>(dst_arena.original(&from_dst(&$at)), (&$at, &$to));
                log::debug!("{:?}", a);
                assert!(actions.has_actions(&[a]));
            }};
            ( del $at:expr, $to:expr ) => {{
                let a = make_delete::<Tree, CompressedTreePath<_>>((&$at, &$to));
                log::debug!("{:?}", a);
                assert!(actions.has_actions(&[a]));
            }};
            ( upd $lab:expr; $at:expr, $to:expr ) => {{
                let a = make_update::<Tree, CompressedTreePath<_>>(label_store.get($lab).unwrap(), (&$at, &$to));
                log::debug!("{:?}", a);
                assert!(actions.has_actions(&[a]));
            }};
            ( mov $from:expr, $m_from:expr => $to:expr, $m_to:expr ) => {{
                let a = make_move::<Tree, CompressedTreePath<_>>((&$from, &$m_from), (&$to, &$m_to));
                log::debug!("{:?}", a);
                assert!(actions.has_actions(&[a]));
            }};
        }
        test_action!(upd "Z"; [], [0]); // root renamed

        test_action!(ins[1], [0, 2]); // h at a.2

        test_action!(ins[2], [0, 3]); // x at a.3

        test_action!(mov [0], [0, 0] => [1, 0], [0, 1, 0]); // e to h.0

        test_action!(ins [3, 0], [0, 6, 0]); // ins u at j.0

        test_action!(upd "y"; [0, 0], [0, 1, 0, 0]); // upd f to y

        test_action!(ins [3, 0, 0], [0, 6, 0, 0]); // ins u at v.0

        test_action!(mov [5, 0], [0, 6, 1] => [3, 0, 0, 0], [0, 6, 0, 0, 0]); // mov k to v.0

        test_action!(del[2], [0, 3]); // del g

        test_action!(del[3], [0, 3]); // del i

        assert_eq!(13, actions.len());
        actions
    };

    let mut root = vec![src];
    for a in actions.iter() {
        log::debug!(
            "mid tree:\n{:?}",
            DisplayTree::new(&label_store, &node_store, *root.last().unwrap())
        );
        apply_action::<_, NS<Tree>, _>(a, &mut root, &mut node_store);
    }
    let then = root; //ActionsVec::apply_actions(actions.iter(), *src, &mut node_store);
    assert_eq!(*then.last().unwrap(), dst);
}

pub(crate) fn make_move<T: Stored + Labeled + WithChildren, P>(
    from: (&[T::ChildIdx], &[T::ChildIdx]),
    to: (&[T::ChildIdx], &[T::ChildIdx]),
) -> SimpleAction<T::Label, P, T::TreeId>
where
    <T as WithChildren>::ChildIdx: num_traits::ToPrimitive,
    P: TreePath<Item = T::ChildIdx> + From<Vec<T::ChildIdx>>,
{
    SimpleAction {
        path: ApplicablePath {
            ori: to.0.to_vec().into(),
            mid: to.1.to_vec().into(),
        },
        action: Act::Move {
            from: ApplicablePath {
                ori: from.0.to_vec().into(),
                mid: from.1.to_vec().into(),
            },
        },
    }
}
pub(crate) fn make_move_update<T: Stored + Labeled + WithChildren, P>(
    from: (&[T::ChildIdx], &[T::ChildIdx]),
    new: T::Label,
    to: (&[T::ChildIdx], &[T::ChildIdx]),
) -> SimpleAction<T::Label, P, T::TreeId>
where
    <T as WithChildren>::ChildIdx: num_traits::ToPrimitive,
    P: TreePath<Item = T::ChildIdx> + From<Vec<T::ChildIdx>>,
{
    SimpleAction {
        path: ApplicablePath {
            ori: to.0.to_vec().into(),
            mid: to.1.to_vec().into(),
        },
        action: Act::MovUpd {
            new,
            from: ApplicablePath {
                ori: from.0.to_vec().into(),
                mid: from.1.to_vec().into(),
            },
        },
    }
}

pub(crate) fn make_delete<T: Stored + Labeled + WithChildren, P>(
    path: (&[T::ChildIdx], &[T::ChildIdx]),
) -> SimpleAction<T::Label, P, T::TreeId>
where
    <T as WithChildren>::ChildIdx: num_traits::ToPrimitive,
    P: TreePath<Item = T::ChildIdx> + From<Vec<T::ChildIdx>>,
{
    SimpleAction {
        path: ApplicablePath {
            ori: path.0.to_vec().into(),
            mid: path.1.to_vec().into(),
        },
        action: Act::Delete {},
    }
}

pub(crate) fn make_insert<T: Stored + Labeled + WithChildren, P>(
    sub: T::TreeId,
    path: (&[T::ChildIdx], &[T::ChildIdx]),
) -> SimpleAction<T::Label, P, T::TreeId>
where
    <T as WithChildren>::ChildIdx: num_traits::ToPrimitive,
    P: TreePath<Item = T::ChildIdx> + From<Vec<T::ChildIdx>>,
{
    SimpleAction {
        path: ApplicablePath {
            ori: path.0.to_vec().into(),
            mid: path.1.to_vec().into(),
        },
        action: Act::Insert { sub },
    }
}

pub(crate) fn make_update<T: Stored + Labeled + WithChildren, P>(
    new: T::Label,
    path: (&[T::ChildIdx], &[T::ChildIdx]),
) -> SimpleAction<T::Label, P, T::TreeId>
where
    <T as WithChildren>::ChildIdx: num_traits::ToPrimitive,
    P: TreePath<Item = T::ChildIdx> + From<Vec<T::ChildIdx>>,
{
    SimpleAction {
        path: ApplicablePath {
            ori: path.0.to_vec().into(),
            mid: path.1.to_vec().into(),
        },
        action: Act::Update { new },
    }
}

#[test]
fn test_with_unmapped_root() {
    todo!()
    // ITree src = new Tree(TypeSet.type("foo"), "");
    // ITree dst = new Tree(TypeSet.type("bar"), "");
    // MappingStore ms = new MappingStore(src, dst);
    // EditScript actions = new SimplifiedChawatheScriptGenerator().computeActions(ms);
    // for (Action a : actions)
    //     System.out.println(a.toString());
}

#[test]
fn test_with_action_example_no_move() {
    todo!()
    // Pair<TreeContext, TreeContext> trees = TreeLoader.getActionPair();
    // ITree src = trees.first.getRoot();
    // ITree dst = trees.second.getRoot();
    // MappingStore ms = new MappingStore(src, dst);
    // ms.addMapping(src, dst);
    // ms.addMapping(src.getChild(1), dst.getChild(0));
    // ms.addMapping(src.getChild(1).getChild(0), dst.getChild(0).getChild(0));
    // ms.addMapping(src.getChild(1).getChild(1), dst.getChild(0).getChild(1));
    // ms.addMapping(src.getChild(0), dst.getChild(1).getChild(0));
    // ms.addMapping(src.getChild(0).getChild(0), dst.getChild(1).getChild(0).getChild(0));
    // ms.addMapping(src.getChild(4), dst.getChild(3));
    // ms.addMapping(src.getChild(4).getChild(0), dst.getChild(3).getChild(0).getChild(0).getChild(0));

    // EditScript actions = new InsertDeleteChawatheScriptGenerator().computeActions(ms);

    // for (Action a : actions)
    //     System.out.println(a.toString());
}
#[test]
fn test_with_zs_custom_example() {
    let (label_store, mut node_store, src, dst) = vpair_to_stores(example_gt_java_code());
    log::debug!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, src)
    );
    log::debug!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, dst)
    );
    let src_arena = CompletePostOrder::<_, IdD>::decompress(&node_store, &src);
    let dst_arena = CompletePostOrder::<_, IdD>::decompress(&node_store, &dst);
    let dst_arena2 = SimpleBfsMapper::from(&node_store, &dst_arena);
    let mut ms = DefaultMappingStore::default();
    let actions = {
        let src = &(src_arena.root());
        let dst = &(dst_arena.root());
        ms.topit(src_arena.len(), dst_arena.len());
        let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
        let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
        // ms.addMapping(src, dst.getChild(0));
        ms.link(from_src(&[]), from_dst(&[0]));
        // ms.addMapping(src.getChild(0), dst.getChild("0.0"));
        ms.link(from_src(&[0]), from_dst(&[0, 0]));
        // ms.addMapping(src.getChild(1), dst.getChild("0.1"));
        ms.link(from_src(&[1]), from_dst(&[0, 1]));
        // ms.addMapping(src.getChild("1.0"), dst.getChild("0.1.0"));
        ms.link(from_src(&[1, 0]), from_dst(&[0, 1, 0]));
        // ms.addMapping(src.getChild("1.2"), dst.getChild("0.1.2"));
        ms.link(from_src(&[1, 2]), from_dst(&[0, 1, 2]));
        // ms.addMapping(src.getChild("1.3"), dst.getChild("0.1.3"));
        ms.link(from_src(&[1, 3]), from_dst(&[0, 1, 3]));

        let actions = ScriptGenerator::<
            _,
            TreeRef<Tree>,
            _,
            SimpleBfsMapper<_, _, CompletePostOrder<_, IdD>, _>,
            NS<Tree>,
            _,
            _,
        >::_compute_actions(&node_store, &src_arena, &dst_arena2, &ms).unwrap();

        log::debug!("{:?}", actions);
        macro_rules! test_action {
            ( ins $at:expr, $to:expr ) => {{
                let a = make_insert::<Tree, CompressedTreePath<_>>(
                    dst_arena.original(&from_dst(&$at)),
                    (&$at, &$to),
                );
                log::debug!("{:?}", a);
                assert!(actions.has_actions(&[a]));
            }};
            ( del $at:expr, $to:expr ) => {{
                let a = make_delete::<Tree, CompressedTreePath<_>>((&$at, &$to));
                log::debug!("{:?}", a);
                assert!(actions.has_actions(&[a]));
            }};
            ( upd $lab:expr; $at:expr, $to:expr ) => {{
                let a = make_update::<Tree, CompressedTreePath<_>>(label_store.get($lab).unwrap(), (&$at, &$to));
                log::debug!("{:?}", a);
                assert!(actions.has_actions(&[a]));
            }};
            ( mov $from:expr, $m_from:expr => $to:expr, $m_to:expr ) => {{
                let a = make_move::<Tree, CompressedTreePath<_>>((&$from, &$m_from), (&$to, &$m_to));
                log::debug!("{:?}", a);
                assert!(actions.has_actions(&[a]));
            }};
        }

        test_action!(del [1, 1], [1, 0, 1, 2]);

        test_action!(ins [], [1]);

        test_action!(mov [], [0] => [0], [1, 0]);

        test_action!(upd "r2"; [1, 3], [1, 0, 1, 4]);

        test_action!(ins [0, 1, 1], [1, 0, 1, 1]);

        assert_eq!(5, actions.len());
        actions
    };

    let mut root = vec![src];
    for a in actions.iter() {
        log::debug!(
            "mid tree:\n{:?}",
            DisplayTree::new(&label_store, &node_store, *root.last().unwrap())
        );
        apply_action::<_, NS<Tree>, _>(a, &mut root, &mut node_store);
    }
    log::debug!(
        "mid tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, *root.last().unwrap())
    );
    log::debug!("{:?}", root);
    let then = *root.last().unwrap(); //ActionsVec::apply_actions(actions.iter(), *src, &mut node_store);
    assert_eq!(then, dst);
}
