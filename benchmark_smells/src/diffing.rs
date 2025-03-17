use hyper_diff::{
    decompressed_tree_store::{lazy_post_order::LazyPostOrder, ShallowDecompressedTreeStore},
    matchers::{
        heuristic::gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
        mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore},
        Decompressible, Mapper, Mapping,
    },
};
use hyperast::types::{self, HyperAST, NodeId};
use std::fmt::Debug;

fn _top_down<HAST: HyperAST + Copy>(
    mapper: &mut Mapper<
        HAST,
        Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, u32>>,
        Decompressible<HAST, &mut LazyPostOrder<HAST::IdN, u32>>,
        VecStore<u32>,
    >,
) where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Label: Clone + Copy + Eq + Debug,
    HAST::Idx: Debug,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let mm = LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::compute_multi_mapping::<
        DefaultMultiMappingStore<_>,
    >(mapper);
    LazyGreedySubtreeMatcher::<_, _, _, VecStore<_>>::filter_mappings(mapper, &mm);
}

pub fn top_down<'a, HAST: HyperAST + Copy>(
    hyperast: HAST,
    src_arena: &'a mut LazyPostOrder<HAST::IdN, u32>,
    dst_arena: &'a mut LazyPostOrder<HAST::IdN, u32>,
) -> Mapper<
    HAST,
    Decompressible<HAST, &'a mut LazyPostOrder<HAST::IdN, u32>>,
    Decompressible<HAST, &'a mut LazyPostOrder<HAST::IdN, u32>>,
    VecStore<u32>,
>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Label: Clone + Copy + Eq + Debug,
    HAST::Idx: Debug,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let mappings = VecStore::<u32>::default();
    let src_arena = Decompressible {
        hyperast,
        decomp: src_arena,
    };
    let dst_arena = Decompressible {
        hyperast,
        decomp: dst_arena,
    };
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
