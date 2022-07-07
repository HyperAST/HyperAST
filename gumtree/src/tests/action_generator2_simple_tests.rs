use crate::{
    actions::{
        action_vec::{ActionsVec, ApplicableActions, TestActions},
        bfs_wrapper,
        script_generator2::{Act, ScriptGenerator, SimpleAction},
        Actions,
    },
    matchers::{
        decompressed_tree_store::{CompletePostOrder, Initializable, ShallowDecompressedTreeStore},
        mapping_store::{DefaultMappingStore, MappingStore},
    },
    tests::{
        action_generator2_tests::{make_delete, make_insert, make_move, make_update, Fmt},
        examples::{example_action, example_gt_java_code},
        simple_examples::{example_delete_action, example_move_action, example_rename_action},
    },
    tree::{
        simple_tree::{vpair_to_stores, DisplayTree, Tree, NS},
        tree::{LabelStore, Labeled, NodeStore, Stored, WithChildren},
    },
};
use std::fmt;

type IdD = u16;

#[test]
fn test_no_actions() {
    let (label_store, node_store, src, dst) =
        vpair_to_stores((example_delete_action().0, example_delete_action().0));
    println!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, src)
    );
    println!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, dst)
    );
    let mut ms = DefaultMappingStore::new();
    let src_arena = CompletePostOrder::<_, u16>::new(&node_store, &src);
    let dst_arena = CompletePostOrder::<_, u16>::new(&node_store, &dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len() + 1, dst_arena.len() + 1);
    let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
    let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
    ms.link(from_src(&[]), from_dst(&[]));
    ms.link(from_src(&[0]), from_dst(&[0]));
    ms.link(from_src(&[0, 0]), from_dst(&[0, 0]));
    ms.link(from_src(&[1]), from_dst(&[1]));
    ms.link(from_src(&[1, 0]), from_dst(&[1, 0]));
    ms.link(from_src(&[1, 1]), from_dst(&[1, 1]));

    let g = |x: &u16| -> String {
        let x = node_store.resolve(x).get_label();
        std::str::from_utf8(&label_store.resolve(x))
            .unwrap()
            .to_owned()
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
    assert_eq!(0, actions.len());

    let mut node_store = node_store;
    let then = ActionsVec::apply_actions(actions.iter(), *src, &mut node_store);
    assert_eq!(then, *dst);
}

#[test]
fn test_delete_actions_1() {
    let (label_store, node_store, s_src, s_dst) = vpair_to_stores(example_delete_action());
    println!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, s_src)
    );
    println!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, s_dst)
    );
    let mut ms = DefaultMappingStore::new();
    let src_arena = CompletePostOrder::<_, u16>::new(&node_store, &s_src);
    let dst_arena = CompletePostOrder::<_, u16>::new(&node_store, &s_dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len() + 1, dst_arena.len() + 1);
    let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
    let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
    ms.link(from_src(&[]), from_dst(&[]));
    ms.link(from_src(&[0]), from_dst(&[0]));
    ms.link(from_src(&[1]), from_dst(&[1]));
    ms.link(from_src(&[1, 0]), from_dst(&[1, 0]));
    ms.link(from_src(&[1, 1]), from_dst(&[1, 1]));

    let g = |x: &u16| -> String {
        let x = node_store.resolve(x).get_label();
        std::str::from_utf8(&label_store.resolve(x))
            .unwrap()
            .to_owned()
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

    // del f
    let a = make_delete((&[0, 0], &[0, 0]));
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    assert_eq!(1, actions.len());

    let mut node_store = node_store;
    let then = ActionsVec::apply_actions(actions.iter(), s_src, &mut node_store);

    println!(
        "then tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, then)
    );

    assert_eq!(then, s_dst);
}

