use criterion::{BenchmarkId, Throughput};
use criterion::{Criterion, criterion_group, criterion_main};
use hyper_diff::OptimizedDiffConfig;
use hyperast_benchmark_diffs::common;

/// Create various optimization configurations for comprehensive benchmarking
fn create_optimization_configs() -> Vec<(&'static str, OptimizedDiffConfig)> {
    vec![
        ("Baseline Deep Label", OptimizedDiffConfig::baseline()),
        (
            "Baseline Statement",
            OptimizedDiffConfig::baseline().with_statement_level_iteration(),
        ),
        (
            "Baseline Deep Statement",
            OptimizedDiffConfig::baseline()
                .with_statement_level_iteration()
                .with_deep_leaves(),
        ),
        // Optimized Label
        ("Optimized Deep Label", OptimizedDiffConfig::optimized()),
        (
            "Optimized Deep Label Cache",
            OptimizedDiffConfig::optimized().with_label_caching(),
        ),
        // Optimized shallow statements
        (
            "Optimized with Statement",
            OptimizedDiffConfig::optimized().with_statement_level_iteration(),
        ),
        (
            "Optimized with Statement and Ngram Caching",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_ngram_caching(),
        ),
        (
            "Optimized with Statement and Label Cache",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_label_caching(),
        ),
        // Optimized deep statements
        (
            "Optimized with Deep Statement",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_deep_leaves(),
        ),
        (
            "Optimized with Deep Statement and Ngram Caching",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_deep_leaves()
                .with_ngram_caching(),
        ),
        (
            "Optimized with Deep Statement and Label Cache",
            OptimizedDiffConfig::optimized()
                .with_statement_level_iteration()
                .with_label_caching()
                .with_deep_leaves(),
        ),
    ]
}

fn benchmark_optimized_change_distiller(c: &mut Criterion) {
    let inputs = common::test_data_small();
    let mut group = c.benchmark_group("cd_runtime");

    let configs = create_optimization_configs();

    for input in inputs {
        let files = common::read_test_data(input);
        let prepro = common::preprocess_file_pair([files.0.as_str(), files.1.as_str()]);
        group.throughput(Throughput::Elements(prepro.node_count as u64));
        for (config_name, config) in configs.iter().copied() {
            group.bench_with_input(BenchmarkId::new(config_name, &input), &prepro, |b, p| {
                b.iter(|| {
                    use change_distiller_optimized::diff_with_complete_decompression;
                    use change_distiller_optimized::diff_with_lazy_decompression;
                    use hyper_diff::algorithms::change_distiller_optimized;
                    if config.use_lazy_decompression {
                        diff_with_lazy_decompression(&p.stores, &p.src, &p.dst, config)
                    } else {
                        diff_with_complete_decompression(&p.stores, &p.src, &p.dst, config)
                    }
                    .into_diff_result()
                });
            });
        }
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10).configure_from_args();
    targets = benchmark_optimized_change_distiller,
}
criterion_main!(benches);
