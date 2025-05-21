use hyperast_benchmark_diffs::common::{get_test_data_mixed, run_diff};
use std::env;

fn main() {
    // Get algorithm name from command line args
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: algorithm_runner <algorithm_name>");
        std::process::exit(1);
    }

    let algorithm = &args[1];

    // Run the algorithm
    run_algorithm(algorithm);
}

fn run_algorithm(algorithm: &str) {
    let test_inputs = get_test_data_mixed();

    // Run the algorithm once for each test case
    for (buggy, fixed) in &test_inputs {
        run_diff(buggy, fixed, algorithm);
    }
}
