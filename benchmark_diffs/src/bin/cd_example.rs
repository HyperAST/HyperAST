use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use hyper_diff::algorithms;
use hyper_diff::matchers::heuristic::cd::{
    BottomUpMatcherConfig, DiffResultSummary, LeavesMatcherConfig,
};
use hyper_diff::{
    OptimizedBottomUpMatcherConfig, OptimizedDiffConfig, OptimizedLeavesMatcherConfig,
};
use hyperast_benchmark_diffs::common;

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
                calculate_script: false,
                leaves_matcher: OptimizedLeavesMatcherConfig {
                    base_config: LeavesMatcherConfig::default(),
                    enable_label_caching: false,
                    enable_type_grouping: false,
                    statement_level_iteration: false,
                    use_binary_heap: false,
                    reuse_qgram_object: false,
                },
                bottom_up_matcher: OptimizedBottomUpMatcherConfig {
                    base_config: BottomUpMatcherConfig::default(),
                    enable_type_grouping: false,
                    statement_level_iteration: false,
                    enable_leaf_count_precomputation: false,
                },
            },
        ),
        OptimizationConfig::new(
            "Lazy Fine grained",
            OptimizedDiffConfig {
                use_lazy_decompression: true,
                use_ranged_similarity: true,
                calculate_script: false,
                leaves_matcher: OptimizedLeavesMatcherConfig {
                    base_config: LeavesMatcherConfig::default(),
                    enable_label_caching: false,
                    enable_type_grouping: false,
                    statement_level_iteration: false,
                    use_binary_heap: false,
                    reuse_qgram_object: false,
                },
                bottom_up_matcher: OptimizedBottomUpMatcherConfig {
                    base_config: BottomUpMatcherConfig::default(),
                    enable_type_grouping: false,
                    statement_level_iteration: true,
                    enable_leaf_count_precomputation: false,
                },
            },
        ),
        OptimizationConfig::new(
            "Lazy Statement Label Cache",
            OptimizedDiffConfig {
                use_lazy_decompression: true,
                use_ranged_similarity: true,
                calculate_script: false,
                leaves_matcher: OptimizedLeavesMatcherConfig {
                    base_config: LeavesMatcherConfig::default(),
                    enable_label_caching: true,
                    enable_type_grouping: false,
                    statement_level_iteration: true,
                    use_binary_heap: false,
                    reuse_qgram_object: false,
                },
                bottom_up_matcher: OptimizedBottomUpMatcherConfig {
                    base_config: BottomUpMatcherConfig::default(),
                    statement_level_iteration: true,
                    enable_type_grouping: false,
                    enable_leaf_count_precomputation: true,
                },
            },
        ),
        OptimizationConfig::new(
            "Lazy Statement",
            OptimizedDiffConfig {
                use_lazy_decompression: true,
                use_ranged_similarity: true,
                calculate_script: false,
                leaves_matcher: OptimizedLeavesMatcherConfig {
                    base_config: LeavesMatcherConfig::default(),
                    enable_label_caching: false,
                    enable_type_grouping: false,
                    statement_level_iteration: true,
                    use_binary_heap: false,
                    reuse_qgram_object: false,
                },
                bottom_up_matcher: OptimizedBottomUpMatcherConfig {
                    base_config: BottomUpMatcherConfig::default(),
                    statement_level_iteration: true,
                    enable_type_grouping: false,
                    enable_leaf_count_precomputation: true,
                },
            },
        ),
        OptimizationConfig::new(
            "Lazy Grouping",
            OptimizedDiffConfig {
                use_lazy_decompression: true,
                use_ranged_similarity: true,
                calculate_script: false,
                leaves_matcher: OptimizedLeavesMatcherConfig {
                    base_config: LeavesMatcherConfig::default(),
                    enable_label_caching: true,
                    enable_type_grouping: true,
                    statement_level_iteration: false,
                    use_binary_heap: false,
                    reuse_qgram_object: false,
                },
                bottom_up_matcher: OptimizedBottomUpMatcherConfig {
                    base_config: BottomUpMatcherConfig::default(),
                    statement_level_iteration: false,
                    enable_type_grouping: true,
                    enable_leaf_count_precomputation: true,
                },
            },
        ),
        // All optimizations enabled
        // OptimizationConfig::new("All Optimizations", OptimizedDiffConfig::default()),
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

fn main() {
    // Initialize logging for debugging if needed
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .is_test(true)
        .try_init();

    let test_inputs = common::get_test_data_small();
    common::print_test_case_table(&test_inputs);
    let total_lines: usize = test_inputs
        .iter()
        .map(|(buggy, _)| buggy.lines().count())
        .sum();

    println!("Running optimized change distiller benchmarks:");
    println!("  - {} test cases", test_inputs.len());
    println!("  - {} total lines of code", total_lines);

    let optimization_configs = create_optimization_configs();

    let total_iterations = test_inputs.len() * optimization_configs.len();

    let skip = 0;
    let mut iteration = skip;

    for (input_idx, input) in test_inputs.iter().enumerate() {
        if input_idx < skip {
            continue;
        }
        let input = common::preprocess(&(&input.0, &input.1));
        for (opt_idx, opt_config) in optimization_configs.iter().enumerate() {
            iteration += 1;
            println!("\n\n--------------------------------------------------------------------");
            println!(
                "Progress: {}/{} (Test case {} of {}, Config {} of {})",
                iteration,
                total_iterations,
                input_idx + 1,
                test_inputs.len(),
                opt_idx + 1,
                optimization_configs.len()
            );
            println!(
                "CD Single - {} - {} loc {} nodes",
                opt_config.name, input.loc, input.node_count,
            );
            let result = algorithms::change_distiller_optimized::diff_optimized_verbose(
                &input.stores,
                &input.src,
                &input.dst,
                opt_config.config.clone(),
            );
            let summary: DiffResultSummary = result.into();
            println!("Result: {:#?}\n\n", summary);
        }
    }
}
