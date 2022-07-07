use crate::{
    actions::{
        action_vec::{ActionsVec, ApplicableActions, TestActions},
        bfs_wrapper,
        script_generator2::{Act, ApplicablePath, ScriptGenerator, SimpleAction},
        Actions,
    },
    matchers::{
        decompressed_tree_store::{CompletePostOrder, Initializable, ShallowDecompressedTreeStore},
        mapping_store::{DefaultMappingStore, MappingStore},
    },
    tests::{
        examples::{example_action, example_action2, example_gt_java_code},
    },
    tree::{
        simple_tree::{vpair_to_stores, DisplayTree, Tree, NS},
        tree::{LabelStore, Labeled, NodeStore, Stored, WithChildren},
    },
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
    let (label_store, node_store, src, dst) = vpair_to_stores(example_action());
    log::debug!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, src)
    );
    log::debug!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, dst)
    );
    let actions = {
        let mut ms = DefaultMappingStore::new();
        let src_arena = CompletePostOrder::<_, u16>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<_, u16>::new(&node_store, &dst);
        let src = &(src_arena.root());
        let dst = &(dst_arena.root());
        ms.topit(src_arena.len() + 1, dst_arena.len() + 1);
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
            let x = node_store.resolve(x).get_label();
            std::str::from_utf8(&label_store.resolve(x))
                .unwrap()
                .to_owned()
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
            Tree,
            _,
            bfs_wrapper::SD<_, _, CompletePostOrder<_, IdD>>,
            NS<Tree>,
        >::compute_actions(
            &node_store,
            &src_arena,
            &bfs_wrapper::SD::from(&node_store, &dst_arena),
            &ms,
        );

        let _lab = |x: &IdD| {
            std::str::from_utf8(&label_store.resolve(&node_store.resolve(x).get_label()))
                .unwrap()
                .to_owned()
        };

        log::debug!("{:?}", actions);

        let a = make_update(
            *node_store
                .resolve(&dst_arena.original(&from_dst(&[])))
                .get_label(),
            (&[], &[0]),
        ); // root renamed

        assert!(actions.has_actions(&[a,]));

        let a = make_insert(
            dst_arena.original(&from_dst(&[1])),
            (&[1], &[0, 2]), /* FIXME should be 1? */
        ); // h at a.2
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a]));

        let a = make_insert(
            dst_arena.original(&from_dst(&[2])),
            (&[2], &[0, 3]), /* FIXME should be 2? */
        ); // x at a.3
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        let a = make_move((&[0], &[0, 0]), (&[1, 0], &[0, 1, 0])); // e to h.0
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // ins u at j.0
        let a = make_insert(
            dst_arena.original(&from_dst(&[3, 0])),
            (&[3, 0], &[0, 5, 0]),
        );
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // upd f to y
        let a = make_update(
            *node_store
                .resolve(&dst_arena.original(&from_dst(&[1, 0, 0])))
                .get_label(),
            (&[0, 0], &[0, 1, 0, 0]),
        );
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // ins u at v.0
        let a = make_insert(
            dst_arena.original(&from_dst(&[3, 0, 0])),
            (&[3, 0, 0], &[0, 5, 0, 0]),
        );
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // mov k to v.0
        let a = make_move((&[4, 0], &[0, 5, 1]), (&[3, 0, 0, 0], &[0, 5, 0, 0, 0]));
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // del g
        let a = make_delete((&[2], &[0, 3]));
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // del i
        let a = make_delete((&[3], &[0, 3]));
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        assert_eq!(12, actions.len()); // FIXME should be 9 if actions are compressed
        actions
    };

    let mut node_store = node_store;
    let mut root = vec![src];
    for a in actions.iter() {
        log::debug!(
            "mid tree:\n{:?}",
            DisplayTree::new(&label_store, &node_store, *root.last().unwrap())
        );
        ActionsVec::apply_action(a, &mut root, &mut node_store);
    }
    let then = root; //ActionsVec::apply_actions(actions.iter(), *src, &mut node_store);
    assert_eq!(*then.last().unwrap(), dst);
}

