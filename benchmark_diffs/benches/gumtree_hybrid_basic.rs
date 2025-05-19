use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use std::path::Path;

const DEFAULT_SIZE_THRESHOLD: usize = 1000;

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

fn diff_benchmark(c: &mut Criterion) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    let mut group = c.benchmark_group("gumtree_hybrid_basic");

    group.sample_size(100);

    // Get path to dataset
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("datasets/defects4j");

    let test_inputs: Vec<_> = TEST_CASES
        .iter()
        .map(|(name, buggy_rel_path, fixed_rel_path)| {
            let buggy_path = root.join(buggy_rel_path);
            let fixed_path = root.join(fixed_rel_path);

            // Read file contents
            let buggy_content = std::fs::read_to_string(&buggy_path)
                .expect(&format!("Failed to read buggy file: {:?}", buggy_path));
            let fixed_content = std::fs::read_to_string(&fixed_path)
                .expect(&format!("Failed to read fixed file: {:?}", fixed_path));

            log::info!(
                "Processing test case: {} with {} lines",
                name,
                buggy_content.lines().count()
            );

            (name, buggy_content, fixed_content)
        })
        .collect();

    group.bench_function("hybrid 100", |b| {
        b.iter(|| {
            let (_name, buggy, fixed) = &test_inputs[5];
            run_diff(buggy, fixed, 100);
        })
    });

    group.finish();
}

fn run_diff(src: &str, dst: &str, max_size: usize) {
    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();

    let (src_tr, dst_tr) =
        parse_string_pair(&mut stores, &mut md_cache, black_box(src), black_box(dst));
    
    let diff_result= algorithms::gumtree_hybrid::diff_hybrid::<_, DEFAULT_SIZE_THRESHOLD>(
        &stores,
        &src_tr.local.compressed_node,
        &dst_tr.local.compressed_node,
    );

    black_box(diff_result);
}

criterion_group!(benches, diff_benchmark);
criterion_main!(benches);