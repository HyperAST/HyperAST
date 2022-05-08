use std::ops::Index;

use num_traits::{zero, PrimInt};

use crate::{matchers::{decompressed_tree_store::{BreathFirst, CompletePostOrder, DecompressedTreeStore, ShallowDecompressedTreeStore}, heuristic::gt::optimized_greedy_subtree_matcher::{GreedySubtreeMatcher, SubtreeMatcher}, mapping_store::{DefaultMappingStore, MappingStore}}, tests::{
        examples::{example_gumtree, example_gumtree_ambiguous},
        simple_tree::{vpair_to_stores},
    }, tree::tree::LabelStore};

#[test]
fn test_min_height_threshold() {
    let (label_store, node_store, src, dst) = vpair_to_stores(example_gumtree());
    assert_eq!(label_store.get_node_at_id(&0).to_owned(), b"");
    let mappings = DefaultMappingStore::new();
    // GreedySubtreeMatcher.MIN_HEIGHT = 0;
    let mapper = GreedySubtreeMatcher::<CompletePostOrder<_, u16>, _, _, _, 0>::matchh(
        &node_store,
        &src,
        &dst,
        mappings,
    );
    let SubtreeMatcher {
        src_arena,
        dst_arena,
        mappings: ms1,
        ..
    } = mapper.into();
    {
        let src = &src_arena.root();
        let dst = &dst_arena.root();
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[1]),
            &dst_arena.child(&node_store, dst, &[0])
        ));
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[1, 0]),
            &dst_arena.child(&node_store, dst, &[0, 0])
        ));
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[1, 1]),
            &dst_arena.child(&node_store, dst, &[0, 1])
        ));
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[2]),
            &dst_arena.child(&node_store, dst, &[2])
        ));
        assert_eq!(4, ms1.len());
    }
    let mappings = DefaultMappingStore::new();
    // GreedySubtreeMatcher.MIN_HEIGHT = 1;
    let mapper = GreedySubtreeMatcher::<CompletePostOrder<_, u16>, _, _, _, 1>::matchh(
        &node_store,
        &src,
        &dst,
        mappings,
    );
    let SubtreeMatcher {
        src_arena,
        dst_arena,
        mappings: ms2,
        ..
    } = mapper.into();
    let src = &src_arena.root();
    let dst = &dst_arena.root();
    assert!(ms2.has(
        &src_arena.child(&node_store, src, &[1]),
        &dst_arena.child(&node_store, dst, &[0])
    ));
    assert!(ms2.has(
        &src_arena.child(&node_store, src, &[1, 0]),
        &dst_arena.child(&node_store, dst, &[0, 0])
    ));
    assert!(ms2.has(
        &src_arena.child(&node_store, src, &[1, 1]),
        &dst_arena.child(&node_store, dst, &[0, 1])
    ));
    assert_eq!(3, ms2.len());
}
#[test]
fn test_min_height_threshold_hybrid() {
    let (label_store, node_store, src, dst) = vpair_to_stores(example_gumtree());
    assert_eq!(label_store.get_node_at_id(&0).to_owned(), b"");
    let mappings = DefaultMappingStore::new();
    // GreedySubtreeMatcher.MIN_HEIGHT = 0;
    let mapper = GreedySubtreeMatcher::<CompletePostOrder<_, u16>, _, _, _, 0>::matchh(
        &node_store,
        &src,
        &dst,
        mappings,
    );
    let SubtreeMatcher {
        src_arena,
        dst_arena,
        mappings: ms1,
        ..
    } = mapper.into();
    {
        let src = &src_arena.root();
        let dst = &dst_arena.root();
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[1]),
            &dst_arena.child(&node_store, dst, &[0])
        ));
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[1, 0]),
            &dst_arena.child(&node_store, dst, &[0, 0])
        ));
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[1, 1]),
            &dst_arena.child(&node_store, dst, &[0, 1])
        ));
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[2]),
            &dst_arena.child(&node_store, dst, &[2])
        ));
        assert_eq!(4, ms1.len());
    }
    let mappings = DefaultMappingStore::new();
    // GreedySubtreeMatcher.MIN_HEIGHT = 1;
    let mapper = GreedySubtreeMatcher::<CompletePostOrder<_, u16>, _, _, _, 1>::matchh(
        &node_store,
        &src,
        &dst,
        mappings,
    );
    let SubtreeMatcher {
        src_arena,
        dst_arena,
        mappings: ms2,
        ..
    } = mapper.into();
    let src = &src_arena.root();
    let dst = &dst_arena.root();
    assert!(ms2.has(
        &src_arena.child(&node_store, src, &[1]),
        &dst_arena.child(&node_store, dst, &[0])
    ));
    assert!(ms2.has(
        &src_arena.child(&node_store, src, &[1, 0]),
        &dst_arena.child(&node_store, dst, &[0, 0])
    ));
    assert!(ms2.has(
        &src_arena.child(&node_store, src, &[1, 1]),
        &dst_arena.child(&node_store, dst, &[0, 1])
    ));
    assert_eq!(3, ms2.len());
}

