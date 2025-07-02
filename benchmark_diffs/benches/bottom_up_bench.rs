mod data_bench;
use criterion::{
    BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
    measurement::Measurement,
};
use hyper_diff::{
    decompressed_tree_store::CompletePostOrder,
    matchers::{
        Decompressible, Mapper,
        heuristic::gt::{
            greedy_bottom_up_matcher::GreedyBottomUpMatcher as GumtreeGreedy,
            greedy_subtree_matcher::GreedySubtreeMatcher,
            simple_bottom_up_matcher3::SimpleBottomUpMatcher as GumtreeSimple,
        },
        mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore},
    },
};
use hyperast::types::{HyperAST, HyperASTShared, NodeId};
use hyperast::{store::SimpleStores, types};
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use serde::Serialize;
use serde_json;
use std::{
    collections::HashMap,
    error::Error,
    fmt::Debug,
    fs::{self, File},
    path::Path,
    usize,
};

#[allow(type_alias_bounds)]
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;

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
    file_pair: String,
}
#[derive(Serialize)]
struct MappingData {
    num_pre_bottom_up: usize,
    num_post_bottom_up: usize,
}

fn log_results<'a>(
    data: HashMap<MappingId, MappingData>,
    algo: &str,
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

fn benchmark_simple(c: &mut Criterion) {
    let mut group = c.benchmark_group("bottom_up_simple");
    let algo = "gumtree_simple";
    let max_size = 0;
    let results = bench_bottom_up(&mut group, algo, max_size);
    log_results(results, algo, max_size).expect("Failed to log results");
    group.finish();
}

fn benchmark_greedy(c: &mut Criterion) {
    let mut group = c.benchmark_group("bottom_up_greedy");
    let algo = "gumtree_greedy";
    let max_size = 200;
    let results = bench_bottom_up(&mut group, algo, max_size);
    log_results(results, algo, max_size).expect("Failed to log results");
    group.finish();
}

fn benchmarks(c: &mut Criterion) {
    let mut group = c.benchmark_group("bottom_up");
    let algo = "gumtree_simple";
    let max_size = 0;
    let results = bench_bottom_up(&mut group, algo, max_size);
    log_results(results, algo, max_size).expect("Failed to log results");
    let algo = "gumtree_greedy";
    let max_size = 50;
    let results = bench_bottom_up(&mut group, algo, max_size);
    log_results(results, algo, max_size).expect("Failed to log results");
    let max_size = 100;
    let results = bench_bottom_up(&mut group, algo, max_size);
    log_results(results, algo, max_size).expect("Failed to log results");
    let max_size = 200;
    let results = bench_bottom_up(&mut group, algo, max_size);
    log_results(results, algo, max_size).expect("Failed to log results");
    let max_size = 400;
    let results = bench_bottom_up(&mut group, algo, max_size);
    log_results(results, algo, max_size).expect("Failed to log results");
    group.finish();
}

fn prepare_mapper<HAST: HyperAST + Copy>(
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
    let base_mapper: Mapper<_, CDS<HAST>, CDS<HAST>, VecStore<_>> =
        hyperast.decompress_pair(src, dst).into();
    GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(base_mapper)
}

fn bench_bottom_up<M: Measurement>(
    group: &mut BenchmarkGroup<M>,
    algo: &str,
    max_size: usize,
) -> HashMap<MappingId, MappingData> {
    let file_pairs = data_bench::get_test_data_small();
    let mut results = HashMap::new();

    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();

    let alg = if max_size == 0 {
        algo
    } else {
        &format!("{algo}_{max_size}")
    };

    for (id, src, dst) in &file_pairs {
        let bid = BenchmarkId::new(alg, id);
        let (src, dst) = parse_string_pair(&mut stores, &mut md_cache, src, dst);
        let mapper = prepare_mapper(
            &stores,
            &src.local.compressed_node,
            &dst.local.compressed_node,
        );
        group.throughput(Throughput::Elements(
            (mapper.mappings.capacity().0 + mapper.mappings.capacity().1).div_ceil(2) as u64,
        ));
        group.bench_with_input(bid, &mapper, |b, mapper| {
            let mut first = true;
            b.iter_batched(
                || mapper.clone(),
                |mapper| {
                    let num_mappings_pre = mapper.mappings.len();
                    let mapper_bottom_up = match algo {
                        "gumtree_greedy" if max_size == 50 => {
                            GumtreeGreedy::<_, _, _, _, 50>::match_it(mapper)
                        }
                        "gumtree_greedy" if max_size == 100 => {
                            GumtreeGreedy::<_, _, _, _, 100>::match_it(mapper)
                        }
                        "gumtree_greedy" if max_size == 200 => {
                            GumtreeGreedy::<_, _, _, _, 200>::match_it(mapper)
                        }
                        "gumtree_greedy" if max_size == 400 => {
                            GumtreeGreedy::<_, _, _, _, 400>::match_it(mapper)
                        }
                        "gumtree_greedy" => panic!("unknown max_size"),
                        "gumtree_simple" => GumtreeSimple::<_, _, _, _>::match_it(mapper),
                        _ => panic!("unknown algorithm"),
                    };
                    let num_mappings_post = mapper_bottom_up.mappings.len();
                    let id = MappingId {
                        algorithm: algo.to_string(),
                        file_pair: id.to_string(),
                    };
                    if first && !results.contains_key(&id) {
                        first = false;
                        results.insert(
                            id,
                            MappingData {
                                num_pre_bottom_up: num_mappings_pre,
                                num_post_bottom_up: num_mappings_post,
                            },
                        );
                    }
                    mapper_bottom_up
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    return results;
}

criterion_group!(
    name = bottom_up;
    config = Criterion::default().configure_from_args();
    targets = benchmarks
);
// criterion_group!(
//     name = bottom_up;
//     config = Criterion::default().configure_from_args();
//     targets = benchmark_simple, benchmark_greedy
// );
criterion_main!(bottom_up);
