use hyper_diff::{actions::{action_vec::actions_vec_f, action_vec::ActionsVec}, algorithms};
use hyper_diff::tree::tree_path::CompressedTreePath;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use std::path::Path;
use hyper_diff::actions::Actions;
use std::process::Command;
use std::io::Write;
use serde::Deserialize;
use hyper_diff::actions::script_generator2::{Act, SimpleAction};
use hyper_diff::matchers::Mapping;
use hyperast::{
    full::FullNode, nodes::SyntaxSerializer, store::SimpleStores, tree_gen::StatsGlobalData, types::NodeId
};
use hyperast_gen_ts_java::{
    legion_with_refs::{self, JavaTreeGen, Local},
    types::TStore,
};
use hyperast_gen_ts_java::legion_with_refs::FNode;

const GUMTREE_JAVA_PATH: &str = "../gumtree/dist/build/install/gumtree/bin/gumtree";
const TEMP_FOLDER: &str = "../gumtree/dist/";

const DATASET_PATH: &str = "../datasets/defects4j/";

const TEST_CASES: [(&str, &str, &str, usize, usize); 3] = [
// Define test case paths relative to root/../datasets/defects4j
    // "Jsoup_17",
    // "before/Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java",
    // "after/Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java",
    (
        "Chart10",
        "before/Chart/10/source_org_jfree_chart_imagemap_StandardToolTipTagFragmentGenerator.java",
        "after/Chart/10/source_org_jfree_chart_imagemap_StandardToolTipTagFragmentGenerator.java",
        45, // Number of mappings when running java gumtree
        2,
    ),
    (
        "Csv1",
        "before/Csv/1/src_main_java_org_apache_commons_csv_ExtendedBufferedReader.java",
        "after/Csv/1/src_main_java_org_apache_commons_csv_ExtendedBufferedReader.java",
        414,
        8,
    ),
    (
        "Jsoup1",
        "before/Jsoup/1/src_main_java_org_jsoup_nodes_Document.java",
        "after/Jsoup/1/src_main_java_org_jsoup_nodes_Document.java",
        566,
        3,
    )
];

#[derive(Deserialize)]
struct GumTreeOutput {
    matches: Vec<Match>,
    actions: Vec<GumTreeAction>,
}

#[derive(Deserialize)]
struct Match {
    src: String,
    dest: String,
}

#[derive(Deserialize)]
struct GumTreeAction {
    action: String,
    tree: String,
    parent: Option<String>,
    at: Option<usize>,
}

fn parse_gumtree_output(path: &Path) -> (usize, usize) { // (usize, ActionsVec<SimpleAction<LabelIdentifier, CompressedTreePath<u16>, NodeIdentifier>>) {
    let output = std::fs::read_to_string(path)
        .expect("Failed to read gumtree output file");

    let parsed: GumTreeOutput = serde_json::from_str(&output)
        .expect("Failed to parse GumTree JSON output");

    let matches_count = parsed.matches.len();
    // let actions = parsed.actions.into_iter()
    //     .map(|a| SimpleAction {
    //         path: ?,
    //         action: match a.action.as_str() {
    //             "delete" => Act::Delete {},
    //             "update" => Act::Update { new: ? },
    //             "move" => Act::Move { from: ? },
    //             "insert" => Act::Insert { sub: ? },
    //             _ => panic!("Unknown action type: {}", a.action)
    //         }
    //     })
    //     .collect();
    let actions_count = parsed.actions.len();

    (matches_count, actions_count)
}

fn prepare_tree_print<'a>(
    stores: &'a SimpleStores<TStore>,
) -> impl Fn(&FullNode<StatsGlobalData, Local>) -> () + 'a {
    return |tree: &FullNode<StatsGlobalData, Local>| {
        println!();
        println!(
            "{}",
            SyntaxSerializer::new(stores, tree.local.compressed_node)
        );
    };
}

// fn print_mappings(mappings: Mapping<NodeId>, src_tr: &FNode, dst_tr: &FNode) {
//
// }

