use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn main() {
    println!("Memory Benchmark using macOS time command");
    println!("=========================================\n");

    // Define algorithms to test
    let algorithms = vec!["change_distiller", "change_distiller_lazy", "gumtree_lazy"];
    
    // Number of iterations for each algorithm
    let iterations = 5;

    println!("Running {} iterations for each algorithm\n", iterations);

    // Results table
    let mut results = Vec::new();

    for algorithm in &algorithms {
        println!("\nRunning benchmark for {}...", algorithm);

        let mut peak_memories = Vec::with_capacity(iterations);
        let mut resident_memories = Vec::with_capacity(iterations);
        let mut durations = Vec::with_capacity(iterations);

        // Run the algorithm multiple times
        for i in 1..=iterations {
            println!("  Iteration {}/{}...", i, iterations);
            
            // Run the benchmark with time command
            let (peak_memory, resident_memory, duration) = run_benchmark_with_time(algorithm);
            
            peak_memories.push(peak_memory);
            resident_memories.push(resident_memory);
            durations.push(duration);
            
            println!("    Peak Memory: {:.2} MB", peak_memory as f64 / 1_048_576.0);
            println!(
                "    Resident Memory: {:.2} MB",
                resident_memory as f64 / 1_048_576.0
            );
            println!("    Duration: {:.2?}", duration);
        }

        // Calculate statistics for peak memory
        let avg_peak = calculate_average(&peak_memories);
        let min_peak = *peak_memories.iter().min().unwrap_or(&0);
        let max_peak = *peak_memories.iter().max().unwrap_or(&0);

        // Calculate statistics for resident memory
        let avg_resident = calculate_average(&resident_memories);
        let min_resident = *resident_memories.iter().min().unwrap_or(&0);
        let max_resident = *resident_memories.iter().max().unwrap_or(&0);

        // Calculate statistics for duration
        let avg_duration = calculate_average_duration(&durations);
        let min_duration = durations.iter().min().cloned().unwrap_or(Duration::from_secs(0));
        let max_duration = durations.iter().max().cloned().unwrap_or(Duration::from_secs(0));

        // Store results
        results.push((
            algorithm,
            avg_peak, min_peak, max_peak,
            avg_resident, min_resident, max_resident,
            avg_duration, min_duration, max_duration
        ));

        // Print summary statistics for this algorithm
        println!("\n{} Summary (over {} iterations):", algorithm, iterations);
        println!("  Peak Memory (MB): Avg: {:.2}, Min: {:.2}, Max: {:.2}", 
            avg_peak as f64 / 1_048_576.0, 
            min_peak as f64 / 1_048_576.0, 
            max_peak as f64 / 1_048_576.0);
        println!("  Resident Memory (MB): Avg: {:.2}, Min: {:.2}, Max: {:.2}", 
            avg_resident as f64 / 1_048_576.0, 
            min_resident as f64 / 1_048_576.0, 
            max_resident as f64 / 1_048_576.0);
        println!("  Duration: Avg: {:.2?}, Min: {:.2?}, Max: {:.2?}", 
            avg_duration, min_duration, max_duration);
    }

    // Print comparison table
    println!("\n=== Memory Usage Comparison (macOS time command, {} iterations) ===", iterations);
    println!("+------------------------+--------------------------------+---------------------------------------+-------------------------+", );
    println!(
        "| {:22} | {:30} | {:37} | {:23} |",
        "Algorithm", "Peak Memory (MB)", "Resident Memory (MB)", "Duration"
    );
    println!(
        "| {:22} | {:8} | {:8} | {:8} | {:11} | {:11} | {:11} | {:7} | {:7} | {:7} |",
        "", "Avg", "Min", "Max", "Avg", "Min", "Max", "Avg", "Min", "Max"
    );
    println!("+------------------------+--------------------------------+---------------------------------------+-------------------------+");

    for (name, avg_peak, min_peak, max_peak, avg_resident, min_resident, max_resident, avg_duration, min_duration, max_duration) in &results {
        println!(
            "| {:22} | {:8.2} | {:8.2} | {:8.2} | {:11.2} | {:11.2} | {:11.2} | {:7.2?} | {:7.2?} | {:7.2?} |",
            name,
            *avg_peak as f64 / 1_048_576.0,
            *min_peak as f64 / 1_048_576.0,
            *max_peak as f64 / 1_048_576.0,
            *avg_resident as f64 / 1_048_576.0,
            *min_resident as f64 / 1_048_576.0,
            *max_resident as f64 / 1_048_576.0,
            avg_duration,
            min_duration,
            max_duration
        );
    }
    println!("+------------------------+--------------------------------+---------------------------------------+-------------------------+");
}

fn calculate_average(values: &[u64]) -> u64 {
    if values.is_empty() {
        return 0;
    }
    values.iter().sum::<u64>() / values.len() as u64
}

fn calculate_average_duration(durations: &[Duration]) -> Duration {
    if durations.is_empty() {
        return Duration::from_secs(0);
    }
    
    let total_nanos: u128 = durations.iter().map(|d| d.as_nanos()).sum();
    Duration::from_nanos((total_nanos / durations.len() as u128) as u64)
}

fn run_benchmark_with_time(algorithm: &str) -> (u64, u64, Duration) {
    // Create a temporary binary that runs the algorithm
    let start = Instant::now();

    // Run the algorithm with macOS time command
    let output = Command::new("/usr/bin/time")
        .args([
            "-l",
            "cargo",
            "run",
            "--release",
            "--bin",
            "algorithm_runner",
            "--",
            algorithm,
        ])
        .stdout(Stdio::inherit()) // Show normal output
        .stderr(Stdio::piped()) // Capture stderr for time stats
        .output()
        .expect("Failed to execute time command");

    let duration = start.elapsed();

    // Parse memory usage from time output
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Extract peak memory (maximum resident set size)
    let peak_memory = extract_memory_stat(&stderr, "maximum resident set size");

    // Extract resident memory
    let resident_memory = extract_memory_stat(&stderr, "average resident set size");

    (peak_memory, resident_memory, duration)
}

fn extract_memory_stat(time_output: &str, stat_name: &str) -> u64 {
    for line in time_output.lines() {
        if line.contains(stat_name) {
            // Format is usually like "123456  maximum resident set size"
            if let Some(value_str) = line.split_whitespace().next() {
                if let Ok(value) = value_str.parse::<u64>() {
                    return value;
                }
            }
        }
    }
    0 // Return 0 if stat not found
}