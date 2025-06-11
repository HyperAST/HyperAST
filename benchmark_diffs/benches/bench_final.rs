pub(crate) mod bench_utils;

use criterion::{Criterion, criterion_group, criterion_main};
use criterion_perf_events::Perf;
use perfcnt::linux::HardwareEventType as Hardware;
use perfcnt::linux::PerfCounterBuilderLinux as Builder;

use crate::bench_utils::bench_utils_methods;
use crate::bench_utils::bench_utils_models::{DataSet, Heuristic};

fn all_heuristics_bugsinpy_httpie(c: &mut Criterion<Perf>) {
    let variants = [
        Heuristic::LazySimple,
        Heuristic::LazyGreedy,
        Heuristic::Simple,
        Heuristic::Greedy,
    ];
    let dataset = DataSet::BugsInPy(Some(String::from("httpie")));
    bench_utils_methods::run_all_heuristics_for_dataset(c, dataset, &variants);
}

fn run_all_heuristics_bugsinpie(c: &mut Criterion<Perf>) {
    let variants = [
        Heuristic::LazySimple,
        Heuristic::LazyGreedy,
        Heuristic::Simple,
        Heuristic::Greedy,
    ];

    let dataset_projects: Vec<DataSet> = DataSet::BugsInPy(None)
        .get_all_projects_of_dataset()
        .into_iter()
        .map(|name| DataSet::GhJava(Some(name)))
        .collect();

    for dataset in dataset_projects {
        bench_utils_methods::run_all_heuristics_for_dataset(c, dataset, &variants);
    }
}

fn run_all_heuristics_ghpython(c: &mut Criterion<Perf>) {
    let variants = [
        Heuristic::LazySimple,
        Heuristic::LazyGreedy,
        Heuristic::Simple,
        Heuristic::Greedy,
    ];

    let dataset_projects: Vec<DataSet> = DataSet::GhPython(None)
        .get_all_projects_of_dataset()
        .into_iter()
        .map(|name| DataSet::GhJava(Some(name)))
        .collect();

    for dataset in dataset_projects {
        bench_utils_methods::run_all_heuristics_for_dataset(c, dataset, &variants);
    }
}

fn run_all_heuristics_ghjava(c: &mut Criterion<Perf>) {
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
criterion_group!(
    name = bugsinpy_httpie;
    config = Criterion::default()
        .with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)))
        .sample_size(10)
        .configure_from_args();
    targets = all_heuristics_bugsinpy_httpie
);
criterion_group!(
    name = bugsinpy_all;
    config = Criterion::default()
        .with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)))
        .sample_size(15)
        .configure_from_args();
    targets = run_all_heuristics_bugsinpie
);
criterion_group!(
    name = ghpython_all;
    config = Criterion::default()
        .with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)))
        .sample_size(15)
        .configure_from_args();
    targets = run_all_heuristics_ghpython
);
criterion_group!(
    name = defects4j_all;
    config = Criterion::default()
        .with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)))
        .sample_size(15)
        .configure_from_args();
    targets = run_all_heuristics_defects4j
);
criterion_group!(
    name = ghjava_all;
    config = Criterion::default()
        .with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)))
        .sample_size(15)
        .configure_from_args();
    targets = run_all_heuristics_ghjava
);
// criterion_main!(bugsinpy_all, ghpython_all, defects4j_all, ghjava_all);
criterion_main!(bugsinpy_httpie);
