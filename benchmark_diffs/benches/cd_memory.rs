use common::get_test_data;
use memory_stats::memory_stats;
use std::time::Instant;

mod common;

fn measure_memory_usage(
    algorithm: &str,
    iterations: usize,
    warmup_iterations: usize,
) -> (usize, usize) {
    // (peak, average)
    let test_inputs = get_test_data();

    // Perform warmup iterations to stabilize JIT and memory usage
    println!("Warming up for {} iterations...", warmup_iterations);
    for _ in 0..warmup_iterations {
        for (buggy, fixed) in &test_inputs {
            common::run_diff(buggy, fixed, algorithm);
        }
    }

    let mut peak_memory = 0;
    let mut total_memory = 0;
    let mut valid_iterations = 0;

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
            // Update peak and total
            peak_memory = peak_memory.max(memory_delta);
            total_memory += memory_delta;
            valid_iterations += 1;

            println!(
                "Iteration {:2}: Memory used: {:10} bytes ({:6.2} MB)",
                i + 1,
                memory_delta,
                memory_delta as f64 / 1_048_576.0
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

    let average_memory = if valid_iterations > 0 {
        total_memory / valid_iterations
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
        "Peak Memory Usage: {:10} bytes ({:6.2} MB)",
        peak_memory,
        peak_memory as f64 / 1_048_576.0
    );
    println!(
        "Average Memory Usage: {:10} bytes ({:6.2} MB)",
        average_memory,
        average_memory as f64 / 1_048_576.0
    );
    (title, peak_memory, average_memory)
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
    println!("+-----------------------+-----------------+-----------------+");
    println!(
        "| {:21} | {:14} | {:14} |",
        "Algorithm", "Peak Memory", "Avg Memory"
    );
    println!("+-----------------------+-----------------+-----------------+");

    for (name, peak, avg) in results {
        println!(
            "| {:21} | {:12.2} MB | {:12.2} MB |",
            name,
            peak as f64 / 1_048_576.0,
            avg as f64 / 1_048_576.0
        );
    }
    println!("+-----------------------+-----------------+-----------------+");
}
