mod bench_utils;
use std::{fmt::Debug, path::PathBuf};

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
use criterion_perf_events::Perf;
use perfcnt::linux::HardwareEventType as Hardware;
use perfcnt::linux::PerfCounterBuilderLinux as Builder;

use hyper_diff::{
    algorithms,
    decompressed_tree_store::{CompletePostOrder, lazy_post_order::LazyPostOrder},
    matchers::{
        Decompressible, Mapper,
        heuristic::gt::{
            greedy_bottom_up_matcher::GreedyBottomUpMatcher,
            greedy_subtree_matcher::GreedySubtreeMatcher,
            lazy_simple_bottom_up_matcher::LazySimpleBottomUpMatcher,
            lazy2_greedy_bottom_up_matcher::GreedyBottomUpMatcher as LazyGreedyBottomUpMatcher,
            simple_bottom_up_matcher3::SimpleBottomUpMatcher,
        },
        mapping_store::{DefaultMultiMappingStore, VecStore},
    },
};
use hyperast::{
    store::SimpleStores,
    types::{self, HyperAST, HyperASTShared, NodeId},
};
use hyperast_benchmark_diffs::preprocess::{JavaPreprocessFileSys, parse_dir_pair};
use hyperast_gen_ts_java::legion_with_refs::Local;

use crate::bench_utils::{DataSet, Heuristic, HeuristicType};

#[allow(type_alias_bounds)]
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;
#[allow(type_alias_bounds)]
type DS<HAST: HyperASTShared> = Decompressible<HAST, LazyPostOrder<HAST::IdN, u32>>;
type M = VecStore<u32>;
type MM = DefaultMultiMappingStore<u32>;

fn prepare_stores_java(dataset_path: PathBuf) -> (JavaPreprocessFileSys, Local, Local) {
    assert!(dataset_path.join("before").exists());
    assert!(dataset_path.join("after").exists());
    let stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let md_cache = Default::default();
    let mut java_gen = JavaPreprocessFileSys {
        main_stores: stores,
        java_md_cache: md_cache,
    };

    let (src, dst) = (dataset_path.join("before"), dataset_path.join("after"));
    let (src, dst) = parse_dir_pair(&mut java_gen, &src, &dst);
    (java_gen, src, dst)
}

fn do_top_down_greedy<HAST: HyperAST + Copy>(
    hyperast: HAST,
    src: &HAST::IdN,
    dst: &HAST::IdN,
) -> Mapper<HAST, CDS<HAST>, CDS<HAST>, VecStore<u32>>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    let mapper: Mapper<_, CDS<HAST>, CDS<HAST>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper)
}

/// This trampoline function is needed because of the generic HAST fml.
pub fn lazy_top_down<'a, HAST: HyperAST + Copy + 'a>(
    mapper_owned: &'a mut (DS<HAST>, DS<HAST>),
) -> Mapper<
    HAST,
    Decompressible<HAST, &'a mut LazyPostOrder<<HAST as HyperASTShared>::IdN, u32>>,
    Decompressible<HAST, &'a mut LazyPostOrder<<HAST as HyperASTShared>::IdN, u32>>,
    VecStore<u32>,
>
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Label: Clone + Copy + Eq + Debug,
    HAST::Idx: hyperast::PrimInt,
    for<'t> types::LendT<'t, HAST>: types::WithHashs + types::WithStats,
{
    algorithms::gumtree_lazy::lazy_top_down(mapper_owned)
}

fn run_all_heuristics_gh_java(c: &mut Criterion<Perf>) {
    let mut group = c.benchmark_group("gh_java_group");
    let dataset = DataSet::Defects4J(Some("drool"));

    let (java_gen, src, dst) = prepare_stores_java(dataset.get_path_dataset_project());
    let stores = hyperast_vcs_git::no_space::as_nospaces2(&java_gen.main_stores);

    let greedy_mapper = do_top_down_greedy(
        stores.clone(),
        &src.clone().compressed_node,
        &dst.clone().compressed_node,
    );

    let mut mapper_owned = stores
        .clone()
        .decompress_pair(&src.clone().compressed_node, &dst.clone().compressed_node)
        .1;
    let lazy_mapper = lazy_top_down(&mut mapper_owned);

    for heuristic in [
        Heuristic::Greedy,
        Heuristic::Simple,
        Heuristic::LazyGreedy,
        Heuristic::LazySimple,
    ] {
        match heuristic.get_heuristic_type() {
            HeuristicType::Lazy => group.bench_with_input(
                BenchmarkId::new(format!("{}", heuristic), dataset),
                &lazy_mapper.clone(),
                |b, mapper| {
                    b.iter_batched(
                        || lazy_mapper.clone(),
                        |mapper| {
                            let output = match heuristic {
                                Heuristic::LazyGreedy => {
                                    LazyGreedyBottomUpMatcher::<_, _, _, _>::match_it(mapper)
                                }
                                Heuristic::LazySimple => {
                                    LazySimpleBottomUpMatcher::<_, _, _, _>::match_it(mapper)
                                }
                                other => {
                                    panic!("Received an unexpected heuristic, got {}", other)
                                }
                            };
                            black_box(output);
                        },
                        criterion::BatchSize::SmallInput,
                    );
                },
            ),
            HeuristicType::Greedy => group.bench_with_input(
                BenchmarkId::new(format!("{}", heuristic), dataset),
                &greedy_mapper.clone(),
                |b, mapper| {
                    b.iter_batched(
                        || greedy_mapper.clone(),
                        |mapper| {
                            let output = match heuristic {
                                Heuristic::Greedy => {
                                    GreedyBottomUpMatcher::<_, _, _, _>::match_it(mapper)
                                }
                                Heuristic::Simple => {
                                    SimpleBottomUpMatcher::<_, _, _, _>::match_it(mapper)
                                }
                                other => {
                                    panic!("Received an unexpected heuristic, got {}", other)
                                }
                            };
                            black_box(output);
                        },
                        criterion::BatchSize::SmallInput,
                    );
                },
            ),
        };

        group.finish();
    }
}

criterion_group!(
    name = gh_java_all_heuristic;
    config = Criterion::default().with_measurement(Perf::new(Builder::from_hardware_event(Hardware::Instructions)));
    targets = run_all_heuristics_gh_java
);
criterion_main!(gh_java_all_heuristic);
