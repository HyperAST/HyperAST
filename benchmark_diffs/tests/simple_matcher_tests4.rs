use hyper_diff::actions::Actions;
use hyper_diff::algorithms;
use hyperast::{
    full::FullNode, nodes::SyntaxSerializer, store::SimpleStores, tree_gen::StatsGlobalData,
};
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use hyperast_gen_ts_java::{legion_with_refs::Local, types::TStore};
use serde::Deserialize;
use std::path::{Path, PathBuf};

const DATASET_PATH: &str = "../datasets/defects4j/";

const TEST_CASES: &[(&str, &str, &str)] = &[
    (
        "Mockito_31",
        "before/Mockito/31/src_org_mockito_internal_stubbing_defaultanswers_ReturnsSmartNulls.java",
        "after/Mockito/31/src_org_mockito_internal_stubbing_defaultanswers_ReturnsSmartNulls.java",
    ),
    (
        "Mockito_32",
        "before/Mockito/32/src_org_mockito_internal_configuration_SpyAnnotationEngine.java",
        "after/Mockito/32/src_org_mockito_internal_configuration_SpyAnnotationEngine.java",
    ),
    (
        "Mockito_34",
        "before/Mockito/34/src_org_mockito_internal_invocation_InvocationMatcher.java",
        "after/Mockito/34/src_org_mockito_internal_invocation_InvocationMatcher.java",
    ),
    (
        "Mockito_37",
        "before/Mockito/37/src_org_mockito_internal_stubbing_answers_AnswersValidator.java",
        "after/Mockito/37/src_org_mockito_internal_stubbing_answers_AnswersValidator.java",
    ),
    (
        "Mockito_38",
        "before/Mockito/38/src_org_mockito_internal_verification_argumentmatching_ArgumentMatchingTool.java",
        "after/Mockito/38/src_org_mockito_internal_verification_argumentmatching_ArgumentMatchingTool.java",
    ),
    (
        "Mockito_9",
        "before/Mockito/9/src_org_mockito_internal_stubbing_answers_CallsRealMethods.java",
        "after/Mockito/9/src_org_mockito_internal_stubbing_answers_CallsRealMethods.java",
    ),
    (
        "Time_26",
        "before/Time/26/src_main_java_org_joda_time_field_LenientDateTimeField.java",
        "after/Time/26/src_main_java_org_joda_time_field_LenientDateTimeField.java",
    ),
    (
        "Chart_10",
        "before/Chart/10/source_org_jfree_chart_imagemap_StandardToolTipTagFragmentGenerator.java",
        "after/Chart/10/source_org_jfree_chart_imagemap_StandardToolTipTagFragmentGenerator.java",
    ),
    (
        "Cli_12",
        "before/Cli/12/src_java_org_apache_commons_cli_GnuParser.java",
        "after/Cli/12/src_java_org_apache_commons_cli_GnuParser.java",
    ),
    (
        "Cli_13",
        "before/Cli/13/src_java_org_apache_commons_cli2_WriteableCommandLine.java",
        "after/Cli/13/src_java_org_apache_commons_cli2_WriteableCommandLine.java",
    ),
    (
        "Cli_21",
        "before/Cli/21/src_java_org_apache_commons_cli2_WriteableCommandLine.java",
        "after/Cli/21/src_java_org_apache_commons_cli2_WriteableCommandLine.java",
    ),
    (
        "Cli_29",
        "before/Cli/29/src_java_org_apache_commons_cli_Util.java",
        "after/Cli/29/src_java_org_apache_commons_cli_Util.java",
    ),
    (
        "JxPath_7a",
        "before/JxPath/7/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationLessThan.java",
        "after/JxPath/7/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationLessThan.java",
    ),
    (
        "JxPath_7b",
        "before/JxPath/7/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationLessThanOrEqual.java",
        "after/JxPath/7/src_java_org_apache_commons_jxpath_ri_compiler_CoreOperationLessThanOrEqual.java",
    ),
];

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
//     let _diff_result = algorithms::gumtree_hybrid::diff_hybrid(
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

