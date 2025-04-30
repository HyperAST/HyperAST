use criterion::{black_box, criterion_group, criterion_main, Criterion};
use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use std::path::Path;

// Define the test cases with their paths relative to root/../datasets/defects4j
const TEST_CASES: &[(&str, &str, &str)] = &[

    (
        "Jsoup_17",
        "before/Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java",
        "after/Jsoup/17/src_main_java_org_jsoup_parser_TreeBuilderState.java"
    ),
    (
        "JacksonDatabind_25",
        "before/JacksonDatabind/25/src_main_java_com_fasterxml_jackson_databind_module_SimpleAbstractTypeResolver.java",
        "after/JacksonDatabind/25/src_main_java_com_fasterxml_jackson_databind_module_SimpleAbstractTypeResolver.java"
    ),
    (
        "Chart_19",
        "before/Chart/19/source_org_jfree_chart_plot_CategoryPlot.java",
        "after/Chart/19/source_org_jfree_chart_plot_CategoryPlot.java"
    ),
];

fn diff_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("diff_tests");

    // Get path to dataset
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("datasets/defects4j");

    for (name, buggy_rel_path, fixed_rel_path) in TEST_CASES {
        let buggy_path = root.join(buggy_rel_path);
        let fixed_path = root.join(fixed_rel_path);

        // Read file contents
        let buggy_content = std::fs::read_to_string(&buggy_path)
            .expect(&format!("Failed to read buggy file: {:?}", buggy_path));
        let fixed_content = std::fs::read_to_string(&fixed_path)
            .expect(&format!("Failed to read fixed file: {:?}", fixed_path));

        group
            .sample_size(10)
            .bench_function(format!("hyperdiff_lazy_{}", name), |b| {
                b.iter(|| {
                    // Initialize stores for each iteration
                    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
                    let mut md_cache = Default::default();

                    // Parse the two Java files
                    let (src_tr, dst_tr) = parse_string_pair(
                        &mut stores,
                        &mut md_cache,
                        black_box(&buggy_content),
                        black_box(&fixed_content),
                    );

                    // Perform the diff using gumtree lazy
                    let diff_result = algorithms::gumtree_lazy::diff(
                        &stores,
                        &src_tr.local.compressed_node,
                        &dst_tr.local.compressed_node,
                    );

                    black_box(diff_result);
                })
            });
    }

    group.finish();
}

criterion_group!(benches, diff_benchmark);
criterion_main!(benches);
