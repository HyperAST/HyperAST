use num_traits::zero;

use crate::{
    actions::{
        bfs_wrapper,
        script_generator::{self, Actions, SimpleAction, TestActions},
    },
    matchers::{
        decompressed_tree_store::{
            CompletePostOrder, DecompressedTreeStore, Initializable, ShallowDecompressedTreeStore,
        },
        mapping_store::{DefaultMappingStore, MappingStore},
    },
    tests::{
        examples::{example_action, example_gt_java_code},
        simple_tree::{vpair_to_stores, Tree, NS},
    },
    tree::tree::{LabelStore, Labeled, NodeStore},
};

#[test]
fn testWithActionExample() {
    let (label_store, node_store, src, dst) = vpair_to_stores(example_action());
    let mut ms = DefaultMappingStore::new();
    let src_arena = CompletePostOrder::<_, u16>::new(&node_store, &src);
    let dst_arena = CompletePostOrder::<_, u16>::new(&node_store, &dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len() + 1, dst_arena.len() + 1);
    ms.link(
        src_arena.child(&node_store, src, &[]),
        dst_arena.child(&node_store, dst, &[]),
    );
    ms.link(
        src_arena.child(&node_store, src, &[1]),
        dst_arena.child(&node_store, dst, &[0]),
    );
    ms.link(
        src_arena.child(&node_store, src, &[1, 0]),
        dst_arena.child(&node_store, dst, &[0, 0]),
    );
    ms.link(
        src_arena.child(&node_store, src, &[1, 1]),
        dst_arena.child(&node_store, dst, &[0, 1]),
    );
    ms.link(
        src_arena.child(&node_store, src, &[0]),
        dst_arena.child(&node_store, dst, &[1, 0]),
    );
    ms.link(
        src_arena.child(&node_store, src, &[0, 0]),
        dst_arena.child(&node_store, dst, &[1, 0, 0]),
    );
    ms.link(
        src_arena.child(&node_store, src, &[4]),
        dst_arena.child(&node_store, dst, &[3]),
    );
    ms.link(
        src_arena.child(&node_store, src, &[4, 0]),
        dst_arena.child(&node_store, dst, &[3, 0, 0, 0]),
    );
    todo!();
    // EditScript actions = new SimplifiedChawatheScriptGenerator().computeActions(ms);
    // assertEquals(9, actions.size());

    // Action a = actions.get(0);
    // assertTrue(a instanceof Insert);
    // Insert i = (Insert) a;
    // assertEquals("h", i.getNode().getLabel());
    // assertEquals("a", i.getParent().getLabel());
    // assertEquals(2, i.getPosition());

    // a = actions.get(1);
    // assertTrue(a instanceof TreeInsert);
    // TreeInsert ti = (TreeInsert) a;
    // assertEquals("x", ti.getNode().getLabel());
    // assertEquals("a", ti.getParent().getLabel());
    // assertEquals(3, ti.getPosition());

    // a = actions.get(2);
    // assertTrue(a instanceof Move);
    // Move m = (Move) a;
    // assertEquals("e", m.getNode().getLabel());
    // assertEquals("h", m.getParent().getLabel());
    // assertEquals(0, m.getPosition());

    // a = actions.get(3);
    // assertTrue(a instanceof Insert);
    // Insert i2 = (Insert) a;
    // assertEquals("u", i2.getNode().getLabel());
    // assertEquals("j", i2.getParent().getLabel());
    // assertEquals(0, i2.getPosition());

    // a = actions.get(4);
    // assertTrue(a instanceof Update);
    // Update u = (Update) a;
    // assertEquals("f", u.getNode().getLabel());
    // assertEquals("y", u.getValue());

    // a = actions.get(5);
    // assertTrue(a instanceof Insert);
    // Insert i3 = (Insert) a;
    // assertEquals("v", i3.getNode().getLabel());
    // assertEquals("u", i3.getParent().getLabel());
    // assertEquals(0, i3.getPosition());

    // a = actions.get(6);
    // assertTrue(a instanceof Move);
    // Move m2 = (Move) a;
    // assertEquals("k", m2.getNode().getLabel());
    // assertEquals("v", m2.getParent().getLabel());
    // assertEquals(0, m.getPosition());

    // a = actions.get(7);
    // assertTrue(a instanceof TreeDelete);
    // TreeDelete td = (TreeDelete) a;
    // assertEquals("g", td.getNode().getLabel());

    // a = actions.get(8);
    // assertTrue(a instanceof Delete);
    // Delete d = (Delete) a;
    // assertEquals("i", d.getNode().getLabel());
}

