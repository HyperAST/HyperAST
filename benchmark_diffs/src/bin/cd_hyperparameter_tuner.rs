use criterion::black_box;
use hyper_diff::algorithms;
use hyper_diff::matchers::heuristic::cd::{BottomUpMatcherConfig, LeavesMatcherConfig};
use hyper_diff::matchers::mapping_store::MappingStore;
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::common;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use std::collections::HashMap;
use std::time::Instant;
use tabled::settings::object::Columns;
use tabled::settings::{Alignment, Modify, Style};
use tabled::{Table, Tabled};

// Import necessary modules for running the diff algorithm
use std::fmt::Debug;

#[derive(Debug, Clone)]
struct HyperparamConfig {
    // Leaves Matcher Config
    label_sim_threshold: f64,

    // Bottom-Up Matcher Config
    max_leaves: usize,
    sim_threshold_large_trees: f64,
    sim_threshold_small_trees: f64,
}

impl HyperparamConfig {
    fn to_configs(&self) -> (BottomUpMatcherConfig, LeavesMatcherConfig) {
        let bottom_up_config = BottomUpMatcherConfig {
            max_leaves: self.max_leaves,
            sim_threshold_large_trees: self.sim_threshold_large_trees,
            sim_threshold_small_trees: self.sim_threshold_small_trees,
        };

        let leaves_config = LeavesMatcherConfig {
            label_sim_threshold: self.label_sim_threshold,
        };

        (bottom_up_config, leaves_config)
    }

    fn description(&self) -> String {
        format!(
            "label_sim: {:.2}, max_leaves: {}, large_sim: {:.2}, small_sim: {:.2}",
            self.label_sim_threshold,
            self.max_leaves,
            self.sim_threshold_large_trees,
            self.sim_threshold_small_trees
        )
    }
}

#[derive(Tabled)]
struct BenchmarkResult {
    #[tabled(rename = "Config")]
    config: String,
    #[tabled(rename = "Subtree Prepare (s)")]
    subtree_prepare: String,
    #[tabled(rename = "Leaves Matcher (s)")]
    leaves_matcher: String,
    #[tabled(rename = "Bottom-Up Matcher (s)")]
    bottomup_matcher: String,
    #[tabled(rename = "Prepare Gen (s)")]
    prepare_gen: String,
    #[tabled(rename = "Gen (s)")]
    generation: String,
    #[tabled(rename = "Total Time (s)")]
    total_time: String,
    #[tabled(rename = "Mappings")]
    total_mappings: String,
}

#[derive(Debug, Default)]
struct BenchmarkStats {
    subtree_prepare: Vec<f64>,
    leaves_matcher: Vec<f64>,
    bottomup_matcher: Vec<f64>,
    prepare_gen: Vec<f64>,
    generation: Vec<f64>,
    total_time: Vec<f64>,
    total_mappings: Vec<usize>,
}

impl BenchmarkStats {
    fn add_run(&mut self, result: &RunResult) {
        self.subtree_prepare.push(result.subtree_prepare);
        self.leaves_matcher.push(result.leaves_matcher);
        self.bottomup_matcher.push(result.bottomup_matcher);
        self.prepare_gen.push(result.prepare_gen);
        self.generation.push(result.generation);
        self.total_time.push(result.total_time);
        self.total_mappings.push(result.total_mappings);
    }

    fn average(&self) -> BenchmarkResult {
        let avg = |values: &[f64]| -> String {
            if values.is_empty() {
                "N/A".to_string()
            } else {
                format!("{:.4}", values.iter().sum::<f64>() / values.len() as f64)
            }
        };

        let avg_usize = |values: &[usize]| -> String {
            if values.is_empty() {
                "N/A".to_string()
            } else {
                format!("{}", values.iter().sum::<usize>() / values.len())
            }
        };

        BenchmarkResult {
            config: "".to_string(), // Will be set by caller
            subtree_prepare: avg(&self.subtree_prepare),
            leaves_matcher: avg(&self.leaves_matcher),
            bottomup_matcher: avg(&self.bottomup_matcher),
            prepare_gen: avg(&self.prepare_gen),
            generation: avg(&self.generation),
            total_time: avg(&self.total_time),
            total_mappings: avg_usize(&self.total_mappings),
        }
    }
}

