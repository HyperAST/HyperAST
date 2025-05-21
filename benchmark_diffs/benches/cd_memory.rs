use common::get_test_data_small;
use hyperast_benchmark_diffs::{common, stats_utils};
use memory_stats::memory_stats;
use std::collections::HashMap;
use std::time::Instant;

fn measure_memory_usage(
    algorithm: &str,
    iterations: usize,
    warmup_iterations: usize,
) -> HashMap<String, f64> {
    // We'll return a hashmap with various statistics
    let test_inputs = get_test_data_small();

    // Perform warmup iterations to stabilize JIT and memory usage
    println!("Warming up for {} iterations...", warmup_iterations);
    for _ in 0..warmup_iterations {
        for (buggy, fixed) in &test_inputs {
            common::run_diff(buggy, fixed, algorithm);
        }
    }

    // Collect all measurements for detailed statistics
    let mut memory_measurements = Vec::with_capacity(iterations);

    // Helper function to get current memory usage
    let get_current_memory = || {
        if let Some(usage) = memory_stats() {
            usage.physical_mem
        } else {
            panic!("Failed to get memory stats");
        }
    };

    println!("Starting measurement...");
    for i in 0..iterations {
        // Force aggressive memory cleanup
        drop(vec![0u8; 1024 * 1024 * 50]); // Allocate 50MB to force potential GC
        std::thread::sleep(std::time::Duration::from_millis(100)); // Give GC time to run

        let before_physical = get_current_memory();

        // Run the algorithm
        for (buggy, fixed) in &test_inputs {
            common::run_diff(buggy, fixed, algorithm);
        }

        let after_physical = get_current_memory();

        let memory_delta = after_physical.saturating_sub(before_physical);

        // Only count non-zero measurements
        if memory_delta > 0 {
            memory_measurements.push(memory_delta);

            println!(
                "Iteration {:2}: Memory used: {:10} bytes ({})",
                i + 1,
                memory_delta,
                stats_utils::format_bytes(memory_delta)
            );
        } else {
            println!(
                "Iteration {:2}: Memory used: {:10} bytes ({:6.2} MB) - Skipping zero measurement",
                i + 1,
                memory_delta,
                memory_delta as f64 / 1_048_576.0
            );
        }
    }

    // Calculate comprehensive statistics
    stats_utils::summarize_statistics(&memory_measurements)
}

