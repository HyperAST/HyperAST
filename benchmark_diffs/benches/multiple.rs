use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hyper_diff::decompressed_tree_store::CompletePostOrder;
use hyper_diff::matchers::heuristic::gt::greedy_subtree_matcher::GreedySubtreeMatcher;
use hyper_diff::matchers::{Decompressible, Mapper, Mapping};
use hyper_diff::matchers::heuristic::gt::greedy_bottom_up_matcher::GreedyBottomUpMatcher;
use hyper_diff::matchers::mapping_store::{DefaultMultiMappingStore, MappingStore, VecStore};
use hyper_diff::tree::tree_path::CompressedTreePath;
use hyperast::store::SimpleStores;
use hyperast::test_utils::simple_tree::TStore;
use hyperast::types;
use hyperast::types::{HyperAST, HyperASTShared, NodeId};
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use hyperast_gen_ts_java::legion_with_refs;
use walkdir::WalkDir;
use std::io::Write;
use hyper_diff::matchers::heuristic::gt::xy_bottom_up_matcher::XYBottomUpMatcher;

const DATASET: &str = "gh-java/";
const INPUT_PATH: &str = "elastic-search/1d732dfc1b08deca0f20b467fe4c66f041bb37b5/";

fn setup_mapper(filename: &str)  {

}
fn diff_benchmark(c: &mut Criterion) {
    let before_folder = format!("gt_datasets/{}before/", DATASET);
    let before_folder: &Path = Path::new(&before_folder);
    let mut result_file = File::create("results_xy.json").unwrap();
    writeln!(result_file, "[").expect("could not write");

    let mut group = c.benchmark_group("large_bench");

    for before_entry in WalkDir::new(Path::new(before_folder)).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let before_entry = before_entry.path();
        let file_name = before_entry.file_name().unwrap().to_string_lossy().to_string();
        let after_entry = PathBuf::from(before_entry.to_string_lossy().replace("before", "after"));

        let before_content = fs::read_to_string(before_entry).unwrap();
        let after_content = fs::read_to_string(after_entry).unwrap();

        let mut hyperast = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
        let mut md_cache = Default::default();

        let (src_tr, dst_tr) = parse_string_pair(
            &mut hyperast,
            &mut md_cache,
            black_box(&before_content),
            black_box(&after_content),
        );

        let mapper: Mapper<_, CDS<_>, CDS<_>, VecStore<_>> = (&hyperast)
            .decompress_pair(&src_tr.local.compressed_node, &dst_tr.local.compressed_node)
            .into();

        let mapper = GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);

        let matches_before = mapper.mappings().len();

        let mapper = GreedyBottomUpMatcher::<_, _, _, _>::match_it(mapper);

        let matches_after = mapper.mappings().len();

        let size_src = mapper.mapping.src_arena.iter().count();
        let size_dst = mapper.mapping.dst_arena.iter().count();
        let size = hyperast.node_store.len();
        let total_size = size_src + size_dst;
        let input_size = total_size - matches_before * 2;

        println!("Size: {}, {}, {}", size, size_src, size_dst);
        println!("Matches: {}, {}", matches_before, matches_after);

        group.throughput(Throughput::Elements(input_size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(&file_name), &input_size, |b, &input_size| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;

                for _ in 0..iters {
                    let mut hyperast = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
                    let mut md_cache = Default::default();

                    let (src_tr, dst_tr) = parse_string_pair(
                        &mut hyperast,
                        &mut md_cache,
                        black_box(&before_content),
                        black_box(&after_content),
                    );

                    let mapper: Mapper<_, CDS<_>, CDS<_>, VecStore<_>> = (&hyperast)
                        .decompress_pair(&src_tr.local.compressed_node, &dst_tr.local.compressed_node)
                        .into();

                    let mapper = GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
                    let start = Instant::now();
                    let mapper = XYBottomUpMatcher::<_, _, _, _>::match_it(mapper);
                    total += start.elapsed();
                }

                total
            })
        });

        let path = format!("/home/maciek/HyperAST/target2/criterion/large_bench/{}/new/estimates.json", &file_name);
        let path = Path::new(&path);
        dbg!(path);
        while !path.exists() {
            sleep(Duration::from_millis(100));
        }
        let result = fs::read_to_string(&path).unwrap();

        writeln!(result_file, "{{
                \"file_name\": \"{file_name}\",
                \"size\": {total_size},
                \"matches_before\": {matches_before},
                \"matches_after\": {matches_after},
                \"criterion\": {result}
            }},"
        ).expect("could not write");
    }

    writeln!(result_file, "]").expect("could not write");

    group.finish();
}

criterion_group!(benches, diff_benchmark);
criterion_main!(benches);

#[allow(type_alias_bounds)]
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;