use criterion::{Criterion, SamplingMode, black_box, criterion_group, criterion_main};
use hyper_diff::algorithms;
use hyper_diff::matchers::heuristic::cd::{BottomUpMatcherConfig, LeavesMatcherConfig};
use hyper_diff::{
    OptimizedBottomUpMatcherConfig, OptimizedDiffConfig, OptimizedLeavesMatcherConfig,
};
use hyperast::store::SimpleStores;
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
        // All except for ranged optimisation
        OptimizationConfig::new(
            "All Optimizations except Ranged",
            OptimizedDiffConfig {
                use_lazy_decompression: true,
                use_ranged_similarity: false,
                calculate_script: false,
                leaves_matcher: OptimizedLeavesMatcherConfig::default(),
                bottom_up_matcher: OptimizedBottomUpMatcherConfig::default(),
            },
        ),
        // Only lazy decompression and ranged similarity
        OptimizationConfig::new(
            "Lazy + Ranged Similarity",
            OptimizedDiffConfig {
                use_lazy_decompression: true,
                use_ranged_similarity: true,
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
        // Only type grouping optimizations
        OptimizationConfig::new(
            "Type Grouping Only",
            OptimizedDiffConfig {
                use_lazy_decompression: true,
                use_ranged_similarity: true,
                calculate_script: true,
                leaves_matcher: OptimizedLeavesMatcherConfig {
                    base_config: LeavesMatcherConfig::default(),
                    enable_label_caching: false,
                    enable_type_grouping: true,
                    use_binary_heap: false,
                    reuse_qgram_object: false,
                },
                bottom_up_matcher: OptimizedBottomUpMatcherConfig {
                    base_config: BottomUpMatcherConfig::default(),
                    enable_type_grouping: true,
                    enable_leaf_count_precomputation: false,
                },
            },
        ),
        // Only caching optimizations
        OptimizationConfig::new(
            "Caching Only",
            OptimizedDiffConfig {
                use_lazy_decompression: true,
                use_ranged_similarity: true,
                calculate_script: true,
                leaves_matcher: OptimizedLeavesMatcherConfig {
                    base_config: LeavesMatcherConfig::default(),
                    enable_label_caching: true,
                    enable_type_grouping: false,
                    use_binary_heap: true,
                    reuse_qgram_object: true,
                },
                bottom_up_matcher: OptimizedBottomUpMatcherConfig {
                    base_config: BottomUpMatcherConfig::default(),
                    enable_type_grouping: false,
                    enable_leaf_count_precomputation: true,
                },
            },
        ),
    ]
}

/// Precomputed test data for fair benchmarking
type PrecomputedTestData = Vec<(
    SimpleStores<hyperast_gen_ts_java::types::TStore>,
    hyperast_gen_ts_java::legion_with_refs::NodeIdentifier,
    hyperast_gen_ts_java::legion_with_refs::NodeIdentifier,
)>;

/// Preprocess all test inputs to avoid parsing overhead during benchmarking
fn preprocess_test_inputs(test_inputs: &[(String, String)]) -> PrecomputedTestData {
    test_inputs
        .iter()
        .map(|(src, dst)| {
            let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
            let mut md_cache = Default::default();
            let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, src, dst);

            (
                stores,
                src_tr.local.compressed_node,
                dst_tr.local.compressed_node,
            )
        })
        .collect()
}

fn benchmark_optimized_change_distiller(c: &mut Criterion) {
    // Initialize logging for debugging if needed
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .is_test(true)
        .try_init();

    let test_inputs = hyperast_benchmark_diffs::common::get_test_data_small();
    let total_lines: usize = test_inputs
        .iter()
        .map(|(buggy, _)| buggy.lines().count())
        .sum();

    println!("Running optimized change distiller benchmarks:");
    println!("  - {} test cases", test_inputs.len());
    println!("  - {} total lines of code", total_lines);

    let mut group = c.benchmark_group("optimized_change_distiller");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);

    // Preprocess test inputs once for all benchmarks
    let precomputed_inputs = preprocess_test_inputs(&test_inputs);
    let optimization_configs = create_optimization_configs();

    // Benchmark each optimization configuration
    for opt_config in &optimization_configs {
        let bench_name = format!("Optimized CD - {}", opt_config.name);
        let config = opt_config.config.clone();

        group.bench_function(&bench_name, |b| {
            b.iter(|| {
                for (stores, src_tr, dst_tr) in &precomputed_inputs {
                    let result = algorithms::change_distiller_optimized::diff_optimized(
                        stores,
                        src_tr,
                        dst_tr,
                        config.clone(),
                    );
                    black_box(result);
                }
            })
        });
    }

    // Add comparison with existing lazy_2 implementation
    group.bench_function("ChangeDistiller Lazy 2 (Reference)", |b| {
        b.iter(|| {
            for (stores, src_tr, dst_tr) in &precomputed_inputs {
                let result = algorithms::change_distiller_lazy_2::diff(stores, src_tr, dst_tr);
                black_box(result);
            }
        })
    });

    // Benchmark convenience functions
    group.bench_function("Optimized CD - All Optimizations (convenience)", |b| {
        b.iter(|| {
            for (stores, src_tr, dst_tr) in &precomputed_inputs {
                let result = algorithms::change_distiller_optimized::diff_with_all_optimizations(
                    stores, src_tr, dst_tr,
                );
                black_box(result);
            }
        })
    });

    group.bench_function("Optimized CD - Baseline (convenience)", |b| {
        b.iter(|| {
            for (stores, src_tr, dst_tr) in &precomputed_inputs {
                let result =
                    algorithms::change_distiller_optimized::diff_baseline(stores, src_tr, dst_tr);
                black_box(result);
            }
        })
    });

    group.finish();
}

/// Benchmark script generation performance separately
fn benchmark_script_generation_impact(c: &mut Criterion) {
    let test_inputs = hyperast_benchmark_diffs::common::get_test_data_small();
    let precomputed_inputs = preprocess_test_inputs(&test_inputs);

    let mut group = c.benchmark_group("script_generation_impact");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);

    // Test with script generation enabled
    let config_with_script = OptimizedDiffConfig {
        calculate_script: true,
        ..OptimizedDiffConfig::default()
    };

    group.bench_function("With Script Generation", |b| {
        b.iter(|| {
            for (stores, src_tr, dst_tr) in &precomputed_inputs {
                let result = algorithms::change_distiller_optimized::diff_optimized(
                    stores,
                    src_tr,
                    dst_tr,
                    config_with_script.clone(),
                );
                black_box(result);
            }
        })
    });

    // Test with script generation disabled
    let config_without_script = OptimizedDiffConfig {
        calculate_script: false,
        ..OptimizedDiffConfig::default()
    };

    group.bench_function("Without Script Generation", |b| {
        b.iter(|| {
            for (stores, src_tr, dst_tr) in &precomputed_inputs {
                let result = algorithms::change_distiller_optimized::diff_optimized(
                    stores,
                    src_tr,
                    dst_tr,
                    config_without_script.clone(),
                );
                black_box(result);
            }
        })
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().configure_from_args();
    targets = benchmark_optimized_change_distiller, benchmark_script_generation_impact
}

criterion_main!(benches);