struct RunResult {
    subtree_prepare: f64,
    leaves_matcher: f64,
    bottomup_matcher: f64,
    prepare_gen: f64,
    generation: f64,
    total_time: f64,
    total_mappings: usize,
}

fn run_benchmark(
    test_inputs: &[(String, String)],
    config: &HyperparamConfig,
    iteration: usize,
) -> RunResult {
    let (bottom_up_config, leaves_config) = config.to_configs();

    let start_time = Instant::now();
    let mut total_subtree_prepare = 0.0;
    let mut total_leaves_matcher = 0.0;
    let mut total_bottomup_matcher = 0.0;
    let mut total_prepare_gen = 0.0;
    let mut total_gen = 0.0;
    let mut total_leaves_mappings = 0;
    let mut total_mappings = 0;

    for (buggy, fixed) in test_inputs {
        let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
        let mut md_cache = Default::default();

        // Parse the two Java files
        let (src_tr, dst_tr) = parse_string_pair(
            &mut stores,
            &mut md_cache,
            black_box(buggy),
            black_box(fixed),
        );

        let diff_result = algorithms::change_distiller_lazy_2::diff_with_config(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
            false,
            bottom_up_config.clone(),
            leaves_config.clone(),
        );

        // Extract timing information
        let preparation = diff_result.mapping_durations.preparation[0];
        let leaves_matcher_time = diff_result.mapping_durations.mappings.0[0];
        let bottomup_matcher_time = diff_result.mapping_durations.mappings.0[1];
        let prepare_gen_time = diff_result.prepare_gen_t;
        let gen_time = diff_result.gen_t;

        // Extract mapping information after leaves matcher and after bottom-up matcher
        // Leaf mappings count would need to be stored separately if we need it,
        // for now we'll use the mapper's total mappings count after completion
        let total_file_mappings = diff_result.mapper.mappings.len();

        // Accumulate statistics
        total_subtree_prepare += preparation;
        total_leaves_matcher += leaves_matcher_time;
        total_bottomup_matcher += bottomup_matcher_time;
        total_prepare_gen += prepare_gen_time;
        total_gen += gen_time;
        // Use the same value for leaves_mappings for now (this is not accurate)
        total_leaves_mappings += 0; // Placeholder - actual leaf mappings count not available
        total_mappings += total_file_mappings;

        black_box(diff_result);
    }

    let total_time = start_time.elapsed().as_secs_f64();

    // Create result for this run
    RunResult {
        subtree_prepare: total_subtree_prepare,
        leaves_matcher: total_leaves_matcher,
        bottomup_matcher: total_bottomup_matcher,
        prepare_gen: total_prepare_gen,
        generation: total_gen,
        total_time,
        total_mappings: total_mappings,
    }
}

fn print_run_result(config: &HyperparamConfig, result: &RunResult, iteration: usize) {
    println!(
        "\nResults for iteration {} with config: {}",
        iteration,
        config.description()
    );

    let table_data = vec![BenchmarkResult {
        config: config.description(),
        subtree_prepare: format!("{:.4}", result.subtree_prepare),
        leaves_matcher: format!("{:.4}", result.leaves_matcher),
        bottomup_matcher: format!("{:.4}", result.bottomup_matcher),
        prepare_gen: format!("{:.4}", result.prepare_gen),
        generation: format!("{:.4}", result.generation),
        total_time: format!("{:.4}", result.total_time),
        total_mappings: format!("{}", result.total_mappings),
    }];

    let mut table = Table::new(table_data);
    table
        .with(Style::rounded())
        .with(Modify::new(Columns::new(1..)).with(Alignment::right()));

    println!("{}", table);
}

fn print_final_results(config_results: &HashMap<String, BenchmarkStats>) {
    println!("\n=== Final Results (Averaged over iterations) ===");

    let mut table_data = Vec::new();

    for (config_desc, stats) in config_results {
        let mut result = stats.average();
        result.config = config_desc.clone();
        table_data.push(result);
    }

    // Sort by total time (fastest first)
    table_data.sort_by(|a, b| {
        let a_time = a.total_time.parse::<f64>().unwrap_or(f64::MAX);
        let b_time = b.total_time.parse::<f64>().unwrap_or(f64::MAX);
        a_time.partial_cmp(&b_time).unwrap()
    });

    let mut table = Table::new(table_data);
    table
        .with(Style::rounded())
        .with(Modify::new(Columns::new(1..)).with(Alignment::right()));

    println!("{}", table);
}

