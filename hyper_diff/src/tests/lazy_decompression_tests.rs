use hyper_ast::types::DecompressedSubtree;

use crate::{
    decompressed_tree_store::{
        lazy_post_order::LazyPostOrder, LazyDecompressedTreeStore,
        ShallowDecompressedTreeStore,
    },
    tests,
    tree::simple_tree::{vpair_to_stores, DisplayTree},
};

#[test]
fn test() {
    let (label_store, node_store, src, dst) =
        vpair_to_stores(tests::examples::example_gt_java_code());
    // assert_eq!(label_store.resolve(&0).to_owned(), b"");
    println!(
        "src tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, src)
    );
    println!(
        "dst tree:\n{:?}",
        DisplayTree::new(&label_store, &node_store, dst)
    );

    let mut src_arena = LazyPostOrder::<_, u16>::decompress(&node_store, &src);
    let mut dst_arena = LazyPostOrder::<_, u16>::decompress(&node_store, &dst);

    dbg!(src_arena.root());
    dbg!(dst_arena.root());
    use crate::decompressed_tree_store::PostOrder;
    use hyper_ast::types::NodeStore;
    use hyper_ast::types::WithStats;
    dbg!(src_arena.tree(&src_arena.root()));
    dbg!(dst_arena.tree(&dst_arena.root()));
    src_arena
        .decompress_children(&node_store, &src_arena.root())
        .len();
    dst_arena
        .decompress_children(&node_store, &dst_arena.root())
        .len();
    dbg!(node_store
        .resolve(&src_arena.tree(&src_arena.root()))
        .size());
    dbg!(node_store
        .resolve(&dst_arena.tree(&dst_arena.root()))
        .size());

    let _src_arena = src_arena.complete(&node_store);
    let _dst_arena = dst_arena.complete(&node_store);
}
