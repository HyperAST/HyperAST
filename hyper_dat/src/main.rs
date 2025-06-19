mod run_diff;
mod gridsearch;
mod tpe;

use std::hint::black_box;
use num_traits::ToPrimitive;
use hyper_diff::actions::script_generator2::SimpleAction;
use hyper_diff::algorithms;
use hyper_diff::algorithms::DiffResult;
use hyperast::store::SimpleStores;
use hyperast::utils::memusage_linux;
use hyperast_vcs_git::preprocessed::PreProcessedRepository;
use hyperast_benchmark_diffs::{other_tools, postprocess::CompressedBfPostProcess};
use hyperast_benchmark_diffs::preprocess::parse_string_pair;



#[test]
fn test_run_diff_commit() {
    // run_diff::run_diff_commit("apache/maven",
    //                 "a02834611bad3442ad073b10f1dee2322916f1f3",
    //                 "c3cf29438e3d65d6ee5c5726f8611af99d9a649a",
    //                           100,
    // );
}

#[test]
fn run_grid_search() {

}

fn main() {
    
}