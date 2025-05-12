use hyper_diff::{actions::action_vec::actions_vec_f, algorithms};
use hyperast::{store::SimpleStores, types::NodeId};
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use std::path::Path;

const TEST_CASES: [(&str, &str, &str, usize); 3] = [
// Define test case paths relative to root/../datasets/defects4j
    // "Jsoup_17",
    // "before/Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java",
    // "after/Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java",
    (
        "Chart10",
        "before/Chart/10/source_org_jfree_chart_imagemap_StandardToolTipTagFragmentGenerator.java",
        "after/Chart/10/source_org_jfree_chart_imagemap_StandardToolTipTagFragmentGenerator.java",
        45 // Number of mappings when running java gumtree
    ),
    (
        "Csv1",
        "before/Csv/1/src_main_java_org_apache_commons_csv_ExtendedBufferedReader.java",
        "after/Csv/1/src_main_java_org_apache_commons_csv_ExtendedBufferedReader.java",
        414,
    ),
    (
        "Jsoup1",
        "before/Jsoup/1/src_main_java_org_jsoup_nodes_Document.java",
        "after/Jsoup/1/src_main_java_org_jsoup_nodes_Document.java",
        566
    )
];

fn test_cd_diff_single(i: usize) {
    // Get path to dataset
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("datasets/defects4j");

    let (name, buggy_rel_path, fixed_rel_path, number_of_mappings) = TEST_CASES[i];
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
    let (src_tr, dst_tr) =
        parse_string_pair(&mut stores, &mut md_cache, &buggy_content, &fixed_content);

    // Perform the diff using gumtree lazy
    let _diff_result = algorithms::gumtree::diff(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    );


    if let Some(actions) = _diff_result.actions {
        actions_vec_f(
            &actions,
            &_diff_result.mapper.hyperast,
            src_tr.local.compressed_node.as_id().clone(),
        )
    }

    assert_eq!(_diff_result.mapper.mappings.src_to_dst.len(), number_of_mappings)
}

#[test]
fn test_cd_diff_0() {
    test_cd_diff_single(0);
}

#[test]
fn test_cd_diff_1() {
    test_cd_diff_single(1);
}

#[test]
fn test_cd_diff_2() {
    test_cd_diff_single(2);
}