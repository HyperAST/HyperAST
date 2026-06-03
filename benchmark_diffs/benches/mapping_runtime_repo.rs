use std::hint::black_box;

use criterion::{
    BenchmarkId, Criterion, Throughput, criterion_group, criterion_main, measurement::Measurement,
};
use hyper_diff::{
    decompressed_tree_store::{CompletePostOrder, lazy_post_order::LazyPostOrder},
    matchers::{Decompressible, mapping_store::VecStore},
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

fn mapping_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("Mapping_runtime");

    let inputs: &[Input] = &[
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("chromium", "chromium"),
        //     commit: "f461f9752e5918c5c87f2e3767bcb24945ee0fa0",
        //     config: hyperast_vcs_git::processing::RepoConfig::CppMake,
        //     fetch: false,
        // },
        Input {
            repo: hyperast_vcs_git::git::Forge::Github.repo("apache", "maven"),
            commit: "c3cf29438e3d65d6ee5c5726f8611af99d9a649a",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            fetch: true,
        },
        Input {
            repo: hyperast_vcs_git::git::Forge::Github.repo("INRIA", "spoon"),
            commit: "56e12a0c0e0e69ea70863011b4f4ca3305e0542b",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            fetch: true,
        },
        Input {
            repo: hyperast_vcs_git::git::Forge::Github.repo("javaparser", "javaparser"),
            commit: "046bf8be251189452ad6b25bf9107a1a2167ce6f",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            fetch: true,
        },
        Input {
            repo: hyperast_vcs_git::git::Forge::Github.repo("apache", "dubbo"),
            commit: "e831b464837ae5d2afac9841559420aeaef6c52b",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            fetch: true,
        },
        Input {
            repo: hyperast_vcs_git::git::Forge::Github.repo("netty", "netty"),
            commit: "c2b846750dd2131d65aa25c8cf66bf3649b248f9",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            fetch: true,
        },
        Input {
            repo: hyperast_vcs_git::git::Forge::Github.repo("apache", "hadoop"),
            commit: "d5e97fe4d6baf43a5576cbd1700c22b788dba01e",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            fetch: true,
        },
    ];
    let mut repositories = PreProcessedRepositories::default();
    for p in inputs.into_iter() {
        repositories.register_config(p.repo.clone(), p.config);
    }
    for p in inputs.into_iter() {
        use hyper_diff::matchers::heuristic::cd;
        use hyper_diff::matchers::heuristic::gt;

        prep_bench(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("Xy", p.repo.name()),
            |b, (repositories, (src, dst))| {
                b.iter(|| {
                    // let hyperast = hyperast_vcs_git::no_space::as_nospaces2(&repositories.processor.main_stores);
                    let hyperast = &repositories.processor.main_stores;
                    let mapper_owned: (CDS<_>, CDS<_>) = hyperast.decompress_pair(&src, &dst).1;
                    let mapper = hyper_diff::matchers::Mapper::new(
                        hyperast,
                        VecStore::default(),
                        mapper_owned,
                    );

                    use gt::greedy_subtree_matcher::GreedySubtreeMatcher;
                    let mapper = GreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);
                    use hyper_diff::matchers::heuristic::xy_bottom_up_matcher::XYBottomUpMatcher;
                    let mapper_bottom_up = XYBottomUpMatcher::<_, _, _, _>::match_it(mapper);
                    black_box(mapper_bottom_up);
                });
            },
        );
        prep_bench(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("PartialLazyXy", p.repo.name()),
            |b, (repositories, (src, dst))| {
                b.iter(|| {
                    let hyperast = &repositories.processor.main_stores;
                    let mut mapper_owned: (DS<_>, DS<_>) = hyperast.decompress_pair(&src, &dst).1;
                    let mapper =
                        hyper_diff::matchers::Mapper::with_mut_decompressible(&mut mapper_owned);

                    use gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
                    let mapper = LazyGreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);
                    let mappings = mapper.mapping.mappings;
                    let mapper =
                        hyper_diff::matchers::Mapper::prep(hyperast, mappings, mapper_owned);
                    let mapper = mapper.map(
                        |src_arena| CDS::<_>::from(src_arena.map(|x| x.decomp.complete(hyperast))),
                        |dst_arena| CDS::<_>::from(dst_arena.map(|x| x.decomp.complete(hyperast))),
                    );
                    use hyper_diff::matchers::heuristic::xy_bottom_up_matcher::XYBottomUpMatcher;
                    let mapper_bottom_up = XYBottomUpMatcher::<_, _, _, _>::match_it(mapper);
                    black_box(mapper_bottom_up);
                });
            },
        );
        prep_bench(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("GreedyGumtree", p.repo.name()),
            |b, (repositories, (src, dst))| {
                b.iter(|| {
                    // let hyperast = hyperast_vcs_git::no_space::as_nospaces2(&repositories.processor.main_stores);
                    let hyperast = &repositories.processor.main_stores;
                    let mapper_owned: (CDS<_>, CDS<_>) = hyperast.decompress_pair(&src, &dst).1;
                    let mapper = hyper_diff::matchers::Mapper::new(
                        hyperast,
                        VecStore::default(),
                        mapper_owned,
                    );

                    use gt::greedy_subtree_matcher::GreedySubtreeMatcher;
                    let mapper = GreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);
                    use gt::greedy_bottom_up_matcher::GreedyBottomUpMatcher;
                    let mapper_bottom_up =
                        GreedyBottomUpMatcher::<_, _, _, _, 200>::match_it(mapper);
                    black_box(mapper_bottom_up);
                });
            },
        );
        prep_bench(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("PartialLazyGreedyGumtree", p.repo.name()),
            |b, (repositories, (src, dst))| {
                b.iter(|| {
                    let hyperast = &repositories.processor.main_stores;
                    let mut mapper_owned: (DS<_>, DS<_>) = hyperast.decompress_pair(&src, &dst).1;
                    let mapper =
                        hyper_diff::matchers::Mapper::with_mut_decompressible(&mut mapper_owned);

                    use gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
                    let mapper = LazyGreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);
                    let mappings = mapper.mapping.mappings;
                    let mapper =
                        hyper_diff::matchers::Mapper::prep(hyperast, mappings, mapper_owned);
                    let mapper = mapper.map(
                        |src_arena| CDS::<_>::from(src_arena.map(|x| x.decomp.complete(hyperast))),
                        |dst_arena| CDS::<_>::from(dst_arena.map(|x| x.decomp.complete(hyperast))),
                    );
                    use gt::greedy_bottom_up_matcher::GreedyBottomUpMatcher;
                    let mapper_bottom_up =
                        GreedyBottomUpMatcher::<_, _, _, _, 200>::match_it(mapper);
                    black_box(mapper_bottom_up);
                });
            },
        );
        prep_bench(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("LazyGreedyGumtree", p.repo.name()),
            |b, (repositories, (src, dst))| {
                b.iter(|| {
                    let hyperast = &repositories.processor.main_stores;
                    let mut mapper_owned: (DS<_>, DS<_>) = hyperast.decompress_pair(&src, &dst).1;
                    let mapper =
                        hyper_diff::matchers::Mapper::with_mut_decompressible(&mut mapper_owned);

                    use gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
                    let mapper = LazyGreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);

                    use gt::lazy2_greedy_bottom_up_matcher::LazyGreedyBottomUpMatcher;
                    let mapper_bottom_up =
                        LazyGreedyBottomUpMatcher::<_, _, _, M, M, 200>::match_it(mapper);
                    black_box(mapper_bottom_up);
                });
            },
        );
        prep_bench(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("PartialLazyHybridGumtree", p.repo.name()),
            |b, (repositories, (src, dst))| {
                b.iter(|| {
                    let hyperast = &repositories.processor.main_stores;
                    let mut mapper_owned: (DS<_>, DS<_>) = hyperast.decompress_pair(&src, &dst).1;
                    let mapper =
                        hyper_diff::matchers::Mapper::with_mut_decompressible(&mut mapper_owned);

                    use gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
                    let mapper = LazyGreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);
                    let mappings = mapper.mapping.mappings;
                    let mapper =
                        hyper_diff::matchers::Mapper::prep(hyperast, mappings, mapper_owned);
                    let mapper = mapper.map(
                        |src_arena| CDS::<_>::from(src_arena.map(|x| x.decomp.complete(hyperast))),
                        |dst_arena| CDS::<_>::from(dst_arena.map(|x| x.decomp.complete(hyperast))),
                    );
                    use gt::hybrid_bottom_up_matcher::HybridBottomUpMatcher;
                    let mapper_bottom_up =
                        HybridBottomUpMatcher::<_, _, _, _, M, 200>::match_it(mapper);
                    black_box(mapper_bottom_up);
                });
            },
        );
        prep_bench(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("LazyHybridGumtree", p.repo.name()),
            |b, (repositories, (src, dst))| {
                b.iter(|| {
                    let hyperast = &repositories.processor.main_stores;
                    let mut mapper_owned: (DS<_>, DS<_>) = hyperast.decompress_pair(&src, &dst).1;
                    let mapper =
                        hyper_diff::matchers::Mapper::with_mut_decompressible(&mut mapper_owned);

                    use gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
                    let mapper = LazyGreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);

                    use gt::lazy_hybrid_bottom_up_matcher::LazyHybridBottomUpMatcher;
                    let mapper_bottom_up =
                        LazyHybridBottomUpMatcher::<_, _, _, M, M, 200>::match_it(mapper);
                    black_box(mapper_bottom_up);
                });
            },
        );
        prep_bench(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("PartialLazyStableGumtree", p.repo.name()),
            |b, (repositories, (src, dst))| {
                b.iter(|| {
                    let hyperast = &repositories.processor.main_stores;
                    let mut mapper_owned: (DS<_>, DS<_>) = hyperast.decompress_pair(&src, &dst).1;
                    let mapper =
                        hyper_diff::matchers::Mapper::with_mut_decompressible(&mut mapper_owned);

                    use gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
                    let mapper = LazyGreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);
                    let mappings = mapper.mapping.mappings;
                    let mapper =
                        hyper_diff::matchers::Mapper::prep(hyperast, mappings, mapper_owned);
                    let mapper = mapper.map(
                        |src_arena| CDS::<_>::from(src_arena.map(|x| x.decomp.complete(hyperast))),
                        |dst_arena| CDS::<_>::from(dst_arena.map(|x| x.decomp.complete(hyperast))),
                    );
                    use gt::marriage_bottom_up_matcher::MarriageBottomUpMatcher;
                    let mapper_bottom_up =
                        MarriageBottomUpMatcher::<_, _, _, _, M, 200>::match_it(mapper);
                    black_box(mapper_bottom_up);
                });
            },
        );
        prep_bench(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("LazyStableGumtree", p.repo.name()),
            |b, (repositories, (src, dst))| {
                b.iter(|| {
                    let hyperast = &repositories.processor.main_stores;
                    let mut mapper_owned: (DS<_>, DS<_>) = hyperast.decompress_pair(&src, &dst).1;
                    let mapper =
                        hyper_diff::matchers::Mapper::with_mut_decompressible(&mut mapper_owned);

                    use gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
                    let mapper = LazyGreedySubtreeMatcher::<_, _, _, M>::match_it::<MM>(mapper);

                    use gt::lazy_marriage_bottom_up_matcher::LazyMarriageBottomUpMatcher;
                    let mapper_bottom_up =
                        LazyMarriageBottomUpMatcher::<_, _, _, M, M, 200>::match_it(mapper);
                    black_box(mapper_bottom_up);
                });
            },
        );

        prep_bench(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("ChangeDistiller", p.repo.name()),
            |b, (repositories, (src, dst))| {
                b.iter(|| {
                    let hyperast = &repositories.processor.main_stores;
                    let mapper_owned: (CDS<_>, CDS<_>) = hyperast.decompress_pair(&src, &dst).1;
                    let mapper = hyper_diff::matchers::Mapper::new(
                        hyperast,
                        VecStore::default(),
                        mapper_owned,
                    );

                    use cd::leaves_matcher::LeavesMatcher;
                    let mapper = LeavesMatcher::<_, _, _, M>::match_it(mapper);
                    use cd::bottom_up_matcher::BottomUpMatcher;
                    let mapper_bottom_up = BottomUpMatcher::<_, _, _, _, 200>::match_it(mapper);
                    black_box(mapper_bottom_up);
                });
            },
        );
        prep_bench(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("PartialLazyChangeDistiller", p.repo.name()),
            |b, (repositories, (src, dst))| {
                b.iter(|| {
                    let hyperast = &repositories.processor.main_stores;
                    let mut mapper_owned: (DS<_>, DS<_>) = hyperast.decompress_pair(&src, &dst).1;
                    let mapper =
                        hyper_diff::matchers::Mapper::with_mut_decompressible(&mut mapper_owned);
                    use cd::lazy_leaves_matcher::LazyLeavesMatcher;
                    let mapper = LazyLeavesMatcher::<_, _, _, M>::match_it(mapper);
                    let mappings = mapper.mapping.mappings;
                    let mapper =
                        hyper_diff::matchers::Mapper::prep(hyperast, mappings, mapper_owned);
                    let mapper = mapper.map(
                        |src_arena| CDS::<_>::from(src_arena.map(|x| x.decomp.complete(hyperast))),
                        |dst_arena| CDS::<_>::from(dst_arena.map(|x| x.decomp.complete(hyperast))),
                    );
                    use cd::bottom_up_matcher::BottomUpMatcher;
                    let mapper_bottom_up = BottomUpMatcher::<_, _, _, _>::match_it(mapper);
                    black_box(mapper_bottom_up);
                });
            },
        );
        prep_bench(
            &mut group,
            &mut repositories,
            &p,
            BenchmarkId::new("LazyChangeDistiller", p.repo.name()),
            |b, (repositories, (src, dst))| {
                b.iter(|| {
                    let hyperast = &repositories.processor.main_stores;
                    let mut mapper_owned: (DS<_>, DS<_>) = hyperast.decompress_pair(&src, &dst).1;
                    let mapper =
                        hyper_diff::matchers::Mapper::with_mut_decompressible(&mut mapper_owned);
                    use cd::lazy_leaves_matcher::LazyLeavesMatcher;
                    let mapper = LazyLeavesMatcher::<_, _, _, M>::match_it(mapper);

                    use cd::lazy_bottom_up_matcher::BottomUpMatcher;
                    let mapper_bottom_up = BottomUpMatcher::<_, _, _, M>::match_it(mapper);
                    black_box(mapper_bottom_up);
                });
            },
        );
    }
    group.finish()
}

fn prep_bench<Mea: Measurement>(
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
    let repository = if p.fetch
        && repositories
            .get_commit(
                &repo.config,
                &hyperast_vcs_git::git::Oid::from_str(p.commit).unwrap(),
            )
            .is_none()
    {
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

#[cfg(target_os = "linux")]
criterion_group!(
    name = mapping;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10))
        .with_measurement(criterion_perf_events::Perf::new(
            perfcnt::linux::PerfCounterBuilderLinux::from_hardware_event(
                perfcnt::linux::HardwareEventType::Instructions
            )
        ))
        .configure_from_args();
    targets = mapping_group
);
#[cfg(not(target_os = "linux"))]
criterion_group!(
    name = mapping;
    config = Criterion::default()
        .sample_size(10)
        .measurement_time(std::time::Duration::from_secs(10))
        .configure_from_args();
    targets = mapping_group
);
criterion_main!(mapping);
