use hyper_diff::algorithms::change_distiller_optimized::{
    diff_with_complete_decompression, diff_with_lazy_decompression,
};
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use std::path::Path;
use std::sync::Once;

static INIT: Once = Once::new();

fn init_logger() {
    INIT.call_once(|| {
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
            .is_test(true)
            .init();
    });
}

const TEST_CASES: &[(&str, &str, &str)] = &[
    // (
    //     "Jsoup_17",
    //     "before/Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java",
    //     "after/Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java",
    // ),
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

#[test]
fn log_all_number_of_lines_of_codes() {
    init_logger();
    for (_name, buggy_rel_path, fixed_rel_path) in TEST_CASES {
        let buggy_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("datasets/defects4j")
            .join(buggy_rel_path);
        let fixed_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("datasets/defects4j")
            .join(fixed_rel_path);

        let buggy_content = std::fs::read_to_string(&buggy_path).unwrap();
        let fixed_content = std::fs::read_to_string(&fixed_path).unwrap();

        log::info!("Processing {}", _name);
        log::info!(
            "Number of lines of code in buggy version: {}",
            buggy_content.lines().count()
        );
        log::info!(
            "Number of lines of code in fixed version: {}",
            fixed_content.lines().count()
        );
    }
}

#[test]
fn test_cd_diff_first() {
    init_logger();
    run_diff_test(&[TEST_CASES[2]]);
}

#[test]
fn test_cd_diff_all() {
    init_logger();
    run_diff_test(TEST_CASES);
}

fn run_diff_test(test_cases: &[(&str, &str, &str)]) {
    for (_name, buggy_rel_path, fixed_rel_path) in test_cases {
        // Get path to dataset
        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .join("datasets/defects4j");

        let buggy_path = root.join(buggy_rel_path);
        let fixed_path = root.join(fixed_rel_path);

        // Read file contents
        let buggy_content = std::fs::read_to_string(&buggy_path)
            .expect(&format!("Failed to read buggy file: {:?}", buggy_path));
        let fixed_content = std::fs::read_to_string(&fixed_path)
            .expect(&format!("Failed to read fixed file: {:?}", fixed_path));

        let lines_buggy = buggy_content.lines().count();
        let lines_fixed = fixed_content.lines().count();

        log::info!(
            "Loaded files with paths: {:?} ({} loc) and {:?} ({})",
            buggy_path,
            lines_buggy,
            fixed_path,
            lines_fixed
        );

        // Initialize stores
        let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
        let mut md_cache = Default::default();

        // Parse the two Java files
        let [src_tr, dst_tr] =
            parse_string_pair(&mut stores, &mut md_cache, &buggy_content, &fixed_content);

        let config = hyper_diff::OptimizedDiffConfig::default();
        // Perform the diff
        let diff_result = if config.use_lazy_decompression {
            diff_with_lazy_decompression(
                &stores,
                &src_tr.local.compressed_node,
                &dst_tr.local.compressed_node,
                config,
            )
        } else {
            diff_with_complete_decompression(
                &stores,
                &src_tr.local.compressed_node,
                &dst_tr.local.compressed_node,
                config,
            )
        }
        .into_diff_result();

        // println!(
        //     "Src Tree:\n{}",
        //     SyntaxSerializer::new(&stores, src_tr.local.compressed_node)
        // );
        // println!(
        //     "Dst Tree:\n{}",
        //     SyntaxSerializer::new(&stores, dst_tr.local.compressed_node)
        // );

        // println!("Mappings:");
        // println!("{:#?}", diff_result.mapper.mappings);

        // if let Some(actions) = diff_result.actions.as_ref() {
        //     actions_vec_f(
        //         &actions,
        //         &diff_result.mapper.hyperast,
        //         src_tr.local.compressed_node.as_id().clone(),
        //     );
        //     actions.iter().for_each(|a| println!("{:?}", a));
        // }

        println!("stats from diffing: \n{:#?}", &diff_result.summarize());
    }

    assert!(false)
}
