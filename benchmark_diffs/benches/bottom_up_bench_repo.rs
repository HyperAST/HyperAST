mod data_bench;
use criterion::measurement::Measurement;
use criterion::{
    BatchSize, BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
};
use hyper_diff::decompressed_tree_store::CompletePostOrder;
use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
use hyper_diff::matchers::heuristic::gt::{
    greedy_bottom_up_matcher::GreedyBottomUpMatcher as GumtreeGreedy,
    lazy_simple_bottom_up_matcher::LazySimpleBottomUpMatcher,
    lazy2_greedy_bottom_up_matcher::GreedyBottomUpMatcher as LazyGreedyBottomUpMatcher,
    lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
    simple_bottom_up_matcher3::SimpleBottomUpMatcher as GumtreeSimple,
};
use hyper_diff::matchers::mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore};
use hyper_diff::matchers::{Decompressible, Mapper};
use hyperast::types::{self, WithStats as _};
use hyperast::types::{HyperAST, HyperASTShared, NodeId};
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;
use serde::Serialize;
use serde_json;
use std::hint::black_box;
use std::{
    collections::HashMap,
    error::Error,
    fmt::Debug,
    fs::{self, File},
    path::Path,
    usize,
};

#[allow(type_alias_bounds)]
type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
#[allow(type_alias_bounds)]
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;
type M = VecStore<u32>;
type MM = DefaultMultiMappingStore<u32>;

#[derive(Serialize)]
struct MappingInfo {
    #[serde(flatten)]
    id: MappingId,
    #[serde(flatten)]
    data: MappingData,
}
#[derive(Serialize, Hash, Clone, Debug, Eq, PartialEq)]
struct MappingId {
    algorithm: String,
    repo_name: String,
}
#[derive(Serialize)]
struct MappingData {
    num_pre_bottom_up: usize,
    num_post_bottom_up: usize,
}

fn log_results<'a>(
    data: HashMap<MappingId, MappingData>,
    algo: hyperast_benchmark_diffs::Algorithm,
    max_size: usize,
) -> Result<(), Box<dyn Error>> {
    let dir_path = Path::new("bench_results");
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)?;
    }
    let data = data
        .into_iter()
        .map(|(id, data)| MappingInfo { id, data })
        .collect::<Vec<_>>();
    let file_name = if max_size == 0 {
        format!("{algo}.json")
    } else {
        format!("{algo}_{max_size}.json")
    };
    let file_path = dir_path.join(file_name);
    let file = File::create(file_path)?;
    serde_json::to_writer_pretty(file, &data)?;
    Ok(())
}