#[test]
fn testWithUnmappedRoot() {
    todo!()
    // ITree src = new Tree(TypeSet.type("foo"), "");
    // ITree dst = new Tree(TypeSet.type("bar"), "");
    // MappingStore ms = new MappingStore(src, dst);
    // EditScript actions = new SimplifiedChawatheScriptGenerator().computeActions(ms);
    // for (Action a : actions)
    //     System.out.println(a.toString());
}

#[test]
fn testWithActionExampleNoMove() {
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
fn testWithZsCustomExample() {
    let (label_store, node_store, src, dst) = vpair_to_stores(example_gt_java_code());
    let mut ms = DefaultMappingStore::new();
    let src_arena = CompletePostOrder::<_, IdD>::new(&node_store, &src);
    let dst_arena = CompletePostOrder::<_, IdD>::new(&node_store, &dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len() + 1, dst_arena.len() + 1);
    // ms.addMapping(src, dst.getChild(0));
    ms.link(
        src_arena.child(&node_store, src, &[]),
        dst_arena.child(&node_store, dst, &[0]),
    );
    // ms.addMapping(src.getChild(0), dst.getChild("0.0"));
    ms.link(
        src_arena.child(&node_store, src, &[0]),
        dst_arena.child(&node_store, dst, &[0, 0]),
    );
    // ms.addMapping(src.getChild(1), dst.getChild("0.1"));
    ms.link(
        src_arena.child(&node_store, src, &[1]),
        dst_arena.child(&node_store, dst, &[0, 1]),
    );
    // ms.addMapping(src.getChild("1.0"), dst.getChild("0.1.0"));
    ms.link(
        src_arena.child(&node_store, src, &[1, 0]),
        dst_arena.child(&node_store, dst, &[0, 1, 0]),
    );
    // ms.addMapping(src.getChild("1.2"), dst.getChild("0.1.2"));
    ms.link(
        src_arena.child(&node_store, src, &[1, 2]),
        dst_arena.child(&node_store, dst, &[0, 1, 2]),
    );
    // ms.addMapping(src.getChild("1.3"), dst.getChild("0.1.3"));
    ms.link(
        src_arena.child(&node_store, src, &[1, 3]),
        dst_arena.child(&node_store, dst, &[0, 1, 3]),
    );

    let actions = script_generator::ScriptGenerator::<
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

    assert_eq!(5, actions.len());
    assert!(actions.has_items(&[
        // new Insert(dst, null, 0),
        SimpleAction::Insert {
            sub: todo!(),
            parent: todo!(),
            idx: 0,
        },
        // new Move(src, dst, 0),
        SimpleAction::Move {
            sub: *src,
            parent: Some(*dst),
            idx: 0,
        },
        // new Insert(dst.getChild("0.1.1"), src.getChild("1"), 1),
        SimpleAction::Insert {
            sub: todo!(),
            parent: Some(src_arena.child(&node_store, src, &[0, 1, 1])),
            idx: 1,
        },
        // new Update(src.getChild("1.3"), "r2"),
        SimpleAction::Update {
            src: src_arena.child(&node_store, src, &[1, 3]),
            dst: dst_arena.child(&node_store, dst, &[0, 1, 3]),
            old: node_store
                .get_node_at_id(&src_arena.child(&node_store, src, &[0, 3]))
                .get_label(),
            new: node_store
                .get_node_at_id(&dst_arena.child(&node_store, dst, &[0, 1, 3]))
                .get_label(),
        }, // label: "r2".to_owned()},
        // new Delete(src.getChild("1.1"))
        SimpleAction::Delete {
            tree: src_arena.child(&node_store, src, &[1, 1]),
        },
    ]));

    assert_eq!(
        label_store
            .get_node_at_id(
                &node_store
                    .get_node_at_id(&dst_arena.child(&node_store, dst, &[0, 1, 3]))
                    .get_label()
            )
            .to_owned(),
        b"r2"
    );
    todo!()

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
