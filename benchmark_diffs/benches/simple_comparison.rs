use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use std::{hint::black_box, path::Path};

// Define the test cases with their paths relative to root/../datasets/defects4j
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

struct Duo<'a> {
    buggy: &'a String,
    fixed: &'a String,
}

impl<'a> Duo<'a> {
    pub fn new(buggy: &'a String, fixed: &'a String) -> Self {
        Duo { buggy, fixed }
    }
}

fn diff_benchmark_hyperdiff(c: &mut Criterion) {
    run_benchmark_group(c, "HyperDiff");
}

fn diff_benchmark_simple(c: &mut Criterion) {
    run_benchmark_group(c, "gumtree_simple");
}

fn diff_benchmark_greedy(c: &mut Criterion) {
    run_benchmark_group(c, "gumtree_greedy");
}

fn run_benchmark_group(c: &mut Criterion, group_name: &str) {
    let mut group = c.benchmark_group("compare greedy and lazy gumtree");
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

    for (name, buggy, fixed) in &test_inputs {
        let duo = Duo::new(buggy, fixed);
        match group_name {
            "HyperDiff" => {
                group.bench_with_input(BenchmarkId::new("HyperDiff", name), &duo, |b, duo| {
                    b.iter(|| {
                        run_diff(duo.buggy, duo.fixed, "gumtree_lazy");
                    });
                });
            }

            "gumtree_greedy" => {
                group.bench_with_input(BenchmarkId::new("gumtree_greedy", name), &duo, |b, duo| {
                    b.iter(|| {
                        run_diff(duo.buggy, duo.fixed, "gumtree_greedy");
                    });
                });
            }

            "gumtree_simple" => {
                group.bench_with_input(BenchmarkId::new("gumtree_simple", name), &duo, |b, duo| {
                    b.iter(|| {
                        run_diff(duo.buggy, duo.fixed, "gumtree_simple");
                    });
                });
            }
            err => panic!("Unknown group name, got: {}", err),
        }
    }

    group.finish();
}

fn run_diff(src: &str, dst: &str, algorithm: &str) {
    // Initialize stores for each iteration
    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();

    // Parse the two Java files
    let (src_tr, dst_tr) =
        parse_string_pair(&mut stores, &mut md_cache, black_box(src), black_box(dst));

    // Perform the diff using specified algorithm
    let diff_result = match algorithm {
        "gumtree_lazy" => algorithms::gumtree_lazy::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        ),
        "gumtree_simple" => algorithms::gumtree_simple::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        ),
        "gumtree_greedy" => algorithms::gumtree::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        ),
        _ => panic!("Unknown diff algorithm"),
    };
    black_box(diff_result);
}

criterion_group!(hyperdiff_group, diff_benchmark_hyperdiff);
criterion_group!(greedy_group, diff_benchmark_greedy);
criterion_group!(simple_group, diff_benchmark_simple);
criterion_main!(hyperdiff_group, greedy_group, simple_group);
