use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tabled::settings::{Reverse, Rotate};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Modify, Padding, Style, object::Columns},
};

fn main() {
    println!("Memory Benchmark using macOS time command");
    println!("=========================================\n");

    #[derive(Tabled)]
    struct BenchmarkResult {
        #[tabled(rename = "Algorithm")]
        algorithm: String,
        #[tabled(rename = "Avg Peak Memory (MB)")]
        peak_avg: String,
        #[tabled(rename = "Min Peak Memory (MB)")]
        peak_min: String,
        #[tabled(rename = "Max Peak Memory (MB)")]
        peak_max: String,
        #[tabled(rename = "Avg Resident Memory (MB)")]
        resident_avg: String,
        #[tabled(rename = "Min Resident Memory (MB)")]
        resident_min: String,
        #[tabled(rename = "Max Resident Memory (MB)")]
        resident_max: String,
        #[tabled(rename = "Avg Duration")]
        duration_avg: String,
        #[tabled(rename = "Min Duration")]
        duration_min: String,
        #[tabled(rename = "Max Duration")]
        duration_max: String,
    }

    // Define algorithms to test
    let algorithms = vec!["change_distiller", "change_distiller_lazy", "gumtree_lazy"];

    // Number of iterations for each algorithm
    let iterations = 2;

    println!("Running {} iterations for each algorithm\n", iterations);

    // Results table
    let mut results = Vec::new();

    // First build the binary
    Command::new("cargo")
        .args(["build", "--release", "--bin", "algorithm_runner"])
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .output()
        .expect("Failed to build binary");

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

            println!(
                "    Peak Memory: {:.2} MB",
                peak_memory as f64 / 1_048_576.0
            );
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
        let min_duration = durations
            .iter()
            .min()
            .cloned()
            .unwrap_or(Duration::from_secs(0));
        let max_duration = durations
            .iter()
            .max()
            .cloned()
            .unwrap_or(Duration::from_secs(0));

        // Store results
        results.push((
            algorithm,
            avg_peak,
            min_peak,
            max_peak,
            avg_resident,
            min_resident,
            max_resident,
            avg_duration,
            min_duration,
            max_duration,
        ));

        // Create an algorithm summary table with tabled
        let summary_data = vec![BenchmarkResult {
            algorithm: algorithm.to_string(),
            peak_avg: format!("{:.2}", avg_peak as f64 / 1_048_576.0),
            peak_min: format!("{:.2}", min_peak as f64 / 1_048_576.0),
            peak_max: format!("{:.2}", max_peak as f64 / 1_048_576.0),
            resident_avg: format!("{:.2}", avg_resident as f64 / 1_048_576.0),
            resident_min: format!("{:.2}", min_resident as f64 / 1_048_576.0),
            resident_max: format!("{:.2}", max_resident as f64 / 1_048_576.0),
            duration_avg: format!("{:.2?}", avg_duration),
            duration_min: format!("{:.2?}", min_duration),
            duration_max: format!("{:.2?}", max_duration),
        }];

        let mut summary_table = Table::new(summary_data);
        summary_table
            .with(Rotate::Left)
            .with(Style::rounded())
            .with(Padding::new(1, 1, 0, 0))
            .with(Modify::new(Columns::new(2..)).with(Alignment::right()));

        println!("\n{} Summary (over {} iterations):", algorithm, iterations);
        println!("{}", summary_table);
    }

    // Create table with tabled
    println!(
        "\n=== Memory Usage Comparison (macOS time command, {} iterations) ===",
        iterations
    );

    let mut table_data = Vec::new();

    for (
        name,
        avg_peak,
        min_peak,
        max_peak,
        avg_resident,
        min_resident,
        max_resident,
        avg_duration,
        min_duration,
        max_duration,
    ) in &results
    {
        table_data.push(BenchmarkResult {
            algorithm: name.to_string(),
            peak_avg: format!("{:.2}", *avg_peak as f64 / 1_048_576.0),
            peak_min: format!("{:.2}", *min_peak as f64 / 1_048_576.0),
            peak_max: format!("{:.2}", *max_peak as f64 / 1_048_576.0),
            resident_avg: format!("{:.2}", *avg_resident as f64 / 1_048_576.0),
            resident_min: format!("{:.2}", *min_resident as f64 / 1_048_576.0),
            resident_max: format!("{:.2}", *max_resident as f64 / 1_048_576.0),
            duration_avg: format!("{:.2?}", avg_duration),
            duration_min: format!("{:.2?}", min_duration),
            duration_max: format!("{:.2?}", max_duration),
        });
    }

    let mut table = Table::new(table_data);
    table
        .with(Rotate::Right)
        .with(Style::modern())
        .with(Reverse::columns(0))
        .with(Padding::new(1, 1, 0, 0))
        .with(Modify::new(Columns::new(2..)).with(Alignment::right()));

    println!("{}", table);
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
    let start = Instant::now();

    // Run the algorithm with macOS time command using the built binary
    let output = Command::new("/usr/bin/time")
        .args(["-l", "./target/release/algorithm_runner", algorithm])
        .stdout(Stdio::inherit()) // Show normal output
        .stderr(Stdio::piped()) // Capture stderr for time stats
        .output()
        .expect("Failed to execute time command");

    let duration = start.elapsed();

    // Parse memory usage from time output
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Extract peak memory (maximum resident set size)
    let resident_size = extract_memory_stat(&stderr, "maximum resident set size");

    // Extract resident memory
    let peak_memory_footprint = extract_memory_stat(&stderr, "peak memory footprint");

    (peak_memory_footprint, resident_size, duration)
}

fn extract_memory_stat(time_output: &str, stat_name: &str) -> u64 {
    for line in time_output.lines() {
        if line.contains(stat_name) {
            // Extract the number using regex
            let re = regex::Regex::new(r"(\d+)").unwrap();
            if let Some(captures) = re.captures(line) {
                if let Some(value_str) = captures.get(1) {
                    if let Ok(value) = value_str.as_str().parse::<u64>() {
                        return value;
                    }
                }
            }
        }
    }
    0 // Return 0 if stat not found
}
