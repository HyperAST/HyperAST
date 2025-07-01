use criterion::{Criterion, criterion_group, criterion_main};
use hyperast_benchmark_diffs::preprocess_repo::parse_repo;
use hyperast_benchmark_diffs::run_diff::run_diff_trees;
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;
use std::time::Duration;

const DEFAULT_SIZE_THRESHOLD: usize = 1000;

fn diff_benchmark(c: &mut Criterion) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    let mut group = c.benchmark_group("gumtree_repo_comparison");

    group.sample_size(10);

    struct BenchmarkItem {
        repositories: PreProcessedRepositories, // todo: ugly workaround to avoid issues with borrowing
        repo_user: &'static str,
        repo_name: &'static str,
        config: hyperast_vcs_git::processing::RepoConfig,
        before: &'static str,
        after: &'static str,
    }

    let mut dataset = vec![
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "apache",
            repo_name: "maven",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "a02834611bad3442ad073b10f1dee2322916f1f3",
            after: "c3cf29438e3d65d6ee5c5726f8611af99d9a649a",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "apache",
            repo_name: "maven",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "14449e426aee2763d6435b63ef632b7c0b9ed767",
            after: "6fba7aa3c4d31d088df3ef682f7307b7c9a2f17c",
        },
    ];

    let dataset_trees: Vec<_> = dataset
        .iter_mut()
        .map(|commit| {
            parse_repo(
                &mut commit.repositories,
                commit.repo_user,
                commit.repo_name,
                commit.config,
                commit.before,
                commit.after,
            )
        })
        .collect();

    log::warn!("Finished loading hyperasts for dataset, running benchmarks");

    use hyperast_benchmark_diffs::run_diff::Algorithm::*;
    let tested_fcts: Vec<_> = vec![
        (Hybrid, 50),
        (Hybrid, 100),
        (Hybrid, 500),
        (Hybrid, 1000),
        (Simple, DEFAULT_SIZE_THRESHOLD),
        (Greedy, DEFAULT_SIZE_THRESHOLD),
        (LazyGreedy, DEFAULT_SIZE_THRESHOLD),
        (LazyHybrid, 50),
        (LazyHybrid, 100),
        (LazyHybrid, 500),
        (LazyHybrid, 1000),
    ];

    let select_metric = |summary: hyper_diff::algorithms::ResultsSummary<
        hyper_diff::algorithms::PreparedMappingDurations<2, Duration>,
        Duration,
    >| { *summary.mapping_durations.mappings.0.get(1).unwrap() };

    for (algo, max_size) in &tested_fcts {
        group.bench_function(format!("{algo}_{max_size}"), |b| {
            b.iter_custom(|iters| {
                let mut time = Duration::ZERO;
                for _i in 0..iters {
                    for (stores, src_tr, dst_tr) in &dataset_trees {
                        let summary = run_diff_trees(stores, &src_tr, &dst_tr, algo, *max_size);
                        dbg!(&summary);
                        time += select_metric(summary);
                    }
                }
                time
            })
        });
    }
    group.finish();
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10).configure_from_args();
    targets = diff_benchmark
);
criterion_main!(benches);
