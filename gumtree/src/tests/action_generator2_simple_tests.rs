use crate::actions;
use crate::actions::action_vec::ActionsVec;
use crate::decompressed_tree_store::bfs_wrapper::SimpleBfsMapper;
use crate::tree::simple_tree::Tree;
use crate::tree::tree_path::CompressedTreePath;
use crate::{
    actions::{
        action_vec::{apply_actions, TestActions},
        script_generator2::ScriptGenerator,
        Actions,
    },
    decompressed_tree_store::{CompletePostOrder, ShallowDecompressedTreeStore},
    matchers::mapping_store::{DefaultMappingStore, MappingStore},
    tests::{
        action_generator2_tests::{make_delete, make_insert, make_move, make_update, Fmt},
        simple_examples::{example_delete_action, example_move_action, example_rename_action},
    },
    tree::simple_tree::{vpair_to_stores, DisplayTree, TreeRef, NS},
};
use hyper_ast::types::{LabelStore, Labeled, NodeStore, DecompressedSubtree};

type IdD = u16;

#[test]
fn test_no_actions() {
    let (label_store, node_store, s_src, s_dst) =
        vpair_to_stores((example_delete_action().0, example_delete_action().0));
    println!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, s_src)
    );
    println!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, s_dst)
    );
    let mut ms = DefaultMappingStore::default();
    let src_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &s_src);
    let dst_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &s_dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len(), dst_arena.len());
    let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
    let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
    ms.link(from_src(&[]), from_dst(&[]));
    ms.link(from_src(&[0]), from_dst(&[0]));
    ms.link(from_src(&[0, 0]), from_dst(&[0, 0]));
    ms.link(from_src(&[1]), from_dst(&[1]));
    ms.link(from_src(&[1, 0]), from_dst(&[1, 0]));
    ms.link(from_src(&[1, 1]), from_dst(&[1, 1]));

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
    let actions: ActionsVec<_> = ScriptGenerator::<
        _,
        TreeRef<Tree>,
        _,
        SimpleBfsMapper<_, _, CompletePostOrder<_, IdD>, _>,
        NS<Tree>,
        _,
        _,
    >::_compute_actions(&node_store, &src_arena, &dst_arena, &ms).unwrap();

    let mut node_store = node_store;
    let mut root = vec![s_src];
    apply_actions::<_, NS<Tree>, CompressedTreePath<_>>(actions, &mut root, &mut node_store);
    let then = *root.last().unwrap();

    println!(
        "then tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, then)
    );

    assert_eq!(then, s_dst);
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
    let mut ms = DefaultMappingStore::default();
    let src_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &s_src);
    let dst_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &s_dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len(), dst_arena.len());
    let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
    let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
    ms.link(from_src(&[]), from_dst(&[]));
    ms.link(from_src(&[0]), from_dst(&[0]));
    ms.link(from_src(&[1]), from_dst(&[1]));
    ms.link(from_src(&[1, 0]), from_dst(&[1, 0]));
    ms.link(from_src(&[1, 1]), from_dst(&[1, 1]));

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
    let actions = ScriptGenerator::<
        _,
        TreeRef<Tree>,
        _,
        SimpleBfsMapper<_, _, CompletePostOrder<_, IdD>, _>,
        NS<Tree>,
        _,
        _,
    >::_compute_actions(&node_store, &src_arena, &dst_arena, &ms).unwrap();

    println!("{:?}", actions);

    // del f
    let a = make_delete::<Tree, CompressedTreePath<_>>((&[0, 0], &[0, 0]));
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    assert_eq!(1, actions.len());

    let mut node_store = node_store;
    let mut root = vec![s_src];
    apply_actions::<_, NS<Tree>, _>(actions, &mut root, &mut node_store);
    let then = *root.last().unwrap();

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
    let mut ms = DefaultMappingStore::default();
    let src_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &s_src);
    let dst_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &s_dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len(), dst_arena.len());
    let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
    let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
    ms.link(from_src(&[]), from_dst(&[]));
    ms.link(from_src(&[0]), from_dst(&[0]));
    ms.link(from_src(&[1]), from_dst(&[1]));
    ms.link(from_src(&[1, 0]), from_dst(&[1, 0]));
    ms.link(from_src(&[1, 1]), from_dst(&[1, 1]));

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
    let actions = ScriptGenerator::<
        _,
        TreeRef<Tree>,
        _,
        SimpleBfsMapper<_, _, CompletePostOrder<_, IdD>, _>,
        NS<Tree>,
        _,
        _,
    >::_compute_actions(&node_store, &src_arena, &dst_arena, &ms).unwrap();

    println!("{:?}", actions);

    // ins f
    let a = make_insert::<Tree, CompressedTreePath<_>>(
        dst_arena.original(&from_dst(&[0, 0])),
        (&[0, 0], &[0, 0]),
    );
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    assert_eq!(1, actions.len());

    let mut node_store = node_store;
    let mut root = vec![s_src];
    apply_actions::<_, NS<Tree>, _>(actions, &mut root, &mut node_store);
    let then = *root.last().unwrap();

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
    let mut ms = DefaultMappingStore::default();
    let src_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &s_src);
    let dst_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &s_dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len(), dst_arena.len());
    let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
    let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
    ms.link(from_src(&[]), from_dst(&[]));
    ms.link(from_src(&[0]), from_dst(&[0]));
    ms.link(from_src(&[0, 0]), from_dst(&[0, 0]));
    ms.link(from_src(&[1]), from_dst(&[1]));
    ms.link(from_src(&[1, 0]), from_dst(&[1, 0]));
    ms.link(from_src(&[1, 1]), from_dst(&[1, 1]));

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
    let actions = ScriptGenerator::<
        _,
        TreeRef<Tree>,
        _,
        SimpleBfsMapper<_, _, CompletePostOrder<_, IdD>, _>,
        NS<Tree>,
        _,
        _,
    >::_compute_actions(&node_store, &src_arena, &dst_arena, &ms).unwrap();

    println!("{:?}", actions);

    // upd f
    let a = make_update::<Tree, CompressedTreePath<_>>(
        *node_store
            .resolve(&dst_arena.original(&from_dst(&[0, 0])))
            .get_label(),
        (&[0, 0], &[0, 0]),
    );
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    assert_eq!(1, actions.len());

    let mut node_store = node_store;
    let mut root = vec![s_src];
    apply_actions::<_, NS<Tree>, _>(actions, &mut root, &mut node_store);
    let then = *root.last().unwrap();

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
    let mut ms = DefaultMappingStore::default();
    let src_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &s_src);
    let dst_arena = CompletePostOrder::<_, u16>::decompress(&node_store, &s_dst);
    let src = &(src_arena.root());
    let dst = &(dst_arena.root());
    ms.topit(src_arena.len(), dst_arena.len());
    let from_src = |path: &[u8]| src_arena.child(&node_store, src, path);
    let from_dst = |path: &[u8]| dst_arena.child(&node_store, dst, path);
    ms.link(from_src(&[]), from_dst(&[]));
    ms.link(from_src(&[0]), from_dst(&[0]));
    ms.link(from_src(&[0, 0]), from_dst(&[1, 1]));
    ms.link(from_src(&[1]), from_dst(&[1]));
    ms.link(from_src(&[1, 0]), from_dst(&[1, 0]));
    ms.link(from_src(&[1, 1]), from_dst(&[1, 2]));

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
    let actions = ScriptGenerator::<
        _,
        TreeRef<Tree>,
        _,
        SimpleBfsMapper<_, _, CompletePostOrder<_, IdD>, _>,
        NS<Tree>,
        _,
        _,
    >::_compute_actions(&node_store, &src_arena, &dst_arena, &ms).unwrap();

    println!("{:?}", actions);

    // move f to b.1
    let a = make_move::<Tree, CompressedTreePath<_>>((&[0, 0], &[0, 0]), (&[1, 1], &[1, 1]));
    println!("{:?}", a);
    assert!(actions.has_actions(&[a,]));

    assert_eq!(1, actions.len());

    let mut node_store = node_store;
    let mut root = vec![s_src];
    apply_actions::<_, NS<Tree>, _>(actions, &mut root, &mut node_store);
    let then = *root.last().unwrap();

    println!(
        "then tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, then)
    );

    assert_eq!(then, s_dst);
}
