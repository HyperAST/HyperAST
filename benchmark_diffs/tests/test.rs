use hyper_diff::algorithms;
use hyper_diff::decompressed_tree_store::CompletePostOrder;
use hyper_diff::matchers::Decompressible;
use hyperast::store::SimpleStores;
use hyperast::types::HyperASTShared;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;

const BEFORE_CONTENT: &str = include_str!("../gt_datasets/gh-java/before/elastic-search/1d732dfc1b08deca0f20b467fe4c66f041bb37b5/JsonSettingsLoader.java");
const AFTER_CONTENT: &str = include_str!("../gt_datasets/gh-java/after/elastic-search/1d732dfc1b08deca0f20b467fe4c66f041bb37b5/JsonSettingsLoader.java");
#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use criterion::black_box;
    use hyper_diff::actions::Actions;
    use hyper_diff::matchers::heuristic::gt::greedy_bottom_up_matcher::GreedyBottomUpMatcher;
    use hyper_diff::matchers::heuristic::gt::greedy_subtree_matcher::GreedySubtreeMatcher;
    use hyper_diff::matchers::Mapper;
    use hyper_diff::matchers::mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore};
    use hyperast::types::HyperAST;
    use hyperast_gen_ts_java::legion_with_refs;
    use super::*;

    #[test]
    fn run_diff() {
        println!("Testing...");

        let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
        let mut md_cache = Default::default();

        // Parse the two Java files
        parse_string_pair(
            &mut stores,
            &mut md_cache,
            &BEFORE_CONTENT,
            &AFTER_CONTENT,
        );

        let size = stores.node_store.len();
        println!("Size: {}", size);

        let tree = legion_with_refs::tree_sitter_parse(BEFORE_CONTENT.as_bytes()).unwrap();
        let file = File::create("before.dot").unwrap();
        tree.print_dot_graph(&file);

        let tree = legion_with_refs::tree_sitter_parse(AFTER_CONTENT.as_bytes()).unwrap();
        let file = File::create("after.dot").unwrap();
        tree.print_dot_graph(&file);
    }

    #[test]
    fn simple() {
        let before_content = "class A {}";
        let after_content = "class B {}";

        let mut hyperast = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
        let mut md_cache = Default::default();

        let (src_tr, dst_tr) = parse_string_pair(
            &mut hyperast,
            &mut md_cache,
            black_box(&before_content),
            black_box(&after_content),
        );

        let mapper: Mapper<_, CDS<_>, CDS<_>, VecStore<_>> = (&hyperast)
            .decompress_pair(&src_tr.local.compressed_node, &dst_tr.local.compressed_node)
            .into();

        let mapper = GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
        
        let matches_before = mapper.mappings().len();
        
        let mapper = GreedyBottomUpMatcher::<_, _, _, _>::match_it(mapper);
        
        let matches_after = mapper.mappings().len();

        let tree = legion_with_refs::tree_sitter_parse(before_content.as_bytes()).unwrap();
        let file = File::create("before.dot").unwrap();
        tree.print_dot_graph(&file);

        let tree = legion_with_refs::tree_sitter_parse(after_content.as_bytes()).unwrap();
        let file = File::create("after.dot").unwrap();
        tree.print_dot_graph(&file);

        let size_src = mapper.mapping.src_arena.iter().count();
        let size_dst = mapper.mapping.dst_arena.iter().count();
        let size = hyperast.node_store.len();

        println!("Size: {}, {}, {}", size, size_src, size_dst);
        println!("Matches: {}, {}", matches_before, matches_after);
    }
}


#[allow(type_alias_bounds)]
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;