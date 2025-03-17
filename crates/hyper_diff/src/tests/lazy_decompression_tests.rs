use hyperast::types::DecompressedFrom;

use crate::{
    decompressed_tree_store::{
        lazy_post_order::LazyPostOrder, LazyDecompressedTreeStore, ShallowDecompressedTreeStore,
    },
    matchers::Decompressible,
    tests,
    tree::simple_tree::{vpair_to_stores, DisplayTree},
};

#[test]
fn test() {
    let (stores, src, dst) = vpair_to_stores(tests::examples::example_gt_java_code());
    // assert_eq!(label_store.resolve(&0).to_owned(), b"");
    let node_store = &stores.node_store;
    let label_store = &stores.label_store;
    println!(
        "src tree:\n{:?}",
        DisplayTree::new(label_store, node_store, src)
    );
    println!(
        "dst tree:\n{:?}",
        DisplayTree::new(label_store, node_store, dst)
    );

    let mut src_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &src);
    let mut dst_arena = Decompressible::<_, LazyPostOrder<_, u16>>::decompress(&stores, &dst);
    let mut src_arena = src_arena.as_mut();
    let mut dst_arena = dst_arena.as_mut();
    dbg!(src_arena.root());
    dbg!(dst_arena.root());
    use crate::decompressed_tree_store::PostOrder;
    dbg!(src_arena.tree(&src_arena.root()));
    dbg!(dst_arena.tree(&dst_arena.root()));
    src_arena.decompress_children(&src_arena.root()).len();
    dst_arena.decompress_children(&dst_arena.root()).len();
    use hyperast::types::NodeStore;
    use hyperast::types::WithStats;
    dbg!(node_store
        .resolve(&src_arena.tree(&src_arena.root()))
        .size());
    dbg!(node_store
        .resolve(&dst_arena.tree(&dst_arena.root()))
        .size());

    src_arena.complete_subtree(&src_arena.root());
    dst_arena.complete_subtree(&dst_arena.root());
}
