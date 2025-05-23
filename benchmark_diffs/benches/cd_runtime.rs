use std::cmp::max;

use criterion::{BenchmarkId, Criterion, SamplingMode, black_box, criterion_group, criterion_main};
use hyper_diff::algorithms;
use hyper_diff::matchers::heuristic::cd::{BottomUpMatcherConfig, LeavesMatcherConfig};
use hyper_diff::{
    OptimizedBottomUpMatcherConfig, OptimizedDiffConfig, OptimizedLeavesMatcherConfig,
};
use hyperast::store::SimpleStores;
use hyperast::types::HyperAST;
use hyperast_benchmark_diffs::common;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;

/// Configuration for different optimization combinations to benchmark
struct OptimizationConfig {
    name: &'static str,
    config: OptimizedDiffConfig,
}

impl OptimizationConfig {
    fn new(name: &'static str, config: OptimizedDiffConfig) -> Self {
        Self { name, config }
    }
}

/// Create various optimization configurations for comprehensive benchmarking
fn create_optimization_configs() -> Vec<OptimizationConfig> {
    vec![
        // Baseline: No optimizations (equivalent to original change_distiller)
        OptimizationConfig::new(
            "Baseline",
            OptimizedDiffConfig {
                use_lazy_decompression: false,
                use_ranged_similarity: false,
                calculate_script: true,
                leaves_matcher: OptimizedLeavesMatcherConfig {
                    base_config: LeavesMatcherConfig::default(),
                    enable_label_caching: false,
                    enable_type_grouping: false,
                    use_binary_heap: false,
                    reuse_qgram_object: false,
                },
                bottom_up_matcher: OptimizedBottomUpMatcherConfig {
                    base_config: BottomUpMatcherConfig::default(),
                    enable_type_grouping: false,
                    enable_leaf_count_precomputation: false,
                },
            },
        ),
        // All optimizations enabled
        OptimizationConfig::new("All Optimizations", OptimizedDiffConfig::default()),
        // // All except for ranged optimisation
        // OptimizationConfig::new(
        //     "All Optimizations except Ranged",
        //     OptimizedDiffConfig {
        //         use_lazy_decompression: true,
        //         use_ranged_similarity: false,
        //         calculate_script: false,
        //         leaves_matcher: OptimizedLeavesMatcherConfig::default(),
        //         bottom_up_matcher: OptimizedBottomUpMatcherConfig::default(),
        //     },
        // ),
        // // Only lazy decompression and ranged similarity
        // OptimizationConfig::new(
        //     "Lazy + Ranged Similarity",
        //     OptimizedDiffConfig {
        //         use_lazy_decompression: true,
        //         use_ranged_similarity: true,
        //         calculate_script: true,
        //         leaves_matcher: OptimizedLeavesMatcherConfig {
        //             base_config: LeavesMatcherConfig::default(),
        //             enable_label_caching: false,
        //             enable_type_grouping: false,
        //             use_binary_heap: false,
        //             reuse_qgram_object: false,
        //         },
        //         bottom_up_matcher: OptimizedBottomUpMatcherConfig {
        //             base_config: BottomUpMatcherConfig::default(),
        //             enable_type_grouping: false,
        //             enable_leaf_count_precomputation: false,
        //         },
        //     },
        // ),
        // // Only type grouping optimizations
        // OptimizationConfig::new(
        //     "Type Grouping Only",
        //     OptimizedDiffConfig {
        //         use_lazy_decompression: true,
        //         use_ranged_similarity: true,
        //         calculate_script: true,
        //         leaves_matcher: OptimizedLeavesMatcherConfig {
        //             base_config: LeavesMatcherConfig::default(),
        //             enable_label_caching: false,
        //             enable_type_grouping: true,
        //             use_binary_heap: false,
        //             reuse_qgram_object: false,
        //         },
        //         bottom_up_matcher: OptimizedBottomUpMatcherConfig {
        //             base_config: BottomUpMatcherConfig::default(),
        //             enable_type_grouping: true,
        //             enable_leaf_count_precomputation: false,
        //         },
        //     },
        // ),
        // // Only caching optimizations
        // OptimizationConfig::new(
        //     "Caching Only",
        //     OptimizedDiffConfig {
        //         use_lazy_decompression: true,
        //         use_ranged_similarity: true,
        //         calculate_script: true,
        //         leaves_matcher: OptimizedLeavesMatcherConfig {
        //             base_config: LeavesMatcherConfig::default(),
        //             enable_label_caching: true,
        //             enable_type_grouping: false,
        //             use_binary_heap: true,
        //             reuse_qgram_object: true,
        //         },
        //         bottom_up_matcher: OptimizedBottomUpMatcherConfig {
        //             base_config: BottomUpMatcherConfig::default(),
        //             enable_type_grouping: false,
        //             enable_leaf_count_precomputation: true,
        //         },
        //     },
        // ),
    ]
}

struct Input {
    stores: SimpleStores<hyperast_gen_ts_java::types::TStore>,
    src: hyperast_gen_ts_java::legion_with_refs::NodeIdentifier,
    dst: hyperast_gen_ts_java::legion_with_refs::NodeIdentifier,
    loc: usize,
    node_count: usize,
}

/// Preprocesses all test inputs to avoid parsing overhead during benchmarking.
fn preprocess_test_inputs(test_inputs: &[(String, String)]) -> Vec<Input> {
    test_inputs.iter().map(|input| preprocess(input)).collect()
}

fn preprocess(input: &(String, String)) -> Input {
    let (src, dst) = input;
    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();
    let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, src, dst);
    let loc = max(src.lines().count(), dst.lines().count());
    let node_count = stores.node_store().len();

    Input {
        stores,
        src: src_tr.local.compressed_node,
        dst: dst_tr.local.compressed_node,
        loc,
        node_count,
    }
}

fn benchmark_optimized_change_distiller(c: &mut Criterion) {
    // Initialize logging for debugging if needed
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .is_test(true)
        .try_init();

    let test_inputs = common::get_test_data_mixed();
    let input_count = test_inputs.len();
    common::print_test_case_table(&test_inputs);
    let total_lines: usize = test_inputs
        .iter()
        .map(|(buggy, _)| buggy.lines().count())
        .sum();

    println!("Running optimized change distiller benchmarks:");
    println!("  - {} test cases", test_inputs.len());
    println!("  - {} total lines of code", total_lines);

    let mut group = c.benchmark_group("optimized_change_distiller");
    group.sample_size(10);
    // group.sampling_mode(SamplingMode::Flat);

    let optimization_configs = create_optimization_configs();

    for input in &test_inputs {
        let input = preprocess(input);
        for opt_config in &optimization_configs {
            group.bench_with_input(
                format!(
                    "CD Single - {} - {} loc {} nodes",
                    opt_config.name, input.loc, input.node_count,
                ),
                &input,
                |b, input| {
                    b.iter(|| {
                        algorithms::change_distiller_optimized::diff_optimized(
                            &input.stores,
                            &input.src,
                            &input.dst,
                            opt_config.config.clone(),
                        )
                    });
                },
            );
        }
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().configure_from_args();
    targets = benchmark_optimized_change_distiller,
}

criterion_main!(benches);
