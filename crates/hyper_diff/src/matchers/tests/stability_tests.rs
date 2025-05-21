use hyperast::{
    store::SimpleStores,
    test_utils::simple_tree::{DisplayTree, LS, SimpleTree, TStore},
    types::DecompressedFrom,
};

use crate::{
    decompressed_tree_store::{CompletePostOrder, ShallowDecompressedTreeStore},
    matchers::{
        Decompressible, Mapper, Mapping,
        heuristic::gt::{
            bottom_up_matcher::BottomUpMatcher, greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            simple_marriage_bottom_up_matcher::SimpleMarriageBottomUpMatcher,
        },
        mapping_store::{DefaultMappingStore, MappingStore, VecStore},
    },
    tests::examples::*,
    tree::simple_tree::{NS, Tree, vpair_to_stores},
};

#[test]
fn test_gumtree_stable() {
    test_stable(example_stable_test1());
    test_stable(example_stable_test2());
    test_stable(example_stable1());
    test_stable(example_stable2());
    test_stable(example_stable3());
    test_stable(example_unstable1());
    test_stable(example_unstable2());
}

#[test]
fn test_gumtree_greedy_unstable() {
    test_unstable(example_unstable1());
    test_unstable(example_unstable2());
}

// This test will only succeed on examples specifically made to be unstable
fn test_unstable(example: ((SimpleTree<u8>, SimpleTree<u8>), Vec<Vec<u8>>, Vec<Vec<u8>>)) {
    let (vpair, map_src, map_dst) = example;
    let (stores, src, dst) = vpair_to_stores(vpair);
    let unstable_result1 = test_with_mappings(&stores, src, dst, &map_src, &map_dst, false);
    let unstable_result2 = test_with_mappings(&stores, dst, src, &map_dst, &map_src, false);

    assert_ne!(unstable_result1.src_to_dst, unstable_result2.dst_to_src);
    assert_ne!(unstable_result2.src_to_dst, unstable_result1.dst_to_src);
}

// This test should succeed on any example
fn test_stable(example: ((SimpleTree<u8>, SimpleTree<u8>), Vec<Vec<u8>>, Vec<Vec<u8>>)) {
    let (vpair, map_src, map_dst) = example;
    let (stores, src, dst) = vpair_to_stores(vpair);
    let stable_result1 = test_with_mappings(&stores, src, dst, &map_src, &map_dst, true);
    let stable_result2 = test_with_mappings(&stores, dst, src, &map_dst, &map_src, true);

    assert_eq!(stable_result1.src_to_dst, stable_result2.dst_to_src);
    assert_eq!(stable_result2.src_to_dst, stable_result1.dst_to_src);
}

fn test_with_mappings(
    stores: &SimpleStores<TStore, NS<Tree>, LS<u16>>,
    src: u16,
    dst: u16,
    map_src: &Vec<Vec<u8>>,
    map_dst: &Vec<Vec<u8>>,
    stable: bool,
) -> VecStore<u16> {
    print_tree(stores, src, "src tree");
    print_tree(stores, dst, "dst tree");
    let src_arena = Decompressible::<_, CompletePostOrder<_, u16>>::decompress(stores, &src);
    let dst_arena = Decompressible::<_, CompletePostOrder<_, u16>>::decompress(stores, &dst);

    let mut m: DefaultMappingStore<u16> = DefaultMappingStore::default();
    m.topit(src_arena.len(), dst_arena.len());

    let src = src_arena.root();
    let dst = dst_arena.root();

    for (map_src, map_dst) in map_src.iter().zip(map_dst) {
        m.link(
            src_arena.child(&src, &map_src),
            dst_arena.child(&dst, &map_dst),
        );
    }

    let mapping = Mapper {
        hyperast: stores,
        mapping: Mapping {
            src_arena,
            dst_arena,
            mappings: m,
        },
    };
    let mapping = if stable {
        SimpleMarriageBottomUpMatcher::<
            Decompressible<_, CompletePostOrder<_, u16>>,
            Decompressible<_, CompletePostOrder<_, u16>>,
            &SimpleStores<TStore, NS<Tree>, LS<u16>>,
            VecStore<u16>,
        >::match_it(mapping)
        .mapping
    } else {
        GreedyBottomUpMatcher::<
            Decompressible<_, CompletePostOrder<_, u16>>,
            Decompressible<_, CompletePostOrder<_, u16>>,
            &SimpleStores<TStore, NS<Tree>, LS<u16>>,
            VecStore<u16>,
        >::match_it(mapping)
        .mapping
    };
    println!(
        "{}",
        mapping.mappings.display(
            &|src: u16| mapping.src_arena.original(&src).to_string(),
            &|dst: u16| mapping.dst_arena.original(&dst).to_string(),
        )
    );
    return mapping.mappings;
}

fn print_tree(stores: &SimpleStores<TStore, NS<Tree>, LS<u16>>, src: u16, caption: &str) {
    let label_store = &stores.label_store;
    let node_store = &stores.node_store;
    println!(
        "{}:\n{:?}",
        caption,
        DisplayTree::new(label_store, node_store, src)
    );
}

fn print_mappings(
    src_arena: &Decompressible<
        &SimpleStores<TStore, NS<Tree>, LS<u16>>,
        CompletePostOrder<u16, u16>,
    >,
    dst_arena: &Decompressible<
        &SimpleStores<TStore, NS<Tree>, LS<u16>>,
        CompletePostOrder<u16, u16>,
    >,
    mappings: &VecStore<u16>,
) {
    println!("printing src -> dst mappings");
    for (i, m) in mappings.src_to_dst.iter().enumerate() {
        //let src = src_arena.original(&(i as u16));
        //let dst = dst_arena.original(m);
        println!("{} -> {}", i, m);
    }
}
