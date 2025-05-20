use common::{get_test_data_medium, get_test_data_mixed, get_test_data_small, run_diff};
use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use std::env;

mod common;

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
