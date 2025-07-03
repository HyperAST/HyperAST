use hyper_diff::OptimizedDiffConfig;
use hyper_diff::algorithms::change_distiller_optimized::diff_with_complete_decompression;
use hyper_diff::algorithms::change_distiller_optimized::diff_with_lazy_decompression;
use hyper_diff::matchers::heuristic::cd::DiffResultSummary;
use hyperast_benchmark_diffs::common;

/// Create various optimization configurations for comprehensive benchmarking
fn create_optimization_configs() -> Vec<(&'static str, OptimizedDiffConfig)> {
    vec![
        ("Baseline with Deep Label", OptimizedDiffConfig::baseline()),
        (
            "Baseline with Statement",
            OptimizedDiffConfig::baseline().with_statement_level_iteration(),
        ),
        (
            "Baseline with Deep Statement",
            OptimizedDiffConfig::baseline()
                .with_statement_level_iteration()
                .with_deep_leaves(),
        ),
        // Optimized Label
        (
            "Optimized with Deep Label",
            OptimizedDiffConfig::optimized(),
        ),
        (
            "Optimized with Deep Label and Label Cache",
            OptimizedDiffConfig::optimized().with_label_caching(),
        ),
        // Optimized shallow statements
        (
            "Optimized with Statement",
            OptimizedDiffConfig::optimized().with_statement_level_iteration(),
        ),
        (
            "Optimized with Statement and Ngram Cache",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_ngram_caching(),
        ),
        (
            "Optimized with Statement and Label Cache",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_label_caching(),
        ),
        // Optimized deep statements
        (
            "Optimized with Deep Statement",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_deep_leaves(),
        ),
        (
            "Optimized with Deep Statement and Ngram Cache",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_deep_leaves()
                .with_ngram_caching(),
        ),
        (
            "Optimized with Deep Statement and Label Cache",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_label_caching()
                .with_deep_leaves(),
        ),
    ]
}

fn main() {
    // Initialize logging for debugging if needed
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .is_test(true)
        .try_init();

    let args: Vec<String> = std::env::args().collect();

    let test_inputs = {
        if !args.is_empty() {
            // All args are going to be file paths. For each, if it's an absolute path, use as-is.
            // If it's a relative path, leave as-is. For each, strip "defects4j/before" or "defects4j/after"
            // from the paths if present, so that the paths are relative from there.
            let paths: Vec<String> = args
                .iter()
                .skip(1)
                .map(|arg| {
                    if let Some(idx) = arg.find("defects4j/before") {
                        let start = idx + "defects4j/before".len();
                        arg[start..].trim_start_matches('/').to_string()
                    } else if let Some(idx) = arg.find("defects4j/after") {
                        let start = idx + "defects4j/after".len();
                        arg[start..].trim_start_matches('/').to_string()
                    } else {
                        arg.to_string()
                    }
                })
                .collect();
            let path_refs: Vec<&str> = paths.iter().map(|s| s.as_str()).collect();
            common::read_tests_data(&path_refs)
        } else {
            common::read_test_data_small()
        }
    };
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
        let input = common::preprocess_file_pair([&input.0, &input.1]);
        for (opt_idx, (config_name, config)) in optimization_configs.iter().copied().enumerate() {
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
                "CD Single - {} - {} lines {} nodes",
                config_name, input.lines, input.node_count,
            );
            let result = if config.use_lazy_decompression {
                diff_with_lazy_decompression(&input.stores, &input.src, &input.dst, config)
            } else {
                diff_with_complete_decompression(&input.stores, &input.src, &input.dst, config)
            };
            let summary: DiffResultSummary = result.into();
            println!("Result: {:#?}\n\n", summary);
        }
    }
}
