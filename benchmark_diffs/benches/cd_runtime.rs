use criterion::{Criterion, SamplingMode, criterion_group, criterion_main};
use hyper_diff::algorithms;
use hyper_diff::matchers::heuristic::cd::{BottomUpMatcherConfig, LeavesMatcherConfig};
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
        OptimizationConfig::new("Baseline Deep Label", OptimizedDiffConfig::baseline()),
        OptimizationConfig::new(
            "Baseline Statement",
            OptimizedDiffConfig::baseline().with_statement_level_iteration(),
        ),
        OptimizationConfig::new(
            "Baseline Deep Statement",
            OptimizedDiffConfig::baseline()
                .with_statement_level_iteration()
                .with_label_caching()
                .with_deep_leaves(),
        ),
        // Optimized
        OptimizationConfig::new("Optimized Deep Label", OptimizedDiffConfig::optimized()),
        OptimizationConfig::new(
            "Optimized Deep Label Cache",
            OptimizedDiffConfig::optimized().with_label_caching(),
        ),
        OptimizationConfig::new(
            "Optimized with Statement",
            OptimizedDiffConfig::optimized().with_statement_level_iteration(),
        ),
        OptimizationConfig::new(
            "Optimized with Deep Statement",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_deep_leaves(),
        ),
        OptimizationConfig::new(
            "Optimized with Statement and Label Cache",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_label_caching(),
        ),
        OptimizationConfig::new(
            "Optimized with Deep Statement and Label Cache",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_label_caching()
                .with_deep_leaves(),
        ),
    ]
}

fn benchmark_optimized_change_distiller(c: &mut Criterion) {
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

    let mut group = c.benchmark_group("optimized_change_distiller");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);

    let optimization_configs = create_optimization_configs();

    let total_iterations = test_inputs.len() * optimization_configs.len();

    // This lets us skip the first n test cases
    let skip = 0;
    // This lets us only run every nth test case
    let interval = 10;

    let mut iteration = skip;
    let mut interval_counter = 0;

    for (input_idx, input) in test_inputs.iter().enumerate() {
        if input_idx < skip {
            continue;
        }
        if interval_counter == 0 {
            interval_counter = interval;
        } else {
            interval_counter -= 1;
            continue;
        }

        let input = common::preprocess(&(&input.0, &input.1));
        for (opt_idx, opt_config) in optimization_configs.iter().enumerate() {
            iteration += 1;
            println!(
                "Progress: {}/{} (Test case {} of {}, Config {} of {})",
                iteration,
                total_iterations,
                input_idx + 1,
                test_inputs.len(),
                opt_idx + 1,
                optimization_configs.len()
            );
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