fn bottom_up_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("bottom_up_repo");
    struct Input {
        repo: hyperast_vcs_git::git::Repo,
        commit: &'static str,
        config: hyperast_vcs_git::processing::RepoConfig,
        fetch: bool,
    }

    let inputs: &[Input] = &[
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("chromium", "chromium"),
        //     commit: "f461f9752e5918c5c87f2e3767bcb24945ee0fa0",
        //     config: hyperast_vcs_git::processing::RepoConfig::CppMake,
        //     fetch: false,
        // },
        Input {
            repo: hyperast_vcs_git::git::Forge::Github.repo("INRIA", "spoon"),
            commit: "56e12a0c0e0e69ea70863011b4f4ca3305e0542b",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            fetch: true,
        },
        Input {
            repo: hyperast_vcs_git::git::Forge::Github.repo("apache", "maven"),
            commit: "c3cf29438e3d65d6ee5c5726f8611af99d9a649a",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            fetch: true,
        },
    ];
    let mut repositories = PreProcessedRepositories::default();

    let inputs = inputs
        .into_iter()
        .map(|p| {
            repositories.register_config(p.repo.clone(), p.config);
            let repo = repositories
                .get_config((&p.repo).clone())
                .ok_or_else(|| "missing config for repository".to_string())
                .unwrap();
            let repository = if p.fetch {
                repo.fetch()
            } else {
                repo.nofetch()
            };
            (repository, p.commit)
        })
        .collect::<Vec<_>>();

    use hyperast_benchmark_diffs::Heuristic;
    use hyperast_benchmark_diffs::Opti;
    let algo = hyperast_benchmark_diffs::Algorithm(Opti::None, Heuristic::Simple);
    let max_size = 0;
    let results = bench_bottom_up::<0>(&mut group, &mut repositories, &inputs, algo);
    log_results(results, algo, max_size).expect("Failed to log results");
    let algo = hyperast_benchmark_diffs::Algorithm(Opti::None, Heuristic::Greedy);
    let results = bench_bottom_up::<50>(&mut group, &mut repositories, &inputs, algo);
    log_results(results, algo, max_size).expect("Failed to log results");
    let results = bench_bottom_up::<100>(&mut group, &mut repositories, &inputs, algo);
    log_results(results, algo, max_size).expect("Failed to log results");
    let results = bench_bottom_up::<200>(&mut group, &mut repositories, &inputs, algo);
    log_results(results, algo, max_size).expect("Failed to log results");
    let results = bench_bottom_up::<400>(&mut group, &mut repositories, &inputs, algo);
    log_results(results, algo, max_size).expect("Failed to log results");
    let algo = hyperast_benchmark_diffs::Algorithm(Opti::Lazy, Heuristic::Simple);
    let max_size = 0;
    let results = bench_bottom_up::<0>(&mut group, &mut repositories, &inputs, algo);
    log_results(results, algo, max_size).expect("Failed to log results");
    let algo = hyperast_benchmark_diffs::Algorithm(Opti::Lazy, Heuristic::Greedy);
    let results = bench_bottom_up::<50>(&mut group, &mut repositories, &inputs, algo);
    log_results(results, algo, max_size).expect("Failed to log results");
    let results = bench_bottom_up::<100>(&mut group, &mut repositories, &inputs, algo);
    log_results(results, algo, max_size).expect("Failed to log results");
    let results = bench_bottom_up::<200>(&mut group, &mut repositories, &inputs, algo);
    log_results(results, algo, max_size).expect("Failed to log results");
    let results = bench_bottom_up::<400>(&mut group, &mut repositories, &inputs, algo);
    log_results(results, algo, max_size).expect("Failed to log results");
    group.finish();
}

