use clap::{Parser, command};
use hyper_diff::algorithms;
use hyper_diff::matchers::heuristic::cd::{
    BottomUpMatcherConfig, BottomUpMatcherMetrics, CDResult, DiffResultSummary,
    LeavesMatcherConfig, LeavesMatcherMetrics,
};
use hyper_diff::{
    OptimizedBottomUpMatcherConfig, OptimizedDiffConfig, OptimizedLeavesMatcherConfig,
};
use hyperast_benchmark_diffs::common::{self, Input};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Serialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of measurement runs per configuration/test case combination
    #[arg(short, long, default_value_t = 5)]
    runs: usize,

    /// Number of warmup runs (not measured) per configuration/test case combination
    #[arg(short, long, default_value_t = 1)]
    warmup: usize,

    /// Skip first N test cases
    #[arg(long, default_value_t = 0)]
    skip: usize,

    /// Only run every Nth test case (for sampling)
    #[arg(long, default_value_t = 1)]
    interval: usize,

    /// Output directory for results
    #[arg(short, long, default_value = "benchmark_results")]
    output_dir: String,

    /// Additional tag to include in filename
    #[arg(short, long)]
    tag: Option<String>,
}

/// Configuration for different optimization combinations to benchmark
#[derive(Debug, Clone)]
struct OptimizationConfig {
    name: &'static str,
    config: OptimizedDiffConfig,
}

impl OptimizationConfig {
    fn new(name: &'static str, config: OptimizedDiffConfig) -> Self {
        Self { name, config }
    }
}

/// Metadata about the benchmark run
#[derive(Debug, Serialize)]
struct BenchmarkMetadata {
    timestamp: u64,
    runs_per_config: usize,
    warmup_runs: usize,
    total_test_cases: usize,
    total_configurations: usize,
    skip: usize,
    interval: usize,
    tag: Option<String>,
    total_lines_of_code: usize,
    cli_args: Vec<String>,
}

/// Individual measurement result
#[derive(Debug, Serialize)]
struct MeasurementResult {
    // Test case info
    test_case_index: usize,
    file_name: String,
    loc: usize,
    node_count: usize,

    // Configuration info
    config_name: String,
    config_index: usize,

    // Measurement info
    run_index: usize,
    duration_nanos: u64,
    duration_secs: f64,

    // Diff results
    diff_summary: DiffResultSummary,

    // Metadata
    timestamp: u64,
}

/// Create various optimization configurations for comprehensive benchmarking
fn create_optimization_configs() -> Vec<OptimizationConfig> {
    vec![
        // OptimizationConfig::new("Baseline", OptimizedDiffConfig::baseline()),
        OptimizationConfig::new(
            "Baseline Statement",
            OptimizedDiffConfig::baseline().with_statement_level_iteration(true),
        ),
        OptimizationConfig::new(
            "Baseline Deep Statement",
            OptimizedDiffConfig::baseline()
                .with_statement_level_iteration(true)
                .with_label_caching(true)
                .with_deep_leaves(true),
        ),
        OptimizationConfig::new(
            "Optimized with Statement",
            OptimizedDiffConfig::optimized().with_statement_level_iteration(true),
        ),
        OptimizationConfig::new(
            "Optimized with Deep Statement",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration(true)
                .with_deep_leaves(true),
        ),
        OptimizationConfig::new(
            "Optimized with Statement and Label Cache",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration(true)
                .with_label_caching(true),
        ),
        OptimizationConfig::new(
            "Optimized with Deep Statement and Label Cache",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration(true)
                .with_label_caching(true)
                .with_deep_leaves(true),
        ),
    ]
}

fn generate_filename(args: &Args, metadata: &BenchmarkMetadata) -> String {
    let timestamp = chrono::DateTime::from_timestamp(metadata.timestamp as i64, 0)
        .unwrap_or_else(|| chrono::DateTime::from_timestamp(0, 0).unwrap())
        .format("%Y%m%d_%H%M%S");

    let mut filename_parts = vec![
        format!("cd_benchmark_{}", timestamp),
        format!("r{}", args.runs),
    ];

    if args.warmup > 0 {
        filename_parts.push(format!("w{}", args.warmup));
    }

    if args.skip > 0 {
        filename_parts.push(format!("skip{}", args.skip));
    }

    if args.interval > 1 {
        filename_parts.push(format!("int{}", args.interval));
    }

    if let Some(ref tag) = args.tag {
        filename_parts.push(tag.clone());
    }

    filename_parts.push(format!("tc{}", metadata.total_test_cases));
    filename_parts.push(format!("cfg{}", metadata.total_configurations));

    format!("{}.jsonl", filename_parts.join("_"))
}