fn main() {
    // Setup logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .is_test(true)
        .init();

    println!("Hyperparameter Tuning for Change Distiller Lazy 2");
    println!("==================================================\n");

    let iterations = 3;

    // Get test data
    let test_inputs = common::get_test_data_mixed();
    println!("Using {} test cases for evaluation...", test_inputs.len());
    println!(
        "Total lines of code: {}",
        test_inputs
            .iter()
            .map(|(buggy, _)| buggy.lines().count())
            .sum::<usize>()
    );

    // Define hyperparameter configurations to test
    let configs = vec![
        // Default configuration
        HyperparamConfig {
            label_sim_threshold: 0.5,
            max_leaves: 4,
            sim_threshold_large_trees: 0.6,
            sim_threshold_small_trees: 0.4,
        },
        // Variations of label_sim_threshold
        HyperparamConfig {
            label_sim_threshold: 0.3,
            max_leaves: 4,
            sim_threshold_large_trees: 0.6,
            sim_threshold_small_trees: 0.4,
        },
        HyperparamConfig {
            label_sim_threshold: 0.7,
            max_leaves: 4,
            sim_threshold_large_trees: 0.6,
            sim_threshold_small_trees: 0.4,
        },
        // Variations of max_leaves
        HyperparamConfig {
            label_sim_threshold: 0.5,
            max_leaves: 2,
            sim_threshold_large_trees: 0.6,
            sim_threshold_small_trees: 0.4,
        },
        HyperparamConfig {
            label_sim_threshold: 0.5,
            max_leaves: 6,
            sim_threshold_large_trees: 0.6,
            sim_threshold_small_trees: 0.4,
        },
        // Variations of sim_threshold_large_trees
        HyperparamConfig {
            label_sim_threshold: 0.5,
            max_leaves: 4,
            sim_threshold_large_trees: 0.4,
            sim_threshold_small_trees: 0.4,
        },
        HyperparamConfig {
            label_sim_threshold: 0.5,
            max_leaves: 4,
            sim_threshold_large_trees: 0.8,
            sim_threshold_small_trees: 0.4,
        },
        // Variations of sim_threshold_small_trees
        HyperparamConfig {
            label_sim_threshold: 0.5,
            max_leaves: 4,
            sim_threshold_large_trees: 0.6,
            sim_threshold_small_trees: 0.2,
        },
        HyperparamConfig {
            label_sim_threshold: 0.5,
            max_leaves: 4,
            sim_threshold_large_trees: 0.6,
            sim_threshold_small_trees: 0.6,
        },
        // Some combined variations
        HyperparamConfig {
            label_sim_threshold: 0.7,
            max_leaves: 6,
            sim_threshold_large_trees: 0.7,
            sim_threshold_small_trees: 0.3,
        },
        HyperparamConfig {
            label_sim_threshold: 0.3,
            max_leaves: 2,
            sim_threshold_large_trees: 0.5,
            sim_threshold_small_trees: 0.5,
        },
    ];

    println!(
        "Testing {} different hyperparameter configurations",
        configs.len()
    );
    println!("Running 3 iterations for each configuration\n");

    // Store results
    let mut config_results: HashMap<String, BenchmarkStats> = HashMap::new();

    // Run benchmarks for each configuration
    for config in &configs {
        let config_desc = config.description();
        println!("\n=== Testing configuration: {} ===", config_desc);

        let mut stats = BenchmarkStats::default();

        // Run multiple iterations
        for i in 1..=iterations {
            println!("  Running iteration {}/3...", i);
            let result = run_benchmark(&test_inputs, config, i);

            // Print intermediate result
            print_run_result(config, &result, i);

            // Store result
            stats.add_run(&result);
        }

        // Store average results for this configuration
        config_results.insert(config_desc, stats);
    }

    // Print final results
    print_final_results(&config_results);
}