#[test]
fn test_min_height_threshold_ambiguous() {
    let (label_store, node_store, src, dst) = vpair_to_stores(example_gumtree_ambiguous());
    assert_eq!(label_store.get_node_at_id(&0).to_owned(), b"");
    let mappings = DefaultMappingStore::new();
    // GreedySubtreeMatcher.MIN_HEIGHT = 0;
    let mapper = GreedySubtreeMatcher::<CompletePostOrder<_, u16>, _, _, _, 0>::matchh(
        &node_store,
        &src,
        &dst,
        mappings,
    );
    let SubtreeMatcher {
        src_arena,
        dst_arena,
        mappings: ms1,
        ..
    } = mapper.into();
    {
        let src = &src_arena.root();
        let dst = &dst_arena.root();
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[1]),
            &dst_arena.child(&node_store, dst, &[3])
        ));
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[1, 0]),
            &dst_arena.child(&node_store, dst, &[3, 0])
        ));
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[1, 1]),
            &dst_arena.child(&node_store, dst, &[3, 1])
        ));
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[2]),
            &dst_arena.child(&node_store, dst, &[2])
        ));
        assert_eq!(4, ms1.len());
    }
    let mappings = DefaultMappingStore::new();
    // GreedySubtreeMatcher.MIN_HEIGHT = 1;
    let mapper = GreedySubtreeMatcher::<CompletePostOrder<_, u16>, _, _, _, 1>::matchh(
        &node_store,
        &src,
        &dst,
        mappings,
    );
    let SubtreeMatcher {
        src_arena,
        dst_arena,
        mappings: ms2,
        ..
    } = mapper.into();
    let src = &src_arena.root();
    let dst = &dst_arena.root();
    assert!(ms1.has(
        &src_arena.child(&node_store, src, &[1]),
        &dst_arena.child(&node_store, dst, &[3])
    ));
    assert!(ms1.has(
        &src_arena.child(&node_store, src, &[1, 0]),
        &dst_arena.child(&node_store, dst, &[3, 0])
    ));
    assert!(ms1.has(
        &src_arena.child(&node_store, src, &[1, 1]),
        &dst_arena.child(&node_store, dst, &[3, 1])
    ));
    assert_eq!(3, ms2.len());
}

#[test]
fn test_min_height_threshold_ambiguous_hybrid() {
    let (label_store, node_store, src, dst) = vpair_to_stores(example_gumtree_ambiguous());
    assert_eq!(label_store.get_node_at_id(&0).to_owned(), b"");
    let mappings = DefaultMappingStore::new();
    // GreedySubtreeMatcher.MIN_HEIGHT = 0;
    let mapper = GreedySubtreeMatcher::<BreathFirst<_, u16>, _, _, _, 0>::matchh(
        &node_store,
        &src,
        &dst,
        mappings,
    );
    let SubtreeMatcher {
        src_arena,
        dst_arena,
        mappings: ms1,
        ..
    } = mapper.into();
    {
        let src = &src_arena.root();
        let dst = &dst_arena.root();
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[1]),
            &dst_arena.child(&node_store, dst, &[3])
        ));
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[1, 0]),
            &dst_arena.child(&node_store, dst, &[3, 0])
        ));
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[1, 1]),
            &dst_arena.child(&node_store, dst, &[3, 1])
        ));
        assert!(ms1.has(
            &src_arena.child(&node_store, src, &[2]),
            &dst_arena.child(&node_store, dst, &[2])
        ));
        assert_eq!(4, ms1.len());
    }
    let mappings = DefaultMappingStore::new();
    // GreedySubtreeMatcher.MIN_HEIGHT = 1;
    let mapper = GreedySubtreeMatcher::<CompletePostOrder<_, u16>, _, _, _, 1>::matchh(
        &node_store,
        &src,
        &dst,
        mappings,
    );
    let SubtreeMatcher {
        src_arena,
        dst_arena,
        mappings: ms2,
        ..
    } = mapper.into();
    let src = &src_arena.root();
    let dst = &dst_arena.root();
    assert!(ms1.has(
        &src_arena.child(&node_store, src, &[1]),
        &dst_arena.child(&node_store, dst, &[3])
    ));
    assert!(ms1.has(
        &src_arena.child(&node_store, src, &[1, 0]),
        &dst_arena.child(&node_store, dst, &[3, 0])
    ));
    assert!(ms1.has(
        &src_arena.child(&node_store, src, &[1, 1]),
        &dst_arena.child(&node_store, dst, &[3, 1])
    ));
    assert_eq!(3, ms2.len());
}
