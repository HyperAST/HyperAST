pub(crate) mod bench_utils;

use criterion::{Criterion, criterion_group, criterion_main};
use criterion_perf_events::Perf;
use perfcnt::linux::HardwareEventType as Hardware;
use perfcnt::linux::PerfCounterBuilderLinux as Builder;

use crate::bench_utils::bench_utils_methods;
use crate::bench_utils::bench_utils_models::{DataSet, Heuristic};

fn run_all_heuristics_gh_java_drool(c: &mut Criterion<Perf>) {
    let variants = [
        Heuristic::LazySimple,
        Heuristic::LazyGreedy,
        Heuristic::Simple,
        Heuristic::Greedy,
    ];
    let dataset = DataSet::GhJava(Some(String::from("drool")));
    bench_utils_methods::run_all_heuristics_for_dataset(c, dataset, &variants);
}

fn run_all_heuristics_gh_java(c: &mut Criterion<Perf>) {
    let variants = [
        Heuristic::LazySimple,
        Heuristic::LazyGreedy,
        Heuristic::Simple,
        Heuristic::Greedy,
    ];

    let dataset_projects: Vec<DataSet> = DataSet::GhJava(None)
        .get_all_projects_of_dataset()
        .into_iter()
        .map(|name| DataSet::GhJava(Some(name)))
        .collect();

    for dataset in dataset_projects {
        bench_utils_methods::run_all_heuristics_for_dataset(c, dataset, &variants);
    }
}

fn run_all_heuristics_defects4j(c: &mut Criterion<Perf>) {
    let variants = [
        Heuristic::LazySimple,
        Heuristic::LazyGreedy,
        Heuristic::Simple,
        Heuristic::Greedy,
    ];

    let dataset_projects: Vec<DataSet> = DataSet::Defects4J(None)
        .get_all_projects_of_dataset()
        .into_iter()
        .map(|name| DataSet::Defects4J(Some(name)))
        .collect();

    for dataset in dataset_projects {
        bench_utils_methods::run_all_heuristics_for_dataset(c, dataset, &variants);
    }
}

// Make sure the event_paranoid is set for this session, 0 or 1 should suffice.
// sudo sysctl -w kernel.perf_event_paranoid=0
// criterion_group!(
//     name = gh_java_all_heuristic_drool;
//     config = Criterion::default()
//         .with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)))
//         .sample_size(10)
//         .configure_from_args();
//     targets = run_all_heuristics_gh_java_drool
// );
criterion_group!(
    name = defects4j_all_heuristic;
    config = Criterion::default()
        .with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)))
        .sample_size(15)
        .configure_from_args();
    targets = run_all_heuristics_defects4j
);
criterion_group!(
    name = gh_java_all_heuristic;
    config = Criterion::default()
        .with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)))
        .sample_size(15)
        .configure_from_args();
    targets = run_all_heuristics_gh_java
);
criterion_main!(gh_java_all_heuristic, defects4j_all_heuristic);
