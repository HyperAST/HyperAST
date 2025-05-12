use std::fmt::Debug;
use std::time::Instant;
use hyper_diff::decompressed_tree_store::{CompletePostOrder, FullyDecompressedTreeStore, PostOrder};
use hyper_diff::decompressed_tree_store::complete_post_order::DisplayCompletePostOrder;
use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
use hyper_diff::decompressed_tree_store::pre_order_wrapper::{DisplaySimplePreOrderMapper, SimplePreOrderMapper};
use hyper_diff::matchers::heuristic::gt::bottom_up_matcher::BottomUpMatcher;
use hyper_diff::matchers::heuristic::gt::greedy_bottom_up_matcher::GreedyBottomUpMatcher;
use hyper_diff::matchers::heuristic::gt::greedy_subtree_matcher::{GreedySubtreeMatcher, SubtreeMatcher};
use hyper_diff::matchers::heuristic::gt::simple_bottom_up_matcher3::SimpleBottomUpMatcher3;
use hyper_diff::matchers::{mapping_store, Decompressible, Mapper};
use hyper_diff::matchers::mapping_store::{DefaultMappingStore, DefaultMultiMappingStore, MappingStore, MonoMappingStore, VecStore};
use hyper_diff::tree::tree_path::CompressedTreePath;
use hyperast::full::FullNode;
use hyperast::store::SimpleStores;
use hyperast::tree_gen::StatsGlobalData;
use hyperast::types;
use hyperast::types::{HyperAST, HyperASTShared, NodeId};
use hyperast_gen_ts_java::legion_with_refs;
use hyperast_gen_ts_java::legion_with_refs::{JavaTreeGen, Local};
use hyperast_gen_ts_java::types::TStore;

type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;

pub fn get_mappings_gumtree_simple<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
) -> VecStore<u32>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let mapper: Mapper<_, CDS<HAST>, CDS<HAST>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    let mapper =
        GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
    let subtree_mappings = mapper.mappings();
    dbg!(&subtree_mappings);
    let now = Instant::now();
    let mapper = SimpleBottomUpMatcher3::<_, _, _, _>::match_it(mapper);
    let bottomup_matcher_t = now.elapsed().as_secs_f64();
    let bottomup_mappings = mapper.mappings();
    dbg!(&bottomup_matcher_t, &bottomup_mappings);

    bottomup_mappings.clone()
}

// Written by Elias
fn preprocess_for_diff(
    src: &[u8],
    dst: &[u8],
) -> (
    SimpleStores<TStore>,
    FullNode<StatsGlobalData, Local>,
    FullNode<StatsGlobalData, Local>,
) {
    let mut stores = SimpleStores::<TStore>::default();
    let mut md_cache = Default::default(); // [cite: 133, 139]
    let mut java_tree_gen = JavaTreeGen::new(&mut stores, &mut md_cache);
    let tree = match legion_with_refs::tree_sitter_parse(src) {
        Ok(t) => t,
        Err(t) => t,
    };
    let src = java_tree_gen.generate_file(b"", src, tree.walk());
    let tree = match legion_with_refs::tree_sitter_parse(dst) {
        Ok(t) => t,
        Err(t) => t,
    };
    let dst = java_tree_gen.generate_file(b"", dst, tree.walk());
    return (stores, src, dst);
}

#[test]
fn test_gumtree_simple_matcher3() {
    
}