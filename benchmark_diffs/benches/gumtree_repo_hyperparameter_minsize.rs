use criterion::{Criterion, criterion_group, criterion_main};
use hyper_diff::algorithms;
use hyperast::types::WithStats;
use hyperast_benchmark_diffs::preprocess_repo::parse_repo;
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;
use jemalloc_ctl::{epoch, stats};
use std::fs::File;
use std::hint::black_box;
use std::io::BufWriter;
use std::io::Write;
use std::time::Duration;

fn diff_benchmark(c: &mut Criterion) {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("trace"))
        .is_test(true)
        .init();
    let group = c.benchmark_group("gumtree_repo_hyperparameter_minsize");
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
            repo_user: "alibaba",
            repo_name: "arthas",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "0d6c1a63eb308531780ecf85f78e67f18303815c",
            after: "63ee8dfb19e94bcf867f55190fd8b01fd399afb2",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "FasterXML",
            repo_name: "jackson-core",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "373f108a16e3f315e9df9eaecb482e43f9953621",
            after: "0d9823619c4daa3f6aa9ee0d615f140978bcc51d",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "FasterXML",
            repo_name: "jackson-core",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "2272fcf675c3936568c855d2f8b3da58bb7713af",
            after: "6d2236ea9127757cbb85a6b60b42b5a597205d19",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "apache",
            repo_name: "skywalking",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "57746c24d3fd831835a3709ea3078fa26928f54e",
            after: "39508f81c8f8e04e86b670fb3877be28eaf5f01f",
        },
        BenchmarkItem {
            repositories: PreProcessedRepositories::default(),
            repo_user: "apache",
            repo_name: "skywalking",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            before: "47ce2720b9be6af391138c2b84c4ec63c454a3b3",
            after: "43d79d9fec224036cc3cbc7185c9faa7ecd4838c",
        },
    ];

    let mut buf_perfs =
        BufWriter::with_capacity(4 * 8 * 1024, File::create("/tmp/repo_minsize.csv").unwrap());
    writeln!(
        buf_perfs,
        "input,kind,min_height,src_s,dst_s,mappings,actions,prepare_topdown_t,topdown_t,prepare_bottomup_t,bottomup_t,prepare_gen_t,gen_t,total_mem",
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

    macro_rules! run_diff_for_thresholds {
    ($($threshold:expr),*) => {$({
        for ((stores, src_tr, dst_tr), before, after) in dataset_trees.iter() {
            println!("Comparing {} / {}, with min_size {}", before, after, $threshold);
            epoch::advance().unwrap();
            let before_allocated = stats::allocated::read().unwrap();
            let summary = algorithms::gumtree_hybrid::diff_hybrid_minheight::<_, {$threshold}>(
                black_box(stores),
                black_box(src_tr),
                black_box(dst_tr),
                100
            ).summarize();
            epoch::advance().unwrap();
            let after_allocated = stats::allocated::read().unwrap();
            let memory_used = after_allocated.saturating_sub(before_allocated);
            dbg!(&summary);
            dbg!(&memory_used);
            // time += Duration::from_secs_f64(summary
            //     .mapping_durations.mappings.0.get(1).unwrap().clone());
            write_perfs(
                &mut buf_perfs,
                "hybrid_minheight",
                $threshold,
                &before,
                &after,
                stores.node_store.resolve(*src_tr).size(),
                stores.node_store.resolve(*dst_tr).size(),
                &summary,
                memory_used
            ).unwrap();
            buf_perfs.flush().unwrap();
        }
        })*};
    }

    run_diff_for_thresholds!(0, 1, 2, 3, 4, 5);
    group.finish();
}

fn write_perfs(
    buf_perfs: &mut BufWriter<File>,
    kind: &str,
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
