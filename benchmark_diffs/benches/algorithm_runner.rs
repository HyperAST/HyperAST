use hyperast_benchmark_diffs::common::run_diff;
use std::env;
use std::fs;

fn main() {
    // Get algorithm name and file paths from command line args
    let args: Vec<String> = env::args().collect();
    if args.len() != 4 {
        eprintln!("Usage: algorithm_runner <algorithm_name> <buggy_file> <fixed_file>");
        std::process::exit(1);
    }

    let algorithm = &args[1];
    let buggy_file = &args[2];
    let fixed_file = &args[3];

    // Read file contents
    let buggy_content = fs::read_to_string(buggy_file)
        .unwrap_or_else(|_| {
            eprintln!("Failed to read buggy file: {}", buggy_file);
            std::process::exit(1);
        });
    let fixed_content = fs::read_to_string(fixed_file)
        .unwrap_or_else(|_| {
            eprintln!("Failed to read fixed file: {}", fixed_file);
            std::process::exit(1);
        });

    // Run the algorithm on the single file pair
    run_diff(&buggy_content, &fixed_content, algorithm);
}
