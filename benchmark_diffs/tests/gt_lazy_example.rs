use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use std::path::Path;

// Define test case paths relative to root/../datasets/defects4j
const TEST_CASE: (&str, &str, &str) = (
    "Jsoup_17",
    "before/Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java",
    "after/Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java",
);

#[test]
fn test_gt_lazy_jsoup_diff() {
    // Get path to dataset
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("datasets/defects4j");

    let (_name, buggy_rel_path, fixed_rel_path) = TEST_CASE;
    let buggy_path = root.join(buggy_rel_path);
    let fixed_path = root.join(fixed_rel_path);

    // Read file contents
    let buggy_content = std::fs::read_to_string(&buggy_path)
        .expect(&format!("Failed to read buggy file: {:?}", buggy_path));
    let fixed_content = std::fs::read_to_string(&fixed_path)
        .expect(&format!("Failed to read fixed file: {:?}", fixed_path));

    // Initialize stores
    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();

    // Parse the two Java files
    let [src_tr, dst_tr] =
        parse_string_pair(&mut stores, &mut md_cache, &buggy_content, &fixed_content);

    // Perform the diff using gumtree lazy
    let _diff_result = algorithms::gumtree_lazy::diff(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    );
}