fn test_cd_diff_single<const SIZE_THRESHOLD: usize>(
    name: &str,
    buggy_rel_path: &str,
    fixed_rel_path: &str,
) {
    // Get path to dataset
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(DATASET_PATH);

    let buggy_path = root.join(buggy_rel_path);
    let fixed_path = root.join(fixed_rel_path);

    // Call gumtree java to get number of mappings and edit script length
    // let gumtree_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join(GUMTREE_JAVA_PATH);
    // let mut temp_path = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join(TEMP_FOLDER).join(name);
    // temp_path.set_extension("json");
    // let output = Command::new(gumtree_path)
    //     .args(&[
    //         "textdiff",
    //         buggy_path.to_str().unwrap(),
    //         fixed_path.to_str().unwrap(),
    //         "-m", "gumtree-simple",
    //         "-g", "java-treesitter",
    //         "-d", "chawathe",
    //         "-f", "json",
    //         "-o", temp_path.to_str().expect("Failed to get temp path")
    //     ])
    //     .output()
    //     .expect("Failed to execute gumtree command");
    //
    // if !output.status.success() {
    //     panic!("Gumtree command failed: {}", String::from_utf8_lossy(&output.stderr));
    // }
    //
    // dbg!(String::from_utf8_lossy(&output.stdout));
    // dbg!(String::from_utf8_lossy(&output.stderr));
    // // Parse the GumTree output
    // let (gumtree_matches_len, gumtree_actions_len) = parse_gumtree_output(&temp_path);

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
    let _diff_result = algorithms::gumtree_hybrid::diff_hybrid(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
        100,
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
    let hyperast_matches_len = _diff_result
        .mapper
        .mappings
        .src_to_dst
        .iter()
        .filter(|a| **a != 0)
        .count();

    dbg!(hyperast_actions_len);
    dbg!(hyperast_matches_len);

    todo!()
    //assert_eq!(hyperast_actions_len, gumtree_actions_len);
    //assert_eq!(hyperast_matches_len, gumtree_matches_len);
}

#[test]
fn test_cd_diff_0() {
    let (name, buggy_rel_path, fixed_rel_path) = TEST_CASES[3];

    test_cd_diff_single::<100>(name, buggy_rel_path, fixed_rel_path);
}

#[test]
fn test_cd_diff_1() {
    let (name, buggy_rel_path, fixed_rel_path) = TEST_CASES[1];

    test_cd_diff_single::<100>(name, buggy_rel_path, fixed_rel_path);
}

#[test]
fn test_cd_diff_2() {
    let (name, buggy_rel_path, fixed_rel_path) = TEST_CASES[2];

    test_cd_diff_single::<100>(name, buggy_rel_path, fixed_rel_path);
}

#[test]
fn test_csv1_100() {
    let buggy_rel_path =
        "before/Csv/1/src_main_java_org_apache_commons_csv_ExtendedBufferedReader.java";
    let fixed_rel_path =
        "after/Csv/1/src_main_java_org_apache_commons_csv_ExtendedBufferedReader.java";
    test_cd_diff_single::<100>("csv1", buggy_rel_path, fixed_rel_path);
}

#[test]
fn test_csv1_1000() {
    let buggy_rel_path =
        "before/Csv/1/src_main_java_org_apache_commons_csv_ExtendedBufferedReader.java";
    let fixed_rel_path =
        "after/Csv/1/src_main_java_org_apache_commons_csv_ExtendedBufferedReader.java";
    test_cd_diff_single::<1000>("csv1", buggy_rel_path, fixed_rel_path);
}

#[test]
fn test_mockito_37_100() {
    let buggy_rel_path =
        "before/Mockito/37/src_org_mockito_internal_stubbing_answers_AnswersValidator.java";
    let fixed_rel_path =
        "after/Mockito/37/src_org_mockito_internal_stubbing_answers_AnswersValidator.java";
    test_cd_diff_single::<100>("mockito_37", buggy_rel_path, fixed_rel_path);
}

#[test]
fn test_mockito_37_1000() {
    let buggy_rel_path =
        "before/Mockito/37/src_org_mockito_internal_stubbing_answers_AnswersValidator.java";
    let fixed_rel_path =
        "after/Mockito/37/src_org_mockito_internal_stubbing_answers_AnswersValidator.java";
    test_cd_diff_single::<1000>("mockito_37", buggy_rel_path, fixed_rel_path);
}

#[test]
fn test_cd_diff_3() {
    test_cd_diff_single::<100>(
        "simple_class",
        "../custom/before/simple_class.java",
        "../custom/after/simple_class.java",
    )
}

fn find_java_files(dir: &Path, root: &Path) -> Vec<PathBuf> {
    let mut java_files = Vec::new();

    if dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        java_files.extend(find_java_files(&path, &root));
                    } else if path.extension().and_then(|ext| ext.to_str()) == Some("java") {
                        if let Ok(rel_path) = path.strip_prefix(root) {
                            java_files.push(rel_path.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    java_files
}

#[test]
fn script_find_highest_edit_script_difference() {
    println!("Starting test...");
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(DATASET_PATH);

    let before_dir = root.join("before");
    let java_files = find_java_files(&before_dir, &before_dir);

    let mut max_difference = 0;
    let mut max_path = PathBuf::new();
    let mut max_50 = 0;
    let mut max_3000 = 0;
    let measurements = &mut Vec::new();

    for rel_path in java_files {
        println!("DBG Checking {:?}", rel_path);
        let buggy_path = root.join("before").join(&rel_path);
        let fixed_path = root.join("after").join(&rel_path);

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
        let _diff_result_50 = algorithms::gumtree_hybrid::diff_hybrid::<_>(
            //50, 1>(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
            100,
        );

        let _diff_result_3000 = algorithms::gumtree_hybrid::diff_hybrid::<_>(
            //3000, 1>(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
            100,
        );

        let actions_50 = _diff_result_50.actions.expect("Expected a result");
        let actions_3000 = _diff_result_3000.actions.expect("Expected a result");

        if (actions_3000.len() <= actions_50.len()) {
            let difference = actions_50.len() - actions_3000.len();
            println!(
                "DBG Tried: {:?} / {} ({} - {})",
                &rel_path,
                difference,
                actions_50.len(),
                actions_3000.len()
            );
            measurements.push((
                rel_path.clone(),
                difference,
                actions_50.len(),
                actions_3000.len(),
            ));
            if (difference > max_difference) {
                max_difference = difference;
                max_50 = actions_50.len();
                max_3000 = actions_3000.len();
                max_path = rel_path.clone();
                println!(
                    "DBG !!! New file: {:?} / {} ({} - {})",
                    &rel_path, max_difference, max_50, max_3000
                );
            }
        } else {
            println!(
                "DBG Smaller value for action_50: {} / {}",
                actions_50.len(),
                actions_3000.len()
            );
        }
    }

    println!(
        "DBG Max: {:?} / {} ({} - {})",
        &max_path, max_difference, max_3000, max_50
    );
    println!("DBG: {:?}", measurements);
    dbg!(measurements);
}