#[test]
fn test_with_action_example2() {
    let (label_store, node_store, src, dst) = vpair_to_stores(example_action2());
    log::debug!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, src)
    );
    log::debug!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, dst)
    );
    let actions = {
        let mut ms = DefaultMappingStore::new();
        let src_arena = CompletePostOrder::<_, u16>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<_, u16>::new(&node_store, &dst);
        let src = &(src_arena.root());
        let dst = &(dst_arena.root());
        ms.topit(src_arena.len() + 1, dst_arena.len() + 1);
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
            let x = node_store.resolve(x).get_label();
            std::str::from_utf8(&label_store.resolve(x))
                .unwrap()
                .to_owned()
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
            Tree,
            _,
            bfs_wrapper::SD<_, _, CompletePostOrder<_, IdD>>,
            NS<Tree>,
        >::compute_actions(
            &node_store,
            &src_arena,
            &bfs_wrapper::SD::from(&node_store, &dst_arena),
            &ms,
        );

        let _lab = |x: &IdD| {
            std::str::from_utf8(&label_store.resolve(&node_store.resolve(x).get_label()))
                .unwrap()
                .to_owned()
        };

        log::debug!("{:?}", actions);

        let a = make_update(
            *node_store
                .resolve(&dst_arena.original(&from_dst(&[])))
                .get_label(),
            (&[], &[0]),
        ); // root renamed

        assert!(actions.has_actions(&[a,]));

        let a = make_insert(
            dst_arena.original(&from_dst(&[1])),
            (&[1], &[0, 2]),
        ); // h at a.2
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a]));

        let a = make_insert(
            dst_arena.original(&from_dst(&[2])),
            (&[2], &[0, 3]),
        ); // x at a.3
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        let a = make_move((&[0], &[0, 0]), (&[1, 0], &[0, 1, 0])); // e to h.0
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // ins u at j.0
        let a = make_insert(
            dst_arena.original(&from_dst(&[3, 0])),
            (&[3, 0], &[0, 6, 0]),
        );
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // upd f to y
        let a = make_update(
            *node_store
                .resolve(&dst_arena.original(&from_dst(&[1, 0, 0])))
                .get_label(),
            (&[0, 0], &[0, 1, 0, 0]),
        );
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // ins u at v.0
        let a = make_insert(
            dst_arena.original(&from_dst(&[3, 0, 0])),
            (&[3, 0, 0], &[0, 6, 0, 0]),
        );
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // mov k to v.0
        let a = make_move((&[5, 0], &[0, 6, 1]), (&[3, 0, 0, 0], &[0, 6, 0, 0, 0]));
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // del g
        let a = make_delete((&[2], &[0, 3]));
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // del i
        let a = make_delete((&[3], &[0, 3]));
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        assert_eq!(13, actions.len()); // FIXME should be 9 if actions are compressed
        actions
    };

    let mut node_store = node_store;
    let mut root = vec![src];
    for a in actions.iter() {
        log::debug!(
            "mid tree:\n{:?}",
            DisplayTree::new(&label_store, &node_store, *root.last().unwrap())
        );
        ActionsVec::apply_action(a, &mut root, &mut node_store);
    }
    let then = root; //ActionsVec::apply_actions(actions.iter(), *src, &mut node_store);
    assert_eq!(*then.last().unwrap(), dst);
}

pub(crate) fn make_move<T: Stored + Labeled + WithChildren>(
    from: (&[T::ChildIdx], &[T::ChildIdx]),
    to: (&[T::ChildIdx], &[T::ChildIdx]),
) -> SimpleAction<T> {
    SimpleAction {
        path: ApplicablePath {
            ori: to.0.into(),
            mid: to.1.into(),
        },
        action: Act::Move {
            from: ApplicablePath {
                ori: from.0.into(),
                mid: from.1.into(),
            },
        },
    }
}
pub(crate) fn make_move_update<T: Stored + Labeled + WithChildren>(
    from: (&[T::ChildIdx], &[T::ChildIdx]),
    new: T::Label,
    to: (&[T::ChildIdx], &[T::ChildIdx]),
) -> SimpleAction<T> {
    SimpleAction {
        path: ApplicablePath {
            ori: to.0.into(),
            mid: to.1.into(),
        },
        action: Act::MovUpd {
            new,
            from: ApplicablePath {
                ori: from.0.into(),
                mid: from.1.into(),
            },
        },
    }
}

pub(crate) fn make_delete<T: Stored + Labeled + WithChildren>(
    path: (&[T::ChildIdx], &[T::ChildIdx]),
) -> SimpleAction<T> {
    SimpleAction {
        path: ApplicablePath {
            ori: path.0.into(),
            mid: path.1.into(),
        },
        action: Act::Delete {},
    }
}

pub(crate) fn make_insert<T: Stored + Labeled + WithChildren>(
    sub: T::TreeId,
    path: (&[T::ChildIdx], &[T::ChildIdx]),
) -> SimpleAction<T> {
    SimpleAction {
        path: ApplicablePath {
            ori: path.0.into(),
            mid: path.1.into(),
        },
        action: Act::Insert { sub },
    }
}

