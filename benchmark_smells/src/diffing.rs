use hyperast::types::{self, HyperAST};
use hyper_diff::{
    decompressed_tree_store::{lazy_post_order::LazyPostOrder, ShallowDecompressedTreeStore},
    matchers::{
        heuristic::gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
        mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore},
        Mapper, Mapping,
    },
};
use std::fmt::Debug;

fn _top_down<'store, HAST: HyperAST<'store>>(
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
    <HAST::T as types::WithChildren>::ChildIdx: Debug,
    for<'t> HAST::T<'t>: 'store + types::WithHashs + types::WithStats,
{
    let mm = LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::compute_multi_mapping::<
        DefaultMultiMappingStore<_>,
    >(mapper);
    LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::filter_mappings(mapper, &mm);
}

pub fn top_down<'store, 'a, HAST: HyperAST<'store>>(
    hyperast: &'store HAST,
    src_arena: &'a mut LazyPostOrder<HAST::T, u32>,
    dst_arena: &'a mut LazyPostOrder<HAST::T, u32>,
) -> Mapper<
    'store,
    HAST,
    &'a mut LazyPostOrder<HAST::T, u32>,
    &'a mut LazyPostOrder<HAST::T, u32>,
    VecStore<u32>,
>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::Label: Clone + Copy + Eq + Debug,
    <HAST::T as types::WithChildren>::ChildIdx: Debug,
    for<'t> HAST::T<'t>: 'store + types::WithHashs + types::WithStats,
{
    let mappings = VecStore::<u32>::default();
    let mut mapper = Mapper {
        hyperast,
        mapping: Mapping {
            src_arena,
            dst_arena,
            mappings,
        },
    };
    mapper.mapping.mappings.topit(
        mapper.mapping.src_arena.len(),
        mapper.mapping.dst_arena.len(),
    );
    _top_down(&mut mapper);
    mapper
}
