use hyperast_benchmark_diffs::common::get_test_data_small;
use std::time::Instant;

#[cfg(not(target_env = "msvc"))]
use jemalloc_ctl::{epoch, stats};
#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn measure_memory_usage(
    algorithm: &str,
    iterations: usize,
    warmup_iterations: usize,
) -> (usize, usize) {
    // (peak, average)
    let test_inputs = get_test_data_small();

    // Perform warmup iterations to stabilize JIT and memory usage
    println!("Warming up for {} iterations...", warmup_iterations);
    for _ in 0..warmup_iterations {
        for (buggy, fixed) in &test_inputs {
            hyperast_benchmark_diffs::common::run_diff(buggy, fixed, algorithm);
        }
    }

    let mut peak_memory = 0;
    let mut total_memory = 0;

    // Helper functions for jemalloc memory measurement
    let update_jemalloc_stats = || {
        epoch::advance().unwrap();
    };

    let get_allocated_memory = || stats::allocated::read().unwrap();

    println!("Starting measurement...");
    for i in 0..iterations {
        // Reset memory stats
        update_jemalloc_stats();

        // Get memory before algorithm
        let before_allocated = get_allocated_memory();

        // Run the algorithm
        for (buggy, fixed) in &test_inputs {
            hyperast_benchmark_diffs::common::run_diff(buggy, fixed, algorithm);
        }

        // Get memory after algorithm
        update_jemalloc_stats();
        let after_allocated = get_allocated_memory();

        // Calculate memory used by the algorithm
        let memory_used = after_allocated.saturating_sub(before_allocated);

        // Update peak and total
        peak_memory = peak_memory.max(memory_used);
        total_memory += memory_used;

        println!(
            "Iteration {:2}: Memory allocated: {:10} bytes ({:6.2} KB)",
            i + 1,
            memory_used,
            memory_used as f64 / 1_024.0
        );
    }

    let average_memory = if iterations > 0 {
        total_memory / iterations
    } else {
        0
    };

    (peak_memory, average_memory)
}

/// This function runs the benchmark and prints its stats and returns the peak and average memory usage
fn run_benchmark<'a>(
    title: &'a str,
    name: &str,
    iterations: usize,
    warmup_iterations: usize,
) -> (&'a str, usize, usize) {
    println!("\n\n===Running benchmark for {}...===", title);
    let start = Instant::now();
    let (peak_memory, average_memory) = measure_memory_usage(name, iterations, warmup_iterations);
    let duration = start.elapsed();
    println!("\n{} Memory Benchmark Results", title);
    println!("Duration: {:?}", duration);
    println!(
        "Peak Memory Allocated: {:10} bytes ({:6.2} KB)",
        peak_memory,
        peak_memory as f64 / 1_024.0
    );
    println!(
        "Average Memory Allocated: {:10} bytes ({:6.2} KB)",
        average_memory,
        average_memory as f64 / 1_024.0
    );
    (title, peak_memory, average_memory)
}

fn main() {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();

    let iterations = 10;
    let warmup_iterations = 3;

    println!(
        "\n=== Memory Allocation Benchmark: Running {} iterations per algorithm (with {} warmup iterations) ===\n",
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
    println!("\n=== Memory Allocation Comparison ===");
    println!("+-----------------------+-----------------+-----------------+");
    println!(
        "| {:21} | {:14} | {:14} |",
        "Algorithm", "Peak Memory", "Avg Memory"
    );
    println!("+-----------------------+-----------------+-----------------+");

    for (name, peak, avg) in results {
        println!(
            "| {:21} | {:12.2} KB | {:12.2} KB |",
            name,
            peak as f64 / 1_024.0,
            avg as f64 / 1_024.0
        );
    }
    println!("+-----------------------+-----------------+-----------------+");
}
