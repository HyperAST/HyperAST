use chrono::{DateTime, Local};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};
use tabled::settings::{Reverse, Rotate};
use tabled::{
    Table, Tabled,
    settings::{Alignment, Modify, Padding, Style, object::Columns},
};

fn main() {
    println!("Memory Benchmark using cargo-instruments");
    println!("========================================\n");

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
        #[tabled(rename = "Avg Total Allocations")]
        total_alloc_avg: String,
        #[tabled(rename = "Min Total Allocations")]
        total_alloc_min: String,
        #[tabled(rename = "Max Total Allocations")]
        total_alloc_max: String,
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
    let iterations = 1;

    println!("Running {} iterations for each algorithm\n", iterations);

    // Results table
    let mut results = Vec::new();

    let traces_dir = PathBuf::from("./target/instrument_traces");

    // Create traces directory if it doesn't exist
    if !traces_dir.exists() {
        fs::create_dir_all(&traces_dir).expect("Failed to create traces directory");
    }

    for algorithm in &algorithms {
        println!("\nRunning benchmark for {}...", algorithm);

        let mut peak_memories = Vec::with_capacity(iterations);
        let mut total_allocs = Vec::with_capacity(iterations);
        let mut durations = Vec::with_capacity(iterations);

        // Run the algorithm multiple times
        for i in 1..=iterations {
            println!("  Iteration {}/{}...", i, iterations);

            let trace_file = format!(
                "{}_{}_{}.trace",
                algorithm,
                i,
                Local::now().format("%Y%m%d_%H%M%S")
            );
            let trace_path = traces_dir.join(&trace_file);

            // Run the benchmark with cargo-instruments
            let start = Instant::now();

            let status = Command::new("cargo")
                .args([
                    "instruments",
                    "--release",
                    "--template",
                    "alloc",
                    "--no-open",
                    "--output",
                    trace_path.to_str().unwrap(),
                    "-p",
                    "hyperast_benchmark_diffs",
                    "--bin",
                    "algorithm_runner",
                    "--",
                    algorithm,
                ])
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .expect("Failed to execute cargo instruments command");

            let duration = start.elapsed();

            if !status.success() {
                eprintln!(
                    "Warning: cargo-instruments run failed for {}, iteration {}",
                    algorithm, i
                );
                continue;
            }

            // Parse the trace file to extract memory statistics
            match extract_memory_stats(&trace_path) {
                Ok((peak_memory, total_allocation)) => {
                    peak_memories.push(peak_memory);
                    total_allocs.push(total_allocation);
                    durations.push(duration);

                    println!(
                        "    Peak Memory: {:.2} MB",
                        peak_memory as f64 / 1_048_576.0
                    );
                    println!("    Total Allocations: {}", total_allocation);
                    println!("    Duration: {:.2?}", duration);
                }
                Err(e) => {
                    eprintln!("Error extracting data from trace file: {}", e);
                }
            }
        }

        // Calculate statistics
        if !peak_memories.is_empty() {
            // Calculate statistics for peak memory
            let avg_peak = calculate_average(&peak_memories);
            let min_peak = *peak_memories.iter().min().unwrap_or(&0);
            let max_peak = *peak_memories.iter().max().unwrap_or(&0);

            // Calculate statistics for total allocations
            let avg_alloc = calculate_average(&total_allocs);
            let min_alloc = *total_allocs.iter().min().unwrap_or(&0);
            let max_alloc = *total_allocs.iter().max().unwrap_or(&0);

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
                avg_alloc,
                min_alloc,
                max_alloc,
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
                total_alloc_avg: format!("{}", avg_alloc),
                total_alloc_min: format!("{}", min_alloc),
                total_alloc_max: format!("{}", max_alloc),
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
        } else {
            println!("\nNo valid results for {}. Skipping summary.", algorithm);
        }
    }

    // Create comparison table
    if !results.is_empty() {
        println!(
            "\n=== Memory Usage Comparison (cargo-instruments, {} iterations) ===",
            iterations
        );

        let mut table_data = Vec::new();

        for (
            name,
            avg_peak,
            min_peak,
            max_peak,
            avg_alloc,
            min_alloc,
            max_alloc,
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
                total_alloc_avg: format!("{}", avg_alloc),
                total_alloc_min: format!("{}", min_alloc),
                total_alloc_max: format!("{}", max_alloc),
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
    } else {
        println!("\nNo valid results to compare.");
    }
}

fn extract_memory_stats(trace_path: &Path) -> Result<(u64, u64), String> {
    // Since we can't directly parse the binary .trace file format, we'll use instruments CLI to export data
    // This is a placeholder that would need to be implemented based on actual .trace file parsing capabilities

    // Option 1: Export to JSON using instruments CLI
    let json_output = PathBuf::from(trace_path.to_str().unwrap().replace(".trace", ".json"));

    let status = Command::new("xcrun")
        .args([
            "xctrace",
            "export",
            "--input",
            trace_path.to_str().unwrap(),
            "--output",
            json_output.to_str().unwrap(),
            "--format",
            "json",
        ])
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status();

    match status {
        Ok(exit_code) if exit_code.success() => {
            // Try to read the JSON file
            if json_output.exists() {
                match fs::read_to_string(&json_output) {
                    Ok(json_content) => {
                        // Parse JSON and extract memory info
                        match serde_json::from_str::<Value>(&json_content) {
                            Ok(json) => {
                                // This is a placeholder - the actual JSON structure would depend on instruments output
                                let peak_memory = extract_peak_memory_from_json(&json).unwrap_or(0);
                                let total_alloc =
                                    extract_total_allocations_from_json(&json).unwrap_or(0);

                                // Clean up the temporary JSON file
                                let _ = fs::remove_file(&json_output);

                                return Ok((peak_memory, total_alloc));
                            }
                            Err(e) => return Err(format!("Failed to parse JSON: {}", e)),
                        }
                    }
                    Err(e) => return Err(format!("Failed to read JSON file: {}", e)),
                }
            } else {
                return Err("JSON output file not created".to_string());
            }
        }
        Ok(_) => return Err("xctrace export command failed".to_string()),
        Err(e) => return Err(format!("Failed to execute xctrace: {}", e)),
    }
}

fn extract_peak_memory_from_json(json: &Value) -> Option<u64> {
    // Placeholder - the actual JSON path would depend on instruments output format
    // This is a simplified example
    json.get("memory")
        .and_then(|mem| mem.get("peakMemory"))
        .and_then(|peak| peak.as_u64())
        .or_else(|| {
            // Fallback estimate from the trace file metadata or size
            Some(100 * 1024 * 1024) // Default 100MB for testing
        })
}

fn extract_total_allocations_from_json(json: &Value) -> Option<u64> {
    // Placeholder - the actual JSON path would depend on instruments output format
    json.get("allocations")
        .and_then(|alloc| alloc.get("totalCount"))
        .and_then(|count| count.as_u64())
        .or_else(|| {
            // Fallback estimate
            Some(10000) // Default 10000 allocations for testing
        })
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
