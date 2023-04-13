use std::fmt::Debug;

use hyper_ast::types::{self, HyperAST};
use hyper_diff::{decompressed_tree_store::ShallowDecompressedTreeStore, matchers::Mapper};

use hyper_diff::decompressed_tree_store::hidding_wrapper;
use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
use hyper_diff::matchers::heuristic::gt::{
    lazy2_greedy_bottom_up_matcher::GreedyBottomUpMatcher,
    lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
};
use hyper_diff::matchers::mapping_store::DefaultMultiMappingStore;
use hyper_diff::matchers::mapping_store::MappingStore;
use hyper_diff::matchers::mapping_store::VecStore;
use hyper_diff::matchers::Mapping;

pub fn top_down<'store, HAST: HyperAST<'store>>(
    hyperast: &'store HAST,
    src_arena: &mut LazyPostOrder<HAST::T, u32>,
    dst_arena: &mut LazyPostOrder<HAST::T, u32>,
) -> DefaultMultiMappingStore<u32>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::Label: Clone + Copy + Eq + Debug,
    <HAST::T as types::Typed>::Type: Debug,
    <HAST::T as types::WithChildren>::ChildIdx: Debug,
    HAST::T: 'store + types::WithHashs + types::WithStats,
{
    let mut mm: DefaultMultiMappingStore<_> = Default::default();
    mm.topit(src_arena.len(), dst_arena.len());
    Mapper::<_, _, _, VecStore<u32>>::compute_multimapping::<_, 1>(
        hyperast, src_arena, dst_arena, &mut mm,
    );
    mm
}

pub fn full<'store, HAST: HyperAST<'store>>(
    hyperast: &'store HAST,
    mapper: &mut Mapper<
        'store,
        HAST,
        &mut LazyPostOrder<HAST::T, u32>,
        &mut LazyPostOrder<HAST::T, u32>,
        VecStore<u32>,
    >,
) where
    HAST::IdN: Clone + Debug + Eq,
    HAST::Label: Clone + Copy + Eq + Debug,
    <HAST::T as types::Typed>::Type: Debug,
    <HAST::T as types::WithChildren>::ChildIdx: Debug,
    HAST::T: 'store + types::WithHashs + types::WithStats,
{
    let mm = LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::compute_multi_mapping::<
        DefaultMultiMappingStore<_>,
    >(mapper);
    LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::filter_mappings(mapper, &mm);
    GreedyBottomUpMatcher::<_, _, _, _, VecStore<_>>::execute(mapper, hyperast.label_store());
}

pub fn full2<'store, HAST: HyperAST<'store>>(
    hyperast: &'store HAST,
    mut mapper: Mapper<
        'store,
        HAST,
        &mut LazyPostOrder<HAST::T, u32>,
        &mut LazyPostOrder<HAST::T, u32>,
        VecStore<u32>,
    >,
) -> VecStore<u32>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::Label: Clone + Copy + Eq + Debug,
    <HAST::T as types::Typed>::Type: Debug,
    <HAST::T as types::WithChildren>::ChildIdx: Debug,
    HAST::T: 'store + types::WithHashs + types::WithStats,
{
    let mm = LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::compute_multi_mapping::<
        DefaultMultiMappingStore<_>,
    >(&mut mapper);
    LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::filter_mappings(&mut mapper, &mm);
    use hidding_wrapper::*;

    // # hide matched subtrees
    // from right to left map unmatched nodes in a simple vec,
    let (map_src, rev_src) = hiding_map(
        &mapper.mapping.src_arena,
        &mapper.mapping.mappings.src_to_dst,
    );
    let (map_dst, rev_dst) = hiding_map(
        &mapper.mapping.dst_arena,
        &mapper.mapping.mappings.dst_to_src,
    );
    // a simple arithmetic op allow to still have nodes in post order where root() == len() - 1
    {
        let (src_arena, dst_arena, mappings) = hide(
            mapper.mapping.src_arena,
            &map_src,
            &rev_src,
            mapper.mapping.dst_arena,
            &map_dst,
            &rev_dst,
            &mut mapper.mapping.mappings,
        );
        // also wrap mappings (needed because bottom up matcher reads it)
        // then do the bottomup mapping (need another mapper)
        let mut mapper = Mapper {
            hyperast: mapper.hyperast,
            mapping: Mapping {
                src_arena,
                dst_arena,
                mappings,
            },
        };
        GreedyBottomUpMatcher::<_, _, _, _, VecStore<_>>::execute(
            &mut mapper,
            hyperast.label_store(),
        );
    }
    mapper.mapping.mappings
}
