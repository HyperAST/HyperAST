mod data_bench;

use std::{
    error::Error,
    fmt::Debug,
    fs::{self, File},
    path::Path,
    usize,
};

use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};
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

#[allow(type_alias_bounds)]
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;

#[derive(Serialize)]
struct MappingInfo {
    algorithm: String,
    file_pair: String,
    num_pre_bottom_up: usize,
    num_post_bottom_up: usize,
}

fn log_results<'a>(data: Vec<MappingInfo>, algo: &str) -> Result<(), Box<dyn Error>> {
    let dir_path = Path::new("bench_results");
    if !dir_path.exists() {
        fs::create_dir_all(dir_path)?;
    }

    let file_path = dir_path.join(format!("{algo}.json"));
    let file = File::create(file_path)?;
    serde_json::to_writer_pretty(file, &data)?;
    Ok(())
}

fn benchmark_simple(c: &mut Criterion) {
    let algo = "gumtree_simple";
    let results = bench_bottom_up(c, algo);
    log_results(results, algo).expect("Failed to log results");
}

fn benchmark_greedy(c: &mut Criterion) {
    let algo = "gumtree_greedy";
    let results = bench_bottom_up(c, algo);
    log_results(results, algo).expect("Failed to log results");
}

fn foo<HAST: HyperAST + Copy>(
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

fn bench_bottom_up(c: &mut Criterion, algo: &str) -> Vec<MappingInfo> {
    let mut group = c.benchmark_group("bottom_up_bench");
    let file_pairs = data_bench::get_test_data_small();
    let mut results = Vec::with_capacity(file_pairs.len());

    for (id, src, dst) in &file_pairs {
        {
            let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
            let mut md_cache = Default::default();
            let (src, dst) =
                parse_string_pair(&mut stores, &mut md_cache, black_box(src), black_box(dst));
            let mapper = foo(
                &stores,
                &src.local.compressed_node,
                &dst.local.compressed_node,
            );
            let num_mappings_pre = mapper.mappings.len();
            let mapper_bottom_up = match algo {
                "gumtree_greedy" => GumtreeGreedy::<_, _, _, _>::match_it(mapper.clone()),
                "gumtree_simple" => GumtreeSimple::<_, _, _, _>::match_it(mapper.clone()),
                _ => panic!("unknown algorithm"),
            };
            let num_mappings_post = mapper_bottom_up.mappings.len();
            results.push(MappingInfo {
                algorithm: algo.to_string(),
                file_pair: id.to_string(),
                num_pre_bottom_up: num_mappings_pre,
                num_post_bottom_up: num_mappings_post,
            });
        }

        // Initialize stores for each iteration
        let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
        let mut md_cache = Default::default();

        // Parse the two Java files
        let (src, dst) =
            parse_string_pair(&mut stores, &mut md_cache, black_box(src), black_box(dst));

        let mapper = foo(
            &stores,
            &src.local.compressed_node,
            &dst.local.compressed_node,
        );
        group.bench_with_input(BenchmarkId::new(algo, id), &mapper, |b, mapper| {
            b.iter_batched(
                || mapper.clone(),
                |mapper| {
                    let mapper_bottom_up = match algo {
                        "gumtree_greedy" => GumtreeGreedy::<_, _, _, _>::match_it(mapper),
                        "gumtree_simple" => GumtreeSimple::<_, _, _, _>::match_it(mapper),
                        _ => panic!("unknown algorithm"),
                    };
                    black_box(mapper_bottom_up);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
    return results;
}

criterion_group!(simple_diff, benchmark_simple);
criterion_group!(greedy_diff, benchmark_greedy);
criterion_main!(simple_diff, greedy_diff);
