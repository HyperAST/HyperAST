use crate::decompressed_tree_store::bfs_wrapper::SimpleBfsMapper;
use crate::tree::simple_tree::Tree;
use crate::{
    actions::script_generator::{self, Actions, SimpleAction, TestActions},
    decompressed_tree_store::{CompletePostOrder, ShallowDecompressedTreeStore},
    matchers::mapping_store::{DefaultMappingStore, MappingStore},
    tests::examples::{example_action, example_gt_java_code},
    tree::simple_tree::{vpair_to_stores, DisplayTree, TreeRef},
};
use hyper_ast::types::{LabelStore, Labeled, NodeStore, DecompressedSubtree};
use std::fmt;

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

    println!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, src)
    );
    println!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, dst)
    );

    let mut ms = DefaultMappingStore::default();
    let src_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &src);
    let dst_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &dst);
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
    println!(
        "#src\n{:?}",
        Fmt(|f| {
            src_arena
                .iter()
                .enumerate()
                .for_each(|(i, x)| write!(f, "[{}]: {}\n", i, g(x)).unwrap());
            write!(f, "")
        })
    );

    println!(
        "#dst\n{:?}",
        Fmt(|f| {
            dst_arena
                .iter()
                .enumerate()
                .for_each(|(i, x)| write!(f, "[{}]: {}\n", i, g(x)).unwrap());
            write!(f, "")
        })
    );

    let dst_arena = SimpleBfsMapper::from(&node_store, &dst_arena);
    // let actions = script_generator::ScriptGenerator::<
    //     _,
    //     TreeRef<Tree>,
    //     _,
    //     bfs_wrapper::SD<_, _, CompletePostOrder<_, IdD>>,
    //     _,
    // >::compute_actions(
    //     &node_store,
    //     &src_arena,
    //     &dst_arena,
    //     &ms,
    // );

    let actions = script_generator::ScriptGenerator::<
        _,
        TreeRef<Tree>,
        _,
        SimpleBfsMapper<_, _, CompletePostOrder<_, IdD>, _>,
        _,
    >::compute_actions(&node_store, &src_arena, &dst_arena, &ms);

    let lab = |x: &IdD| {
        label_store
            .resolve(&node_store.resolve(x).get_label())
            .to_string()
    };

    println!("{:?}", actions);

    let a = SimpleAction::Update {
        src: from_src(&[]),
        dst: from_dst(&[]),
        old: *node_store
            .resolve(&src_arena.original(&from_src(&[])))
            .get_label(),
        new: *node_store
            .resolve(&dst_arena.original(&from_dst(&[])))
            .get_label(),
    };

    assert!(actions.has_actions(&[a,]));

    // Action a = actions.get(0);
    // assertTrue(a instanceof Insert);
    // Insert i = (Insert) a;
    // assertEquals("h", i.getNode().getLabel());
    // assertEquals("a", i.getParent().getLabel());
    // assertEquals(2, i.getPosition());

    let tmp = from_dst(&[1]);
    println!("{}", lab(&tmp));
    let a = SimpleAction::Insert {
        sub: dst_arena.original(&from_dst(&[1])),
        parent: Some(*dst),
        idx: 2, // FIXME should be 1? in future ( due to parent ref issue)
    };
    println!("{:?}", a);
    assert!(actions.has_actions(&[a]));

    // a = actions.get(1);
    // assertTrue(a instanceof TreeInsert);
    // TreeInsert ti = (TreeInsert) a;
    // assertEquals("x", ti.getNode().getLabel());
    // assertEquals("a", ti.getParent().getLabel());
    // assertEquals(3, ti.getPosition());
    let a = SimpleAction::Insert {
        sub: dst_arena.original(&from_dst(&[2])),
        parent: Some(*dst),
        idx: 3, // FIXME 2 ?
    };
    assert!(actions.has_actions(&[a,]));

    // // a = actions.get(2);
    // // assertTrue(a instanceof Move);
    // // Move m = (Move) a;
    // // assertEquals("e", m.getNode().getLabel());
    // // assertEquals("h", m.getParent().getLabel());
    // // assertEquals(0, m.getPosition());
    let a = SimpleAction::Move {
        sub: from_src(&[1]),
        parent: Some(from_dst(&[1])),
        idx: 0,
    };
    assert!(actions.has_actions(&[a,]));

    // a = actions.get(3);
    // assertTrue(a instanceof Insert);
    // Insert i2 = (Insert) a;
    // assertEquals("u", i2.getNode().getLabel());
    // assertEquals("j", i2.getParent().getLabel());
    // assertEquals(0, i2.getPosition());
    let a = SimpleAction::Insert {
        sub: dst_arena.original(&from_dst(&[3, 0])),
        parent: Some(from_dst(&[3])),
        idx: 0,
    };
    assert!(actions.has_actions(&[a,]));

    // a = actions.get(4);
    // assertTrue(a instanceof Update);
    // Update u = (Update) a;
    // assertEquals("f", u.getNode().getLabel());
    // assertEquals("y", u.getValue());
    let a = SimpleAction::Update {
        src: from_src(&[0, 0]),
        dst: from_dst(&[1, 0, 0]),
        old: *node_store
            .resolve(&src_arena.original(&from_src(&[0, 0])))
            .get_label(),
        new: *node_store
            .resolve(&dst_arena.original(&from_dst(&[1, 0, 0])))
            .get_label(),
    };
    assert!(actions.has_actions(&[a,]));

    // a = actions.get(5);
    // assertTrue(a instanceof Insert);
    // Insert i3 = (Insert) a;
    // assertEquals("v", i3.getNode().getLabel());
    // assertEquals("u", i3.getParent().getLabel());
    // assertEquals(0, i3.getPosition());
    assert!(actions.has_actions(&[SimpleAction::Insert {
        sub: dst_arena.original(&from_dst(&[3, 0, 0])),
        parent: Some(from_dst(&[3, 0])),
        idx: 0
    },]));

    // a = actions.get(6);
    // assertTrue(a instanceof Move);
    // Move m2 = (Move) a;
    // assertEquals("k", m2.getNode().getLabel());
    // assertEquals("v", m2.getParent().getLabel());
    // assertEquals(0, m.getPosition());
    let a = SimpleAction::Move {
        sub: from_src(&[4, 0]),
        parent: Some(from_dst(&[3, 0, 0])),
        idx: 0,
    };
    assert!(actions.has_actions(&[a,]));

    // a = actions.get(7);
    // assertTrue(a instanceof TreeDelete);
    // TreeDelete td = (TreeDelete) a;
    // assertEquals("g", td.getNode().getLabel());
    assert!(actions.has_actions(&[SimpleAction::Delete {
        tree: from_src(&[2]),
    },]));

    // a = actions.get(8);
    // assertTrue(a instanceof Delete);
    // Delete d = (Delete) a;
    // assertEquals("i", d.getNode().getLabel());
    assert!(actions.has_actions(&[SimpleAction::Delete {
        tree: from_src(&[3]),
    },]));
    assert_eq!(12, actions.len()); // FIXME should be 9 if actions are compressed
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
type IdD = u16;
#[test]
fn test_with_zs_custom_example() {
    let (_, node_store, src, dst) = vpair_to_stores(example_gt_java_code());
    let mut ms = DefaultMappingStore::default();
    let src_arena = CompletePostOrder::<_, IdD>::decompress(&node_store, &src);
    let dst_arena = CompletePostOrder::<_, IdD>::decompress(&node_store, &dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len(), dst_arena.len());
    let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
    let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
    ms.link(from_src(&[]), from_dst(&[0]));
    ms.link(from_src(&[0]), from_dst(&[0, 0]));
    ms.link(from_src(&[1]), from_dst(&[0, 1]));
    ms.link(from_src(&[1, 0]), from_dst(&[0, 1, 0]));
    ms.link(from_src(&[1, 2]), from_dst(&[0, 1, 2]));
    ms.link(from_src(&[1, 3]), from_dst(&[0, 1, 3]));

    let dst_arena = SimpleBfsMapper::from(&node_store, &dst_arena);
    // let actions = script_generator::ScriptGenerator::<
    //     _,
    //     TreeRef<Tree>,
    //     _,
    //     bfs_wrapper::SD<_, _, CompletePostOrder<_, IdD>>,
    //     NS<Tree>,
    // >::compute_actions(
    //     &node_store,
    //     &src_arena,
    //     &dst_arena,
    //     &ms,
    // );

    let actions = script_generator::ScriptGenerator::<
        _,
        TreeRef<Tree>,
        _,
        SimpleBfsMapper<_, _, CompletePostOrder<_, IdD>, _>,
        _,
    >::compute_actions(&node_store, &src_arena, &dst_arena, &ms);

    // new Delete(src.getChild("1.1"))
    assert!(actions.has_actions(&[SimpleAction::Delete {
        tree: from_src(&[1, 1]),
    },]));

    assert!(actions.has_actions(&[
        // new Insert(dst, null, 0),
        SimpleAction::Insert {
            sub: dst_arena.original(&dst),
            parent: None,
            idx: 0,
        },
    ]));
    assert!(actions.has_actions(&[
        // new Move(src, dst, 0),
        SimpleAction::Move {
            sub: *src,
            parent: Some(*dst),
            idx: 0,
        },
    ]));
    assert!(actions.has_actions(&[
        // new Update(src.getChild("1.3"), "r2"),
        SimpleAction::Update {
            src: from_src(&[1, 3]),
            dst: from_dst(&[0, 1, 3]),
            old: *node_store
                .resolve(&src_arena.original(&from_src(&[1, 3])))
                .get_label(),
            new: *node_store
                .resolve(&dst_arena.original(&from_dst(&[0, 1, 3])))
                .get_label(),
        }, // label: "r2".to_owned()},
    ]));
    assert!(actions.has_actions(&[
        // new Insert(dst.getChild("0.1.1"), src.getChild("1"), 1),
        SimpleAction::Insert {
            sub: dst_arena.original(&from_dst(&[0, 1, 1])),
            parent: Some(from_dst(&[0, 1])),
            idx: 1,
        },
    ]));
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
}