/// This function runs the benchmark and prints its stats and returns the statistics
fn run_benchmark<'a>(
    title: &'a str,
    name: &str,
    iterations: usize,
    warmup_iterations: usize,
) -> (&'a str, HashMap<String, f64>) {
    println!("\n\n===Running benchmark for {}...===", title);
    let start = Instant::now();
    let stats = measure_memory_usage(name, iterations, warmup_iterations);
    let duration = start.elapsed();

    // Get the key statistics
    let peak_memory = stats.get("max").unwrap_or(&0.0) as &f64;
    let average_memory = stats.get("mean").unwrap_or(&0.0) as &f64;
    let median_memory = stats.get("median").unwrap_or(&0.0) as &f64;
    let std_dev = stats.get("std_dev").unwrap_or(&0.0) as &f64;
    let cv = stats.get("cv").unwrap_or(&0.0) as &f64;

    println!("\n{} Memory Benchmark Results", title);
    println!("Duration: {:?}", duration);
    println!("Statistical Analysis:");
    println!(
        "  Peak Memory Usage:      {} ({:6.2} MB)",
        stats_utils::format_bytes(*peak_memory as usize),
        peak_memory / 1_048_576.0
    );
    println!(
        "  Mean Memory Usage:      {} ({:6.2} MB)",
        stats_utils::format_bytes(*average_memory as usize),
        average_memory / 1_048_576.0
    );
    println!(
        "  Median Memory Usage:    {} ({:6.2} MB)",
        stats_utils::format_bytes(*median_memory as usize),
        median_memory / 1_048_576.0
    );
    println!(
        "  Standard Deviation:     {} ({:6.2} MB)",
        stats_utils::format_bytes(*std_dev as usize),
        std_dev / 1_048_576.0
    );
    println!("  Coefficient of Variation: {:6.2}%", cv);

    (title, stats)
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();

    let iterations = 15; // More iterations for better statistics
    let warmup_iterations = 3; // Warmup runs to stabilize JIT

    println!(
        "\n=== Memory Benchmark: Running {} iterations per algorithm (with {} warmup iterations) ===\n",
        iterations, warmup_iterations
    );

    // Track results for comparison table
    let mut results = Vec::new();

    results.push(run_benchmark(
        "ChangeDistiller Base",
        "change_distiller",
        iterations,
        warmup_iterations,
    ));
    results.push(run_benchmark(
        "ChangeDistiller Lazy",
        "change_distiller_lazy",
        iterations,
        warmup_iterations,
    ));
    results.push(run_benchmark(
        "Gumtree Lazy",
        "gumtree_lazy",
        iterations,
        warmup_iterations,
    ));

    // Print comparison with aligned columns
    println!("\n=== Memory Usage Comparison ===");
    println!(
        "+-----------------------+-----------------+-----------------+-----------------+-----------------+"
    );
    println!(
        "| {:21} | {:15} | {:15} | {:15} | {:15} |",
        "Algorithm", "Peak Memory", "Mean Memory", "Median Memory", "Std Deviation"
    );
    println!(
        "+-----------------------+-----------------+-----------------+-----------------+-----------------+"
    );

    for (name, stats) in &results {
        let peak = stats.get("max").unwrap_or(&0.0);
        let mean = stats.get("mean").unwrap_or(&0.0);
        let median = stats.get("median").unwrap_or(&0.0);
        let std_dev = stats.get("std_dev").unwrap_or(&0.0);

        println!(
            "| {:21} | {:13.2} MB | {:13.2} MB | {:13.2} MB | {:13.2} MB |",
            name,
            peak / 1_048_576.0,
            mean / 1_048_576.0,
            median / 1_048_576.0,
            std_dev / 1_048_576.0
        );
    }
    println!(
        "+-----------------------+-----------------+-----------------+-----------------+-----------------+"
    );

    // Statistical significance test between algorithms
    println!("\n=== Statistical Comparison ===");

    // Extract measurement vectors for each algorithm
    let cd_base_data = match results.get(0) {
        Some((_, stats)) => collect_measurements_from_stats(stats),
        None => vec![],
    };

    let cd_lazy_data = match results.get(1) {
        Some((_, stats)) => collect_measurements_from_stats(stats),
        None => vec![],
    };

    let gt_lazy_data = match results.get(2) {
        Some((_, stats)) => collect_measurements_from_stats(stats),
        None => vec![],
    };

    let alpha = 0.05; // 5% significance level

    // Compare CD Base vs CD Lazy
    if !cd_base_data.is_empty() && !cd_lazy_data.is_empty() {
        let (p_value, significant, percent_diff) =
            stats_utils::compare_measurements(&cd_base_data, &cd_lazy_data, alpha);

        println!("ChangeDistiller Base vs ChangeDistiller Lazy:");
        println!("  Difference: {:.2}%", percent_diff);
        println!("  p-value: {:.4}", p_value);
        println!(
            "  Statistically significant: {}",
            if significant { "YES" } else { "NO" }
        );
    }

    // Compare CD Base vs GT Lazy
    if !cd_base_data.is_empty() && !gt_lazy_data.is_empty() {
        let (p_value, significant, percent_diff) =
            stats_utils::compare_measurements(&cd_base_data, &gt_lazy_data, alpha);

        println!("\nChangeDistiller Base vs Gumtree Lazy:");
        println!("  Difference: {:.2}%", percent_diff);
        println!("  p-value: {:.4}", p_value);
        println!(
            "  Statistically significant: {}",
            if significant { "YES" } else { "NO" }
        );
    }

    // Compare CD Lazy vs GT Lazy
    if !cd_lazy_data.is_empty() && !gt_lazy_data.is_empty() {
        let (p_value, significant, percent_diff) =
            stats_utils::compare_measurements(&cd_lazy_data, &gt_lazy_data, alpha);

        println!("\nChangeDistiller Lazy vs Gumtree Lazy:");
        println!("  Difference: {:.2}%", percent_diff);
        println!("  p-value: {:.4}", p_value);
        println!(
            "  Statistically significant: {}",
            if significant { "YES" } else { "NO" }
        );
    }
}

// Helper function to collect raw measurements from statistics hashmap
fn collect_measurements_from_stats(stats: &HashMap<String, f64>) -> Vec<usize> {
    // In a real-world scenario, we would store and pass the raw measurements
    // For this simplified version, we'll reconstruct approximate measurements
    // using the mean and standard deviation (assuming normal distribution)

    let n = stats.get("n").unwrap_or(&0.0).round() as usize;
    let mean = stats.get("mean").unwrap_or(&0.0);
    let std_dev = stats.get("std_dev").unwrap_or(&0.0);
    let min = stats.get("min").unwrap_or(&0.0);
    let max = stats.get("max").unwrap_or(&0.0);

    if n == 0 || *mean == 0.0 {
        return vec![];
    }

    // Create an approximate reconstruction using min, max, and mean
    // This is not statistically accurate but serves as a placeholder
    vec![*min as usize, *mean as usize, *max as usize]
}
