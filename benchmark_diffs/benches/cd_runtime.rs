use common::run_diff;
use criterion::{Criterion, criterion_group, criterion_main};

mod common;

fn diff_benchmark(c: &mut Criterion) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();

    let test_inputs = common::get_test_data_small();

    println!(
        "Running benchmarks with {} test cases...",
        test_inputs.len()
    );
    println!(
        "Total lines of code in src: {}",
        test_inputs
            .iter()
            .map(|(buggy, fixed)| buggy.lines().count())
            .sum::<usize>()
    );

    let mut group = c.benchmark_group("change_distiller_comparison");
    group.sample_size(10);

    // group.bench_function("HyperDiff Lazy", |b| {
    //     b.iter(|| {
    //         for (buggy, fixed) in &test_inputs {
    //             run_diff(buggy, fixed, "gumtree_lazy");
    //         }
    //     })
    // });

    group.bench_function("ChangeDistiller Lazy 2", |b| {
        b.iter(|| {
            for (buggy, fixed) in &test_inputs {
                run_diff(buggy, fixed, "change_distiller_lazy_2");
            }
        })
    });

    // group.bench_function("ChangeDistiller Lazy", |b| {
    //     b.iter(|| {
    //         for (buggy, fixed) in &test_inputs {
    //             run_diff(buggy, fixed, "change_distiller_lazy");
    //         }
    //     })
    // });

    // group.bench_function("ChangeDistiller", |b| {
    //     b.iter(|| {
    //         for (buggy, fixed) in &test_inputs {
    //             run_diff(buggy, fixed, "change_distiller");
    //         }
    //     })
    // });

    group.finish();
}

criterion_group!(benches, diff_benchmark);
criterion_main!(benches);
