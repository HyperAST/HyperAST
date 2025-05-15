use common::{get_test_data, run_diff};
use criterion::{Criterion, black_box, criterion_group, criterion_main};
use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;

mod common;

fn diff_benchmark(c: &mut Criterion) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();

    let test_inputs = get_test_data();

    let mut group = c.benchmark_group("change_distiller_comparison");
    group.sample_size(100);

    // group.bench_function("HyperDiff Lazy", |b| {
    //     b.iter(|| {
    //         for ( buggy, fixed) in &test_inputs {
    //             run_diff(buggy, fixed, "gumtree_lazy");
    //         }
    //     })
    // });

    group.bench_function("ChangeDistiller", |b| {
        b.iter(|| {
            for (buggy, fixed) in &test_inputs {
                run_diff(buggy, fixed, "change_distiller");
            }
        })
    });

    group.bench_function("ChangeDistiller Lazy", |b| {
        b.iter(|| {
            for (buggy, fixed) in &test_inputs {
                run_diff(buggy, fixed, "change_distiller_lazy");
            }
        })
    });

    group.finish();
}

criterion_group!(benches, diff_benchmark);
criterion_main!(benches);
