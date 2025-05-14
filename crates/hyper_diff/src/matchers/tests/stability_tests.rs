use hyperast::{
    store::SimpleStores,
    test_utils::simple_tree::{DisplayTree, TStore, LS},
    types::DecompressedFrom,
};

use crate::{
    decompressed_tree_store::{CompletePostOrder, ShallowDecompressedTreeStore},
    matchers::{
        heuristic::gt::{
            bottom_up_matcher::BottomUpMatcher, greedy_bottom_up_matcher::GreedyBottomUpMatcher,
        },
        mapping_store::{DefaultMappingStore, MappingStore, VecStore},
        Decompressible, Mapper, Mapping,
    },
    tests::examples::example_unstable,
    tree::simple_tree::{vpair_to_stores, Tree, NS},
};

#[test]
fn test_stability() {
    let (stores, src, dst) = vpair_to_stores(example_unstable());

    let src_arena = Decompressible::<_, CompletePostOrder<_, u16>>::decompress(&stores, &src);
    let dst_arena = Decompressible::<_, CompletePostOrder<_, u16>>::decompress(&stores, &dst);

    let mut mappings: DefaultMappingStore<u16> = DefaultMappingStore::default();
    mappings.topit(src_arena.len(), dst_arena.len());

    let label_store = &stores.label_store;
    let node_store = &stores.node_store;
    println!(
        "src tree:\n{:?}",
        DisplayTree::new(label_store, node_store, src)
    );
    println!(
        "dst tree:\n{:?}",
        DisplayTree::new(label_store, node_store, dst)
    );

    let src = src_arena.root();
    let dst = dst_arena.root();

    let from_src = |path: &[u8]| src_arena.child(&src, path);
    let from_dst = |path: &[u8]| dst_arena.child(&dst, path);

    mappings.link(from_src(&[0, 0]), from_dst(&[1, 1]));
    mappings.link(from_src(&[0, 1]), from_dst(&[0, 1]));
    mappings.link(from_src(&[1, 0]), from_dst(&[0, 0]));
    mappings.link(from_src(&[1, 1]), from_dst(&[1, 0]));
    dbg!(&mappings.src_to_dst);
    dbg!(&mappings.dst_to_src);

    // let mapping = Mapper {
    // hyperast: &stores,
    // mapping: Mapping {
    // src_arena,
    // dst_arena,
    // mappings,
    // },
    // };
    // let result = GreedyBottomUpMatcher::<
    // Decompressible<_, CompletePostOrder<_, u16>>,
    // Decompressible<_, CompletePostOrder<_, u16>>,
    // &SimpleStores<TStore, NS<Tree>, LS<u16>>,
    // VecStore<u16>,
    // >::match_it(mapping);
    let mapper = GreedyBottomUpMatcher::<
        Decompressible<_, CompletePostOrder<_, u16>>,
        Decompressible<_, CompletePostOrder<_, u16>>,
        &SimpleStores<TStore, NS<Tree>, LS<u16>>,
        VecStore<u16>,
    >::matchh(&stores, &src, &dst, mappings.clone());
    let BottomUpMatcher::<_, _, _, _> {
        src_arena,
        dst_arena,
        mappings,
        ..
    }: BottomUpMatcher<_, _, _, _> = mapper.into();
    let src = src_arena.root();
    let dst = dst_arena.root();

    // // assertEquals(5, ms1.size());
    assert_eq!(5, mappings.src_to_dst.iter().filter(|x| **x != 0).count());
    assert_eq!(5, mappings.len());
    assert_eq!(mappings.src_to_dst, mappings.src_to_dst);
    assert!(mappings.has(&src, &dst));
}