// fn test_custom_tree(name: &str, buggy_rel_path: &str, fixed_rel_path: &str) {
//     // Get path to dataset
// 
//     // Initialize stores
//     let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
//     let mut md_cache = Default::default();
// 
//     // Parse the two Java files
//     let (src_tr, dst_tr) =
//         parse_string_pair(&mut stores, &mut md_cache, &buggy_content, &fixed_content);
//     // let src_tr = tree!(
//     // 
//     // );
// 
//     // Perform the diff using gumtree lazy
//     let _diff_result = algorithms::gumtree::diff(
//         &stores,
//         &src_tr.local.compressed_node,
//         &dst_tr.local.compressed_node,
//     );
// 
// 
//     let print_tree = prepare_tree_print(&stores);
//     print_tree(&src_tr);
//     print_tree(&dst_tr);
// 
//     let actions = _diff_result.actions.expect("Expected a result");
//     actions_vec_f(
//         &actions,
//         &_diff_result.mapper.hyperast,
//         src_tr.local.compressed_node.as_id().clone(),
//     );
//     
//     // print_mappings(&_diff_result.mapper.mappings, &src_tr, &dst_tr);
//     // for (let m in &_diff_result.mapper.mappings.src_to_dst) {
//     //     src_tr.
//     // }
//     
//     let hyperast_actions_len = actions.len();
//     let hyperast_matches_len = _diff_result.mapper.mappings.src_to_dst.iter().filter(|a| **a != 0).count();
// 
//     assert_eq!(hyperast_actions_len, gumtree_actions_len);
//     assert_eq!(hyperast_matches_len, gumtree_matches_len);
// }

fn test_cd_diff_single(name: &str, buggy_rel_path: &str, fixed_rel_path: &str) {
    // Get path to dataset
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(DATASET_PATH);

    let buggy_path = root.join(buggy_rel_path);
    let fixed_path = root.join(fixed_rel_path);

    // Call gumtree java to get number of mappings and edit script length
    let gumtree_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join(GUMTREE_JAVA_PATH);
    let mut temp_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join(TEMP_FOLDER).join(name);
    temp_path.set_extension("json");
    let output = Command::new(gumtree_path)
        .args(&[
            "textdiff",
            buggy_path.to_str().unwrap(),
            fixed_path.to_str().unwrap(),
            "-m", "gumtree-simple",
            "-g", "java-treesitter",
            "-d", "chawathe",
            "-f", "json",
            "-o", temp_path.to_str().expect("Failed to get temp path")
        ])
        .output()
        .expect("Failed to execute gumtree command");

    if !output.status.success() {
        panic!("Gumtree command failed: {}", String::from_utf8_lossy(&output.stderr));
    }

    dbg!(String::from_utf8_lossy(&output.stdout));
    dbg!(String::from_utf8_lossy(&output.stderr));
    // Parse the GumTree output
    let (gumtree_matches_len, gumtree_actions_len) = parse_gumtree_output(&temp_path);


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


    let print_tree = prepare_tree_print(&stores);
    print_tree(&src_tr);
    print_tree(&dst_tr);

    let actions = _diff_result.actions.expect("Expected a result");
    // actions_vec_f(
    //     &actions,
    //     &_diff_result.mapper.hyperast,
    //     src_tr.local.compressed_node.as_id().clone(),
    // );

    dbg!(&_diff_result.mapper.mappings.src_to_dst);
    dbg!(&_diff_result.mapper.mappings.dst_to_src);

    let hyperast_actions_len = actions.len();
    let hyperast_matches_len = _diff_result.mapper.mappings.src_to_dst.iter().filter(|a| **a != 0).count();

    assert_eq!(hyperast_actions_len, gumtree_actions_len);
    assert_eq!(hyperast_matches_len, gumtree_matches_len);
}

#[test]
fn test_cd_diff_0() {
    let (name, buggy_rel_path, fixed_rel_path,_,_) = TEST_CASES[0];

    test_cd_diff_single(name, buggy_rel_path, fixed_rel_path);
}

#[test]
fn test_cd_diff_1() {
    let (name, buggy_rel_path, fixed_rel_path,_,_) = TEST_CASES[1];

    test_cd_diff_single(name, buggy_rel_path, fixed_rel_path);
}

#[test]
fn test_cd_diff_2() {
    let (name, buggy_rel_path, fixed_rel_path,_,_) = TEST_CASES[2];

    test_cd_diff_single(name, buggy_rel_path, fixed_rel_path);
}

#[test]
fn test_cd_diff_3() {
    test_cd_diff_single(
        "simple_class",
        "../custom/before/simple_class.java",
        "../custom/after/simple_class.java",
    )
}

fn find_java_files(dir: &Path) -> Vec<String> {
    let mut java_files = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_dir() {
                    java_files.extend(find_java_files(&path));
                } else if path.extension().and_then(|s| s.to_str()) == Some("java") {
                    if let Some(rel_path) = path.strip_prefix(dir).ok().and_then(|p| p.to_str()) {
                        java_files.push(rel_path.to_string());
                    }
                }
            }
        }
    }
    java_files
}

#[test]
fn test_all() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(DATASET_PATH);

    let before_dir = root.join("before");
    let java_files = find_java_files(&before_dir);

    for rel_path in java_files {
        dbg!(&rel_path);
        let name = rel_path.replace("/", "-");
        let before_path = format!("before/{}", rel_path);
        let after_path = format!("after/{}", rel_path);
        test_cd_diff_single(&name, &before_path, &after_path);
    }
}