pub(crate) fn make_update<T: Stored + Labeled + WithChildren>(
    new: T::Label,
    path: (&[T::ChildIdx], &[T::ChildIdx]),
) -> SimpleAction<T> {
    SimpleAction {
        path: ApplicablePath {
            ori: path.0.into(),
            mid: path.1.into(),
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
    let (label_store, node_store, src, dst) = vpair_to_stores(example_gt_java_code());
    log::debug!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, src)
    );
    log::debug!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, dst)
    );
    let actions = {
        let mut ms = DefaultMappingStore::new();
        let src_arena = CompletePostOrder::<_, IdD>::new(&node_store, &src);
        let dst_arena = CompletePostOrder::<_, IdD>::new(&node_store, &dst);
        let src = &(src_arena.root());
        let dst = &(dst_arena.root());
        ms.topit(src_arena.len() + 1, dst_arena.len() + 1);
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
            Tree,
            _,
            bfs_wrapper::SD<_, _, CompletePostOrder<_, IdD>>,
            NS<Tree>,
        >::compute_actions(
            &node_store,
            &src_arena,
            &bfs_wrapper::SD::from(&node_store, &dst_arena),
            &ms,
        );

        log::debug!("{:?}", actions);

        // // new Delete(src.getChild("1.1"))
        // assert!(actions.has_actions(&[SimpleAction::Delete {
        //     tree: from_src(&[1, 1]),
        // },]));
        let a = make_delete((&[1, 1], &[1, 0, 1, 2]));
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // assert!(actions.has_actions(&[
        //     // new Insert(dst, null, 0),
        //     SimpleAction::Insert {
        //         sub: dst_arena.original(&dst),
        //         parent: None,
        //         idx: 0,
        //     },
        // ]));
        let a = make_insert(dst_arena.original(&dst), (&[], &[1]));
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));

        // assert!(actions.has_actions(&[
        //     // new Move(src, dst, 0),
        //     SimpleAction::Move {
        //         sub: *src,
        //         parent: Some(*dst),
        //         idx: 0,
        //     },
        // ]));
        let a = make_move((&[], &[0]), (&[0], &[1, 0]));
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));
        // assert!(actions.has_actions(&[
        //     // new Update(src.getChild("1.3"), "r2"),
        //     SimpleAction::Update {
        //         src: from_src(&[1, 3]),
        //         dst: from_dst(&[0, 1, 3]),
        //         old: node_store
        //             .get_node_at_id(&src_arena.original(&from_src(&[1, 3])))
        //             .get_label(),
        //         new: node_store
        //             .get_node_at_id(&dst_arena.original(&from_dst(&[0, 1, 3])))
        //             .get_label(),
        //     }, // label: "r2".to_owned()},
        // ]));
        let a = make_update(
            *node_store
                .resolve(&dst_arena.original(&from_dst(&[0, 1, 3])))
                .get_label(),
            (&[1, 3], &[1, 0, 1, 4]),
        );
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));
        // assert!(actions.has_actions(&[
        //     // new Insert(dst.getChild("0.1.1"), src.getChild("1"), 1),
        //     SimpleAction::Insert {
        //         sub: dst_arena.original(&from_dst(&[0, 1, 1])),
        //         parent: Some(from_dst(&[0, 1])),
        //         idx: 1,
        //     },
        // ]));
        let a = make_insert(
            dst_arena.original(&from_dst(&[0, 1, 1])),
            (&[0, 1, 1], &[1, 0, 1, 1]),
        );
        log::debug!("{:?}", a);
        assert!(actions.has_actions(&[a,]));
        assert_eq!(5, actions.len());

        // assert_eq!(
        //     label_store
        //         .get_node_at_id(
        //             &node_store
        //                 .get_node_at_id(&from_dst(&[0, 1, 3]))
        //                 .get_label()
        //         )
        //         .to_owned(),
        //     b"r2"
        // );

        // actions = new SimplifiedChawatheScriptGenerator().computeActions(ms);
        // assertEquals(5, actions.size());
        // assertThat(actions, hasItems(
        //         new Insert(dst, null, 0),
        //         new Move(src, dst, 0),
        //         new Insert(dst.getChild("0.1.1"), src.getChild("1"), 1),
        //         new Update(src.getChild("1.3"), "r2"),
        //         new Delete(src.getChild("1.1"))
        // ));

        // actions = new InsertDeleteChawatheScriptGenerator().computeActions(ms);

        // assertEquals(7, actions.size());
        // assertThat(actions, hasItems(
        //         new Insert(dst, null, 0),
        //         new TreeDelete(src),
        //         new TreeInsert(dst.getChild(0), dst, 0),
        //         new Insert(dst.getChild("0.1.1"), src.getChild("1"), 1),
        //         new Delete(src.getChild("1.1")),
        //         new Delete(src.getChild("1.3")),
        //         new Insert(dst.getChild("0.1.1"), src.getChild(1), 1)
        // ));
        actions
    };

    let mut node_store = node_store;
    let mut root = vec![src];
    for a in actions.iter() {
        log::debug!(
            "mid tree:\n{:?}",
            DisplayTree::new(&label_store, &node_store, *root.last().unwrap())
        );
        ActionsVec::apply_action(a, &mut root, &mut node_store);
    }
    log::debug!(
        "mid tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, *root.last().unwrap())
    );
    log::debug!("{:?}", root);
    let then = *root.last().unwrap(); //ActionsVec::apply_actions(actions.iter(), *src, &mut node_store);
    assert_eq!(then, dst);
}
