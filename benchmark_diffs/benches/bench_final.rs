pub(crate) mod bench_utils;

use criterion::measurement::Measurement;
use criterion::{Criterion, criterion_group, criterion_main};

use crate::bench_utils::bench_utils_methods;
use crate::bench_utils::bench_utils_models::{DataSet, Heuristic};

fn run_all_heuristics_ghjava<M: Measurement>(c: &mut Criterion<M>) {
    let variants = [
        Heuristic::LazySimple,
        Heuristic::Simple,
        // Heuristic::LazyGreedy,
        // Heuristic::Greedy,
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

fn run_all_heuristics_defects4j<M: Measurement>(c: &mut Criterion<M>) {
    let variants = [
        Heuristic::LazySimple,
        Heuristic::Simple,
        // Heuristic::LazyGreedy,
        // Heuristic::Greedy,
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
#[cfg(target_os = "linux")]
criterion_group!(
    name = defects4j_all;
    config = Criterion::default()
        .with_measurement(criterion_perf_events::Perf::new(perfcnt::linux::PerfCounterBuilderLinux::from_hardware_event(perfcnt::linux::HardwareEventType::Instructions)))
        .sample_size(15)
        .configure_from_args();
    targets = run_all_heuristics_defects4j
);
#[cfg(not(target_os = "linux"))]
criterion_group!(
    name = defects4j_all;
    config = Criterion::default()
        .sample_size(15)
        .configure_from_args();
    targets = run_all_heuristics_defects4j
);
#[cfg(target_os = "linux")]
criterion_group!(
    name = ghjava_all;
    config = Criterion::default()
        .with_measurement(criterion_perf_events::Perf::new(perfcnt::linux::PerfCounterBuilderLinux::from_hardware_event(perfcnt::linux::HardwareEventType::Instructions)))
        .sample_size(15)
        .configure_from_args();
    targets = run_all_heuristics_ghjava
);
#[cfg(not(target_os = "linux"))]
criterion_group!(
    name = ghjava_all;
    config = Criterion::default()
        .sample_size(15)
        .configure_from_args();
    targets = run_all_heuristics_ghjava
);
#[cfg(target_os = "linux")]
criterion_main!(defects4j_all, ghjava_all);
#[cfg(not(target_os = "linux"))]
criterion_main!(defects4j_all, ghjava_all);
