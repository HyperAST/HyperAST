use common::get_test_data;
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
    let test_inputs = get_test_data();

    // Run the algorithm once for each test case
    for (buggy, fixed) in &test_inputs {
        run_diff(buggy, fixed, algorithm);
    }
}

fn run_diff(src: &str, dst: &str, algorithm: &str) {
    // Initialize stores
    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();

    // Parse the two Java files
    let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, src, dst);

    // Perform the diff using specified algorithm
    let diff_result = match algorithm {
        "gumtree_lazy" => algorithms::gumtree_lazy::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        ),
        "change_distiller" => algorithms::change_distiller::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        ),
        "change_distiller_lazy" => algorithms::change_distiller_lazy::diff(
            &stores,
            &src_tr.local.compressed_node,
            &dst_tr.local.compressed_node,
        ),
        _ => panic!("Unknown diff algorithm: {}", algorithm),
    };

    // Force result to be used to prevent optimization
    if diff_result.actions.is_none() {
        println!("No changes found");
    }
}