fn bench_bottom_up<const MAX_SIZE: usize>(
    group: &mut BenchmarkGroup<impl Measurement>,
    repositories: &mut PreProcessedRepositories,
    inputs: &[(hyperast_vcs_git::processing::ConfiguredRepo2, &str)],
    algo: hyperast_benchmark_diffs::Algorithm,
) -> HashMap<MappingId, MappingData> {
    let mut results = HashMap::new();

    let alg = if MAX_SIZE == 0 {
        &algo.to_string()
    } else {
        &format!("{algo}_{MAX_SIZE}")
    };

    for (repository, commit) in inputs {
        let mut rw = pair_commit(commit, &repository.repo).unwrap();
        let commits = repositories.pre_process_chunk(&mut rw, &repository, usize::MAX);
        let bid = BenchmarkId::new(alg, repository.spec.name());
        let src = repositories
            .get_commit(&repository.config, &commits[1])
            .unwrap()
            .ast_root;
        let dst = repositories
            .get_commit(&repository.config, &commits[0])
            .unwrap()
            .ast_root;
        let hyperast = &repositories.processor.main_stores;
        group.throughput(Throughput::Elements(
            (hyperast.node_store().resolve(src).size() + hyperast.node_store().resolve(dst).size())
                .div_ceil(2) as u64,
        ));
        let mut mapper_owned: (DS<_>, DS<_>) = hyperast.decompress_pair(&src, &dst).1;
        let mapper = Mapper::with_mut_decompressible(&mut mapper_owned);

        let mapper = LazyGreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);
        let mappings = mapper.mapping.mappings.clone();

        group.throughput(Throughput::Elements(
            (mapper.mappings.capacity().0 + mapper.mappings.capacity().1).div_ceil(2) as u64,
        ));

        group.bench_with_input(
            bid,
            &(mapper_owned, mappings),
            |b, (mapper_owned, mappings)| {
                let num_mappings_pre = mappings.len();
                let mut num_mappings_post = None;
                use hyperast_benchmark_diffs::Heuristic;
                use hyperast_benchmark_diffs::Opti;
                match algo.0 {
                    Opti::None => {
                        b.iter_batched(
                            || {
                                let owned = mapper_owned.clone();
                                Mapper::new(hyperast, mappings.clone(), owned)
                            },
                            |mapper| {
                                let mapper = mapper.map(
                                    |src_arena| {
                                        CDS::<_>::from(src_arena.map(|x| x.complete(hyperast)))
                                    },
                                    |dst_arena| {
                                        CDS::<_>::from(dst_arena.map(|x| x.complete(hyperast)))
                                    },
                                );
                                let mapper_bottom_up = match algo.1 {
                                    Heuristic::Greedy => {
                                        GumtreeGreedy::<_, _, _, _, MAX_SIZE>::match_it(mapper)
                                    }
                                    Heuristic::Simple => {
                                        GumtreeSimple::<_, _, _, _>::match_it(mapper)
                                    }
                                    Heuristic::Hybrid => unimplemented!(),
                                };
                                if num_mappings_post.is_none() {
                                    num_mappings_post = Some(mapper_bottom_up.mappings.len());
                                }
                                black_box(mapper_bottom_up);
                            },
                            BatchSize::SmallInput,
                        );
                    }
                    Opti::Lazy => b.iter_batched(
                        || (mappings.clone(), mapper_owned.clone()),
                        |(mappings, mut arenas)| {
                            let mapper = Mapper::new(
                                hyperast,
                                mappings,
                                (arenas.0.as_mut(), arenas.1.as_mut()),
                            );
                            let mapper_bottom_up = match algo.1 {
                                Heuristic::Greedy => {
                                    LazyGreedyBottomUpMatcher::<_, _, _, _>::match_it(mapper)
                                }
                                Heuristic::Simple => {
                                    LazySimpleBottomUpMatcher::<_, _, _, _>::match_it(mapper)
                                }
                                Heuristic::Hybrid => {
                                    unimplemented!()
                                }
                            };
                            if num_mappings_post.is_none() {
                                num_mappings_post = Some(mapper_bottom_up.mappings.len());
                            }
                            black_box(mapper_bottom_up);
                        },
                        BatchSize::SmallInput,
                    ),
                };
                let id = MappingId {
                    algorithm: algo.to_string(),
                    repo_name: repository.spec.name().to_string(),
                };
                if let Some(num_mappings_post) = num_mappings_post {
                    if !results.contains_key(&id) {
                        results.insert(
                            id,
                            MappingData {
                                num_pre_bottom_up: num_mappings_pre,
                                num_post_bottom_up: num_mappings_post,
                            },
                        );
                    }
                }
            },
        );
    }

    return results;
}

fn pair_commit<'repo>(
    commit: &str,
    repository: &'repo hyperast_vcs_git::git::Repository,
) -> Result<impl Iterator<Item = hyperast_vcs_git::git::Oid> + 'repo, hyperast_vcs_git::git::Error>
{
    Ok(hyperast_vcs_git::git::Builder::new(repository)?
        .after(commit)?
        .first_parents()?
        .walk()?
        .take(2)
        .map(|x| x.expect("a valid commit oid")))
}

criterion_group!(
    name = bottom_up;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10))
        .configure_from_args();
    targets = bottom_up_group
);
criterion_main!(bottom_up);