#[test]
fn test_insert_actions_1() {
    let (label_store, node_store, s_src, s_dst) =
        vpair_to_stores((example_delete_action().1, example_delete_action().0));
    println!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, s_src)
    );
    println!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, s_dst)
    );
    let mut ms = DefaultMappingStore::new();
    let src_arena = CompletePostOrder::<_, u16>::new(&node_store, &s_src);
    let dst_arena = CompletePostOrder::<_, u16>::new(&node_store, &s_dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len() + 1, dst_arena.len() + 1);
    let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
    let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
    ms.link(from_src(&[]), from_dst(&[]));
    ms.link(from_src(&[0]), from_dst(&[0]));
    ms.link(from_src(&[1]), from_dst(&[1]));
    ms.link(from_src(&[1, 0]), from_dst(&[1, 0]));
    ms.link(from_src(&[1, 1]), from_dst(&[1, 1]));

    let g = |x: &u16| -> String {
        let x = node_store.resolve(x).get_label();
        std::str::from_utf8(&label_store.resolve(x))
            .unwrap()
            .to_owned()
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

    // ins f
    let a = make_insert(dst_arena.original(&from_dst(&[0, 0])), (&[0, 0], &[0, 0]));
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    assert_eq!(1, actions.len());

    let mut node_store = node_store;
    let then = ActionsVec::apply_actions(actions.iter(), s_src, &mut node_store);

    println!(
        "then tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, then)
    );

    assert_eq!(then, s_dst);
}

#[test]
fn test_rename_actions_1() {
    let (label_store, node_store, s_src, s_dst) = vpair_to_stores(example_rename_action());
    println!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, s_src)
    );
    println!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, s_dst)
    );
    let mut ms = DefaultMappingStore::new();
    let src_arena = CompletePostOrder::<_, u16>::new(&node_store, &s_src);
    let dst_arena = CompletePostOrder::<_, u16>::new(&node_store, &s_dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len() + 1, dst_arena.len() + 1);
    let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
    let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
    ms.link(from_src(&[]), from_dst(&[]));
    ms.link(from_src(&[0]), from_dst(&[0]));
    ms.link(from_src(&[0, 0]), from_dst(&[0, 0]));
    ms.link(from_src(&[1]), from_dst(&[1]));
    ms.link(from_src(&[1, 0]), from_dst(&[1, 0]));
    ms.link(from_src(&[1, 1]), from_dst(&[1, 1]));

    let g = |x: &u16| -> String {
        let x = node_store.resolve(x).get_label();
        std::str::from_utf8(&label_store.resolve(x))
            .unwrap()
            .to_owned()
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

    // upd f
    let a = make_update(
        *node_store
            .resolve(&dst_arena.original(&from_dst(&[0, 0])))
            .get_label(),
        (&[0, 0], &[0, 0]),
    );
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    assert_eq!(1, actions.len());

    let mut node_store = node_store;
    let then = ActionsVec::apply_actions(actions.iter(), s_src, &mut node_store);

    println!(
        "then tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, then)
    );

    assert_eq!(then, s_dst);
}

#[test]
fn test_move_actions_1() {
    let (label_store, node_store, s_src, s_dst) = vpair_to_stores(example_move_action());
    println!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, s_src)
    );
    println!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, s_dst)
    );
    let mut ms = DefaultMappingStore::new();
    let src_arena = CompletePostOrder::<_, u16>::new(&node_store, &s_src);
    let dst_arena = CompletePostOrder::<_, u16>::new(&node_store, &s_dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len() + 1, dst_arena.len() + 1);
    let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
    let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
    ms.link(from_src(&[]), from_dst(&[]));
    ms.link(from_src(&[0]), from_dst(&[0]));
    ms.link(from_src(&[0, 0]), from_dst(&[1, 1]));
    ms.link(from_src(&[1]), from_dst(&[1]));
    ms.link(from_src(&[1, 0]), from_dst(&[1, 0]));
    ms.link(from_src(&[1, 1]), from_dst(&[1, 2]));

    let g = |x: &u16| -> String {
        let x = node_store.resolve(x).get_label();
        std::str::from_utf8(&label_store.resolve(x))
            .unwrap()
            .to_owned()
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

    // move f to b.1
    let a = make_move((&[0, 0], &[0, 0]), (&[1, 1], &[1, 1]));
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    assert_eq!(1, actions.len());

    let mut node_store = node_store;
    let then = ActionsVec::apply_actions(actions.iter(), s_src, &mut node_store);

    println!(
        "then tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, then)
    );

    assert_eq!(then, s_dst);
}
