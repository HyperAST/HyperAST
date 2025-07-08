use std::hint::black_box;

use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
    measurement::Measurement,
};
use hyper_diff::{
    decompressed_tree_store::{CompletePostOrder, lazy_post_order::LazyPostOrder},
    matchers::{
        Decompressible,
        heuristic::gt::{greedy_bottom_up_matcher, lazy2_greedy_bottom_up_matcher},
    },
};
use hyperast::{
    store::nodes::legion::NodeIdentifier,
    types::{HyperAST as _, HyperASTShared, WithStats as _},
};
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;

#[allow(type_alias_bounds)]
type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
#[allow(type_alias_bounds)]
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;
type M = hyper_diff::matchers::mapping_store::VecStore<u32>;
type MM = hyper_diff::matchers::mapping_store::DefaultMultiMappingStore<u32>;

struct Input {
    repo: hyperast_vcs_git::git::Repo,
    commit: &'static str,
    config: hyperast_vcs_git::processing::RepoConfig,
    fetch: bool,
}

fn construction_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("Mapping_runtime");

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
    ];
    let mut repositories = PreProcessedRepositories::default();
    for p in inputs.into_iter() {
        repositories.register_config(p.repo.clone(), p.config);
    }
    for p in inputs.into_iter() {
        prep_bench_subtree(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("GreedyGumtree", p.repo.name()),
            |b, (repositories, (src, dst))| {
                // let hyperast = hyperast_vcs_git::no_space::as_nospaces2(&repositories.processor.main_stores);
                b.iter(
                    || {
                        let hyperast = &repositories.processor.main_stores;
                        let mut mapper_owned: (DS<_>, DS<_>) = hyperast.decompress_pair(&src, &dst).1;
                        let mapper =
                            hyper_diff::matchers::Mapper::with_mut_decompressible(&mut mapper_owned);

                        let mapper = hyper_diff::matchers::heuristic::gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);
                        let mappings = mapper.mapping.mappings;
                        let mapper = hyper_diff::matchers::Mapper::prep(
                            hyperast,
                            mappings,
                            mapper_owned,
                        );
                        let mapper = mapper.map(
                            |src_arena| CDS::<_>::from(src_arena.map(|x| x.decomp.complete(hyperast))),
                            |dst_arena| CDS::<_>::from(dst_arena.map(|x| x.decomp.complete(hyperast))),
                        );
                        let mapper_bottom_up = greedy_bottom_up_matcher::GreedyBottomUpMatcher::<
                            _,
                            _,
                            _,
                            _,
                            200,
                        >::match_it(mapper);
                        black_box(mapper_bottom_up);
                    },
                );
            },
        );
        prep_bench_subtree(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("LazyGumtree", p.repo.name()),
            |b, (repositories, (src, dst))| {
                // let hyperast = hyperast_vcs_git::no_space::as_nospaces2(&repositories.processor.main_stores);
                b.iter(
                    || {
                        let hyperast = &repositories.processor.main_stores;
                        let mut mapper_owned: (DS<_>, DS<_>) = hyperast.decompress_pair(&src, &dst).1;
                        let mapper =
                            hyper_diff::matchers::Mapper::with_mut_decompressible(&mut mapper_owned);

                        let mapper = hyper_diff::matchers::heuristic::gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);

                        let mapper_bottom_up =
                            lazy2_greedy_bottom_up_matcher::GreedyBottomUpMatcher::<
                                _,
                                _,
                                _,
                                M,
                                M,
                                200,
                            >::match_it(mapper);
                        black_box(mapper_bottom_up);
                    },
                );
            },
        );
    }
    group.finish()
}

fn prep_bench_subtree<Mea: Measurement>(
    group: &mut criterion::BenchmarkGroup<'_, Mea>,
    repositories: &mut PreProcessedRepositories,
    p: &Input,
    bid: BenchmarkId,
    f: impl FnMut(
        &mut criterion::Bencher<'_, Mea>,
        &(&PreProcessedRepositories, &(NodeIdentifier, NodeIdentifier)),
    ),
) {
    group.bench_with_input_prepared(
        bid,
        repositories,
        |group, repositories| {
            let (src, dst) = prep_commits(p, repositories);
            let hyperast = &repositories.processor.main_stores;
            group.throughput(Throughput::Elements(
                (hyperast.node_store().resolve(src).size()
                    + hyperast.node_store().resolve(dst).size())
                .div_ceil(2) as u64,
            ));
            (src, dst)
        },
        f,
    );
}

fn prep_commits(
    p: &Input,
    repositories: &mut PreProcessedRepositories,
) -> (NodeIdentifier, NodeIdentifier) {
    let repo = repositories
        .get_config((&p.repo).clone())
        .ok_or_else(|| "missing config for repository".to_string())
        .unwrap();
    let repository = if p.fetch {
        repo.fetch()
    } else {
        repo.nofetch()
    };

    let commits = repositories
        .pre_process_with_limit(&repository, "", &p.commit, 2)
        .unwrap();
    let src = repositories
        .get_commit(&repository.config, &commits[1])
        .unwrap()
        .ast_root;
    let dst = repositories
        .get_commit(&repository.config, &commits[0])
        .unwrap()
        .ast_root;
    (src, dst)
}

criterion_group!(
    name = construction;
    config = Criterion::default().sample_size(10).measurement_time(std::time::Duration::from_secs(10)).configure_from_args();
    targets = construction_group
);
criterion_main!(construction);
