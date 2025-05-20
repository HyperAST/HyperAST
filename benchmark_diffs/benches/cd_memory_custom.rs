use common::get_test_data_small;
use std::time::Instant;

// Import the memory tracker
use hyperast_benchmark_diffs::memory_tracker::{
    self, get_allocated, get_allocation_count, get_allocations_since_mark, get_peak_allocated,
    get_peak_net_allocated, mark, reset_all, reset_peak,
};

// Use our custom memory tracker as the global allocator
#[global_allocator]
static GLOBAL: memory_tracker::MemoryTracker = memory_tracker::MemoryTracker;

mod common;

fn measure_memory_usage(
    algorithm: &str,
    iterations: usize,
    warmup_iterations: usize,
) -> (usize, usize, usize, usize) {
    // (peak_net, peak_absolute, average_net, allocation_count)
    let test_inputs = get_test_data_small();

    // Perform warmup iterations to stabilize JIT and memory usage
    println!("Warming up for {} iterations...", warmup_iterations);
    for _ in 0..warmup_iterations {
        for (buggy, fixed) in &test_inputs {
            common::run_diff(buggy, fixed, algorithm);
        }
    }

    let mut peak_net_memory = 0;
    let mut peak_absolute_memory = 0;
    let mut total_net_memory = 0;
    let mut total_allocation_count = 0;

    println!("Starting measurement...");
    for i in 0..iterations {
        // Reset memory tracking before each iteration
        reset_all();
        reset_peak();

        // Mark this point to measure changes from here
        mark();

        // Run the algorithm
        for (buggy, fixed) in &test_inputs {
            common::run_diff(buggy, fixed, algorithm);
        }

        // Get memory measurements
        let peak_memory = get_peak_allocated();
        let peak_net = get_peak_net_allocated();
        let allocation_count = get_allocations_since_mark();

        // Update stats
        peak_net_memory = peak_net_memory.max(peak_net);
        peak_absolute_memory = peak_absolute_memory.max(peak_memory);
        total_net_memory += peak_net; // Use peak_net instead of final diff
        total_allocation_count += allocation_count;

        println!(
            "Iteration {:2}: Peak Net: {:10} bytes ({:6.2} KB), Peak Absolute: {:10} bytes ({:6.2} KB), Allocations: {}",
            i + 1,
            peak_net,
            peak_net as f64 / 1_024.0,
            peak_memory,
            peak_memory as f64 / 1_024.0,
            allocation_count
        );
    }

    let average_net_memory = if iterations > 0 {
        total_net_memory / iterations
    } else {
        0
    };

    let average_allocation_count = if iterations > 0 {
        total_allocation_count / iterations
    } else {
        0
    };

    (
        peak_net_memory,
        peak_absolute_memory,
        average_net_memory,
        average_allocation_count,
    )
}

/// This function runs the benchmark and prints its stats and returns detailed memory usage metrics
fn run_benchmark<'a>(
    title: &'a str,
    name: &str,
    iterations: usize,
    warmup_iterations: usize,
) -> (&'a str, usize, usize, usize, usize) {
    println!("\n\n===Running benchmark for {}...===", title);
    let start = Instant::now();
    let (peak_net, peak_absolute, average_net, allocation_count) =
        measure_memory_usage(name, iterations, warmup_iterations);
    let duration = start.elapsed();
    println!("\n{} Memory Benchmark Results", title);
    println!("Duration: {:?}", duration);
    println!(
        "Peak Net Memory: {:10} bytes ({:6.2} KB)",
        peak_net,
        peak_net as f64 / 1_024.0
    );
    println!(
        "Peak Memory Absolute: {:10} bytes ({:6.2} KB)",
        peak_absolute,
        peak_absolute as f64 / 1_024.0
    );
    println!(
        "Average Net Memory: {:10} bytes ({:6.2} KB)",
        average_net,
        average_net as f64 / 1_024.0
    );
    println!("Average Allocation Count: {} operations", allocation_count);
    (
        title,
        peak_net,
        peak_absolute,
        average_net,
        allocation_count,
    )
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();

    let iterations = 10;
    let warmup_iterations = 3;

    println!(
        "\n=== Memory Allocation Benchmark (Custom Tracker): Running {} iterations per algorithm (with {} warmup iterations) ===\n",
        iterations, warmup_iterations
    );

    // Reset memory tracking at the start
    reset_all();

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
    println!("\n=== Memory Allocation Comparison ===");
    println!(
        "+-----------------------+-----------------+-----------------+-----------------+-------------------+"
    );
    println!(
        "| {:21} | {:15} | {:15} | {:15} | {:17} |",
        "Algorithm", "Peak Net", "Peak Absolute", "Avg Net", "Avg Allocations"
    );
    println!(
        "+-----------------------+-----------------+-----------------+-----------------+-------------------+"
    );

    for (name, peak_net, peak_abs, avg_net, alloc_count) in results {
        println!(
            "| {:21} | {:12.2} KB | {:12.2} KB | {:12.2} KB | {:17} |",
            name,
            peak_net as f64 / 1_024.0,
            peak_abs as f64 / 1_024.0,
            avg_net as f64 / 1_024.0,
            alloc_count
        );
    }
    println!(
        "+-----------------------+-----------------+-----------------+-----------------+-------------------+"
    );
}