fn run_single_measurement(
    input: &Input,
    opt_config: &OptimizationConfig,
) -> Result<(Duration, DiffResultSummary), Box<dyn std::error::Error>> {
    let start = Instant::now();

    let cd_result = algorithms::change_distiller_optimized::diff_optimized_verbose(
        &input.stores,
        &input.src,
        &input.dst,
        opt_config.config.clone(),
    );

    let duration = start.elapsed();

    Ok((duration, cd_result.into()))
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    // Initialize logging
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn"))
        .is_test(true)
        .try_init();

    // Create output directory
    std::fs::create_dir_all(&args.output_dir)?;

    // Get test inputs and configurations
    let test_inputs = common::get_all_cases_with_paths();
    let optimization_configs = create_optimization_configs();

    // Calculate filtered test cases
    let filtered_test_cases: Vec<_> = test_inputs
        .iter()
        .enumerate()
        .skip(args.skip)
        .step_by(args.interval)
        .collect();

    let total_lines: usize = filtered_test_cases
        .iter()
        .map(|(_, (_, buggy, _))| buggy.lines().count())
        .sum();

    // Create metadata
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();

    let metadata = BenchmarkMetadata {
        timestamp,
        runs_per_config: args.runs,
        warmup_runs: args.warmup,
        total_test_cases: filtered_test_cases.len(),
        total_configurations: optimization_configs.len(),
        skip: args.skip,
        interval: args.interval,
        tag: args.tag.clone(),
        total_lines_of_code: total_lines,
        cli_args: std::env::args().collect(),
    };

    // Generate filename and create output file
    let filename = generate_filename(&args, &metadata);
    let filepath = std::path::Path::new(&args.output_dir).join(&filename);

    let mut output_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&filepath)?;

    // Write metadata as first line
    writeln!(output_file, "{}", serde_json::to_string(&metadata)?)?;

    // Print summary
    println!("Custom Change Distiller Benchmark");
    println!("=================================");
    println!(
        "Test cases: {} (after filtering)",
        filtered_test_cases.len()
    );
    println!("Configurations: {}", optimization_configs.len());
    println!("Runs per config: {}", args.runs);
    println!("Warmup runs: {}", args.warmup);
    println!("Total lines of code: {}", total_lines);
    println!("Output file: {}", filepath.display());
    println!();

    // Setup progress bar
    let total_operations =
        filtered_test_cases.len() * optimization_configs.len() * (args.warmup + args.runs);
    let progress_bar = ProgressBar::new(total_operations as u64);
    progress_bar.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")?
            .progress_chars("#>-"),
    );

    // Run benchmarks
    for (test_idx, (original_idx, (path, before, after))) in filtered_test_cases.iter().enumerate()
    {
        let input = common::preprocess(&(before, after));

        for (config_idx, opt_config) in optimization_configs.iter().enumerate() {
            // Warmup runs
            for warmup_run in 0..args.warmup {
                progress_bar.set_message(format!(
                    "Warmup {:2}/{:2} - Test {} - Config: {}",
                    warmup_run + 1,
                    args.warmup,
                    test_idx + 1,
                    opt_config.name
                ));

                // Run but don't record
                if let Err(e) = run_single_measurement(&input, opt_config) {
                    eprintln!("Warmup run failed: {}", e);
                }

                progress_bar.inc(1);
            }

            // Measurement runs
            for run_idx in 0..args.runs {
                progress_bar.set_message(format!(
                    "Run    {:2}/{:2} - Test {} - Config: {}",
                    run_idx + 1,
                    args.runs,
                    test_idx + 1,
                    opt_config.name
                ));

                match run_single_measurement(&input, opt_config) {
                    Ok((duration, diff_summary)) => {
                        let measurement = MeasurementResult {
                            test_case_index: *original_idx,
                            file_name: path.clone(),
                            loc: input.loc,
                            node_count: input.node_count,
                            config_name: opt_config.name.to_string(),
                            config_index: config_idx,
                            run_index: run_idx,
                            duration_nanos: duration.as_nanos() as u64,
                            duration_secs: duration.as_secs_f64(),
                            diff_summary,
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        };

                        // Write measurement to file
                        writeln!(output_file, "{}", serde_json::to_string(&measurement)?)?;
                        output_file.flush()?;
                    }
                    Err(e) => {
                        eprintln!(
                            "Measurement failed for test case {} with config {}: {}",
                            test_idx, opt_config.name, e
                        );
                    }
                }

                progress_bar.inc(1);
            }
        }
    }

    progress_bar.finish_with_message("Benchmark completed!");
    println!("\nResults written to: {}", filepath.display());

    Ok(())
}
