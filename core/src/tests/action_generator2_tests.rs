use num_traits::PrimInt;

use crate::{
    actions::{
        bfs_wrapper,
        script_generator2::{Act, Actions, ScriptGenerator, SimpleAction, TestActions},
    },
    matchers::{
        decompressed_tree_store::{CompletePostOrder, Initializable, ShallowDecompressedTreeStore},
        mapping_store::{DefaultMappingStore, MappingStore},
    },
    tests::{
        examples::{example_action, example_gt_java_code},
        simple_tree::{vpair_to_stores, Tree, NS},
    },
    tree::tree::{LabelStore, Labeled, NodeStore, Stored, WithChildren},
};
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

    println!("{:?}", Fmt(|f| { node_store.fmt(f, &label_store) }));

    println!(
        "#src\n{:?}",
        Fmt(|f| {
            let a = |x: &u16| -> String {
                let n = node_store.resolve(x);
                let x = &n.get_label();
                std::str::from_utf8(&label_store.resolve(x))
                    .unwrap()
                    .to_owned()
            };
            src_arena.fmt(f, a)
        })
    );

    println!(
        "#dst\n{:?}",
        Fmt(|f| {
            let a = |x: &u16| -> String {
                let n = node_store.resolve(x);
                let x = &n.get_label();
                std::str::from_utf8(&label_store.resolve(x))
                    .unwrap()
                    .to_owned()
            };
            dst_arena.fmt(f, a)
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

    let lab = |x: &IdD| {
        std::str::from_utf8(&label_store.resolve(&node_store.resolve(x).get_label()))
            .unwrap()
            .to_owned()
    };

    println!("{:?}", actions);

    let a = make_update(
        *node_store
            .resolve(&dst_arena.original(&from_dst(&[])))
            .get_label(),
        &[],
    ); // root renamed

    assert!(actions.has_actions(&[a,]));

    let a = make_insert(
        dst_arena.original(&from_dst(&[1])),
        &[2], /* FIXME should be 1? */
    ); // h at a.2
    println!("{:?}", a);
    assert!(actions.has_actions(&[a]));

    let a = make_insert(
        dst_arena.original(&from_dst(&[2])),
        &[3], /* FIXME should be 2? */
    ); // x at a.3
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    let a = make_move(&[0], &[2, 0]); // e to h.0
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    // ins u at j.0
    let a = make_insert(dst_arena.original(&from_dst(&[3, 0])), &[5, 0]);
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    // upd f to y
    let a = make_update(
        *node_store
            .resolve(&dst_arena.original(&from_dst(&[1, 0, 0])))
            .get_label(),
        &[0, 0],
    );
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    // ins u at v.0
    let a = make_insert(dst_arena.original(&from_dst(&[3, 0, 0])), &[5, 0, 0]);
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    // mov k to v.0
    let a = make_move(&[4, 0], &[5, 0, 0, 0]);
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    // del g
    let a = make_delete(&[2]);
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    // del i
    let a = make_delete(&[3]);
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    assert_eq!(12, actions.len()); // FIXME should be 9 if actions are compressed
}

fn make_move<T: Stored + Labeled + WithChildren>(
    from: &[T::ChildIdx],
    to: &[T::ChildIdx],
) -> SimpleAction<T> {
    SimpleAction {
        path: to.into(),
        action: Act::Move { from: from.into() },
    }
}

fn make_delete<T: Stored + Labeled + WithChildren>(path: &[T::ChildIdx]) -> SimpleAction<T> {
    SimpleAction {
        path: path.into(),
        action: Act::Delete {},
    }
}

fn make_insert<T: Stored + Labeled + WithChildren>(
    sub: T::TreeId,
    path: &[T::ChildIdx],
) -> SimpleAction<T> {
    SimpleAction {
        path: path.into(),
        action: Act::Insert { sub },
    }
}

fn make_update<T: Stored + Labeled + WithChildren>(
    new: T::Label,
    path: &[T::ChildIdx],
) -> SimpleAction<T> {
    SimpleAction {
        path: path.into(),
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
type IdD = u16;
#[test]
fn test_with_zs_custom_example() {
    let (_, node_store, src, dst) = vpair_to_stores(example_gt_java_code());
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

    println!("{:?}", actions);

    // // new Delete(src.getChild("1.1"))
    // assert!(actions.has_actions(&[SimpleAction::Delete {
    //     tree: from_src(&[1, 1]),
    // },]));
    let a = make_delete(&[1, 1]);
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    // assert!(actions.has_actions(&[
    //     // new Insert(dst, null, 0),
    //     SimpleAction::Insert {
    //         sub: dst_arena.original(&dst),
    //         parent: None,
    //         idx: 0,
    //     },
    // ]));
    let a = make_insert(dst_arena.original(&dst), &[]);
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));
    // assert!(actions.has_actions(&[
    //     // new Move(src, dst, 0),
    //     SimpleAction::Move {
    //         sub: *src,
    //         parent: Some(*dst),
    //         idx: 0,
    //     },
    // ]));
    let a = make_move(&[], &[0]);
    println!("{:?}", a);
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
        &[],
    );
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));
    // assert!(actions.has_actions(&[
    //     // new Insert(dst.getChild("0.1.1"), src.getChild("1"), 1),
    //     SimpleAction::Insert {
    //         sub: dst_arena.original(&from_dst(&[0, 1, 1])),
    //         parent: Some(from_dst(&[0, 1])),
    //         idx: 1,
    //     },
    // ]));
    let a = make_insert(dst_arena.original(&from_dst(&[0, 1, 1])), &[0, 1, 1]);
    println!("{:?}", a);
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
}
