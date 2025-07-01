use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use hyperast::types::WithStats;
use hyperast_benchmark_diffs::preprocess_repo::parse_repo;
use hyperast_benchmark_diffs::run_diff::run_diff_trees;
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;
use jemalloc_ctl::{epoch, stats};
use std::fmt::Display;
use std::fs::File;
use std::io::BufWriter;
use std::io::Write;
use std::time::Duration;

// #[global_allocator]
// static GLOBAL: Jemalloc = Jemalloc;

fn diff_benchmark(c: &mut Criterion) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    let mut group = c.benchmark_group("gumtree_repo_hyperparameter_maxsize");

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
            before: "90457e519a7bc130de14ace69d26368ac28ead51",
            after: "e90830280a4eb28b21581c6a089b250410c5f0e2",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "apache",
            repo_name: "maven",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "3e9c164ba3bc9bdab7503807bb74041a39a6ca68",
            after: "51bac9714a6deebd61589a7f8163b2e759751208",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "apache",
            repo_name: "maven",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "4ac3b14be2668ea70740dd94e486dc877b83d38a",
            after: "92fa43d143fc1a94efbdfd4bb65b30d48da329f2",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "quarkusio",
            repo_name: "quarkus",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "a0659dba3ff3df590088262f42329efa0b4b30e9",
            after: "be1bda0f121ac24cb789b103e216151b53c0a076",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "quarkusio",
            repo_name: "quarkus",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "93cb61373d8cf3f6bddea7a32b7a528fea1fbd33",
            after: "0345645705655c8a3e84476b53f14f35cce0fb5a",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "qos-ch",
            repo_name: "slf4j",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "9349d64f95d0ee4e6ee0057e7376d15f1d15b37c",
            after: "0def25ebfa0e546525fb90aa8d5946d16f26c561",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "qos-ch",
            repo_name: "slf4j",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "03aa6b915a82a037d2936ca0b166626d32e9a1f6",
            after: "7c3b0ef011fd4da98579b7cb55a1edb92fb9a9df",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "INRIA",
            repo_name: "spoon",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "32980c548e737cdf39c681f6b42d8a39ab97e8c5",
            after: "f4477395fae949332416603ac503675d575018ab",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "INRIA",
            repo_name: "spoon",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "5622951ebbbc9649858e62edb6924511736badde",
            after: "5dbcfe1770c200c27eb7a2dbe4d4d3865fca7d3b",
        },
    ];

    let mut buf_perfs =
        BufWriter::with_capacity(4 * 8 * 1024, File::create("/tmp/repo_maxsize.csv").unwrap());
    writeln!(
        buf_perfs,
        "input,kind,max_size,src_s,dst_s,mappings,actions,prepare_topdown_t,topdown_t,prepare_bottomup_t,bottomup_t,prepare_gen_t,gen_t,total_mem",
    )
        .unwrap();

    let dataset_trees: Vec<_> = dataset
        .iter_mut()
        .map(|commit| {
            (
                parse_repo(
                    &mut commit.repositories,
                    commit.repo_user,
                    commit.repo_name,
                    commit.config,
                    commit.before,
                    commit.after,
                ),
                commit.before,
                commit.after,
            )
        })
        .collect();

    let values = vec![0, 50, 100, 200, 300, 500, 700, 1000, 1500];

    for max_size in values {
        group.bench_with_input(
            BenchmarkId::new("repo_hybrid_hyperparameter", max_size),
            &max_size,
            |b, _i| {
                b.iter_custom(|iters| {
                    let mut time = Duration::ZERO;
                    for _ in 0..iters {
                        for ((stores, src_tr, dst_tr), before, after) in dataset_trees.iter() {
                            epoch::advance().unwrap();
                            let before_allocated = stats::allocated::read().unwrap();
                            let algo = hyperast_benchmark_diffs::run_diff::Algorithm::Hybrid;
                            let summary = run_diff_trees(stores, src_tr, dst_tr, algo, max_size);
                            epoch::advance().unwrap();
                            let after_allocated = stats::allocated::read().unwrap();
                            let memory_used = after_allocated.saturating_sub(before_allocated);
                            dbg!(&summary);
                            dbg!(&memory_used);
                            time += *summary.mapping_durations.mappings.0.get(1).unwrap();
                            write_perfs(
                                &mut buf_perfs,
                                algo,
                                max_size,
                                &before,
                                &after,
                                stores.node_store.resolve(*src_tr).size(),
                                stores.node_store.resolve(*dst_tr).size(),
                                &summary,
                                memory_used,
                            )
                            .unwrap();
                            buf_perfs.flush().unwrap();
                        }
                    }
                    time
                })
            },
        );
    }

    group.finish();
}

fn write_perfs(
    buf_perfs: &mut BufWriter<File>,
    kind: impl Display,
    max_size: usize,
    oid_src: &str,
    oid_dst: &str,
    src_s: usize,
    dst_s: usize,
    summarized_lazy: &hyper_diff::algorithms::ResultsSummary<
        hyper_diff::algorithms::PreparedMappingDurations<2, Duration>,
        Duration,
    >,
    total_mem: usize,
) -> Result<(), std::io::Error> {
    writeln!(
        buf_perfs,
        "{}/{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
        oid_src,
        oid_dst,
        kind,
        max_size,
        src_s,
        dst_s,
        summarized_lazy.mappings,
        summarized_lazy.actions.map_or(-1, |x| x as isize),
        summarized_lazy.mapping_durations.preparation[0].as_secs_f64(),
        summarized_lazy.mapping_durations.mappings.0[0].as_secs_f64(),
        summarized_lazy.mapping_durations.preparation[1].as_secs_f64(),
        summarized_lazy.mapping_durations.mappings.0[1].as_secs_f64(),
        summarized_lazy.prepare_gen_t.as_secs_f64(),
        summarized_lazy.gen_t.as_secs_f64(),
        total_mem,
    )
}

criterion_group!(
    name = benches;
    config = Criterion::default().sample_size(10).configure_from_args();
    targets = diff_benchmark
);
criterion_main!(benches);
