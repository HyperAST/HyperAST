use std::path::Path;
use std::time::Duration;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use hyperast::utils::memusage_linux;
use hyperast_benchmark_diffs::preprocess_repo::parse_repo;
use hyperast_benchmark_diffs::run_diff::{run_diff, run_diff_trees};
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;
use std::sync::Arc;



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
            after: "c3cf29438e3d65d6ee5c5726f8611af99d9a649a"
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "apache",
            repo_name: "maven",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "14449e426aee2763d6435b63ef632b7c0b9ed767",
            after: "6fba7aa3c4d31d088df3ef682f7307b7c9a2f17c"
        }
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
    
    group.bench_function("hybrid_50", |b| {
        b.iter_custom(|iters| {
            let mut time = Duration::new(0, 0);
            for _i in 0..iters {
                for (stores, src_tr, dst_tr) in &dataset_trees {
                    let summary = run_diff_trees(stores, src_tr, dst_tr, "hybrid", 50);
                    dbg!(&summary);
                    time += Duration::from_secs_f64(summary
                        .mapping_durations.mappings.0.get(1).unwrap().clone());
                }
            }
            time
        })
    });
    group.bench_function("hybrid_100", |b| {
        b.iter_custom(|iters| {
            let mut time = Duration::new(0, 0);
            for _i in 0..iters {
                for (stores, src_tr, dst_tr) in &dataset_trees {
                    let summary = run_diff_trees(stores, src_tr, dst_tr, "hybrid", 100);
                    dbg!(&summary);
                    time += Duration::from_secs_f64(summary
                        .mapping_durations.mappings.0.get(1).unwrap().clone());
                }
            }
            time
        })
    });
    group.bench_function("hybrid_500", |b| {
        b.iter_custom(|iters| {
            let mut time = Duration::new(0, 0);
            for _i in 0..iters {
                for (stores, src_tr, dst_tr) in &dataset_trees {
                    let summary = run_diff_trees(stores, src_tr, dst_tr, "hybrid", 500);
                    dbg!(&summary);
                    time += Duration::from_secs_f64(summary
                        .mapping_durations.mappings.0.get(1).unwrap().clone());
                }
            }
            time
        })
    });
    group.bench_function("hybrid_1000", |b| {
        b.iter_custom(|iters| {
            let mut time = Duration::new(0, 0);
            for _i in 0..iters {
                for (stores, src_tr, dst_tr) in &dataset_trees {
                    let summary = run_diff_trees(stores, src_tr, dst_tr, "hybrid", 1000);
                    dbg!(&summary);
                    time += Duration::from_secs_f64(summary
                        .mapping_durations.mappings.0.get(1).unwrap().clone());
                }
            }
            time
        })
    });
    group.bench_function("simple", |b| {
        b.iter_custom(|iters| {
            let mut time = Duration::new(0, 0);
            for _i in 0..iters {
                for (stores, src_tr, dst_tr) in &dataset_trees {
                    let summary = run_diff_trees(stores, src_tr, dst_tr, "simple", DEFAULT_SIZE_THRESHOLD);
                    dbg!(&summary);
                    time += Duration::from_secs_f64(summary
                        .mapping_durations.mappings.0.get(1).unwrap().clone());
                }
            }
            time
        })
    });
    group.bench_function("greedy", |b| {
        b.iter_custom(|iters| {
            let mut time = Duration::new(0, 0);
            for _i in 0..iters {
                for (stores, src_tr, dst_tr) in &dataset_trees {
                    let summary = run_diff_trees(stores, src_tr, dst_tr, "greedy", DEFAULT_SIZE_THRESHOLD);
                    dbg!(&summary);
                    time += Duration::from_secs_f64(summary
                        .mapping_durations.mappings.0.get(1).unwrap().clone());
                }
            }
            time
        })
    });
    group.bench_function("lazy_greedy", |b| {
        b.iter_custom(|iters| {
            let mut time = Duration::new(0, 0);
            for _i in 0..iters {
                for (stores, src_tr, dst_tr) in &dataset_trees {
                    let summary = run_diff_trees(stores, src_tr, dst_tr, "lazy", DEFAULT_SIZE_THRESHOLD);
                    dbg!(&summary);
                    time += Duration::from_secs_f64(summary
                        .mapping_durations.mappings.0.get(1).unwrap().clone());
                }
            }
            time
        })
    });
    group.bench_function("lazy_hybrid_50", |b| {
        b.iter_custom(|iters| {
            let mut time = Duration::new(0, 0);
            for _i in 0..iters {
                for (stores, src_tr, dst_tr) in &dataset_trees {
                    let summary = run_diff_trees(stores, src_tr, dst_tr, "lazy_hybrid", 50);
                    dbg!(&summary);
                    time += Duration::from_secs_f64(summary
                        .mapping_durations.mappings.0.get(1).unwrap().clone());
                }
            }
            time
        })
    });
    group.bench_function("lazy_hybrid_100", |b| {
        b.iter_custom(|iters| {
            let mut time = Duration::new(0, 0);
            for _i in 0..iters {
                for (stores, src_tr, dst_tr) in &dataset_trees {
                    let summary = run_diff_trees(stores, src_tr, dst_tr, "lazy_hybrid", 100);
                    dbg!(&summary);
                    time += Duration::from_secs_f64(summary
                        .mapping_durations.mappings.0.get(1).unwrap().clone());
                }
            }
            time
        })
    });
    group.bench_function("lazy_hybrid_500", |b| {
        b.iter_custom(|iters| {
            let mut time = Duration::new(0, 0);
            for _i in 0..iters {
                for (stores, src_tr, dst_tr) in &dataset_trees {
                    let summary = run_diff_trees(stores, src_tr, dst_tr, "lazy_hybrid", 500);
                    dbg!(&summary);
                    time += Duration::from_secs_f64(summary
                        .mapping_durations.mappings.0.get(1).unwrap().clone());
                }
            }
            time
        })
    });
    group.bench_function("lazy_hybrid_1000", |b| {
        b.iter_custom(|iters| {
            let mut time = Duration::new(0, 0);
            for _i in 0..iters {
                for (stores, src_tr, dst_tr) in &dataset_trees {
                    let summary = run_diff_trees(stores, &src_tr, &dst_tr, "lazy_hybrid", 1000);
                    dbg!(&summary);
                    time += Duration::from_secs_f64(summary
                        .mapping_durations.mappings.0.get(1).unwrap().clone());
                }
            }
            time
        })
    });
    group.finish();
}

criterion_group!(benches, diff_benchmark);
criterion_main!(benches);