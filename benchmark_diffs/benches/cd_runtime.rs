use criterion::{Criterion, SamplingMode, black_box, criterion_group, criterion_main};
use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast_benchmark_diffs::{common::run_diff, preprocess::parse_string_pair};

fn diff_benchmark(c: &mut Criterion) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();

    let test_inputs = hyperast_benchmark_diffs::common::get_test_data_small();

    println!(
        "Running benchmarks with {} test cases...",
        test_inputs.len()
    );
    println!(
        "Total lines of code in src: {}",
        test_inputs
            .iter()
            .map(|(buggy, _)| buggy.lines().count())
            .sum::<usize>()
    );

    let mut group = c.benchmark_group("change_distiller_comparison");
    group.sample_size(10);
    group.sampling_mode(SamplingMode::Flat);

    // group.bench_function("HyperDiff Lazy", |b| {
    //     b.iter(|| {
    //         for (buggy, fixed) in &test_inputs {
    //             run_diff(buggy, fixed, "gumtree_lazy");
    //         }
    //     })
    // });

    group.bench_function("ChangeDistiller Lazy 2 Precomp", |b| {
        let inputs = test_inputs
            .iter()
            .map(|(src, dst)| {
                let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
                let mut md_cache = Default::default();
                let (src_tr, dst_tr) = parse_string_pair(&mut stores, &mut md_cache, src, dst);
                (stores, src_tr, dst_tr)
            })
            .collect::<Vec<_>>();
        b.iter(|| {
            for (stores, src_tr, dst_tr) in &inputs {
                let res = algorithms::change_distiller_lazy_2::diff(
                    stores,
                    &src_tr.local.compressed_node,
                    &dst_tr.local.compressed_node,
                );
                black_box(res);
            }
        })
    });

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

criterion_group! {
    name = benches;
    config = Criterion::default().configure_from_args();
    targets = diff_benchmark
}
// criterion_group!(benches, diff_benchmark);
criterion_main!(benches);
