use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hyper_diff::decompressed_tree_store::{CompletePostOrder, ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent};
use hyper_diff::matchers::heuristic::gt::greedy_subtree_matcher::GreedySubtreeMatcher;
use hyper_diff::matchers::{Decompressible, Mapper, Mapping};
use hyper_diff::matchers::heuristic::gt::greedy_bottom_up_matcher::GreedyBottomUpMatcher;
use hyper_diff::matchers::mapping_store::{DefaultMultiMappingStore, MappingStore, MonoMappingStore, VecStore};
use hyper_diff::tree::tree_path::CompressedTreePath;
use hyperast::store::SimpleStores;
use hyperast::test_utils::simple_tree::TStore;
use hyperast::types;
use hyperast::types::{DecompressedFrom, HyperAST, HyperASTShared, NodeId};
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use hyperast_gen_ts_java::legion_with_refs;
use walkdir::WalkDir;
use std::io::Write;
use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
use hyper_diff::matchers::heuristic::gt::simple_bottom_up_matcher::SimpleBottomUpMatcher;
use hyper_diff::matchers::heuristic::gt::xy_bottom_up_matcher::XYBottomUpMatcher;
use hyperast::store::defaults::NodeIdentifier;

const DATASET: &str = "gh-java/";
const INPUT_PATH: &str = "elastic-search/1d732dfc1b08deca0f20b467fe4c66f041bb37b5/";

fn setup_mapper(filename: &str)  {

}
fn diff_benchmark(c: &mut Criterion) {
    let before_folder = format!("gt_datasets/{}before/", DATASET);
    let before_folder: &Path = Path::new(&before_folder);
    let mut result_file = File::create("results.json").unwrap();
    writeln!(result_file, "[").expect("could not write");
    fs::remove_dir_all("/home/maciek/HyperAST/target/criterion/large_bench").expect("could not delete old results");

    let mut group = c.benchmark_group("large_bench");

    for before_entry in WalkDir::new(Path::new(before_folder)).into_iter().filter_map(|e| e.ok()).filter(|e| e.file_type().is_file()) {
        let before_entry = before_entry.path();
        let before_path = before_entry.to_string_lossy().to_string();
        let file_id = before_entry.to_string_lossy().to_string().replace("/", "-");
        let file_id = &file_id[file_id.len().saturating_sub(50)..];
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

        let src = &src_tr.local.compressed_node;
        let dst = &dst_tr.local.compressed_node;

        let mut mapper: Mapper<_, CDS<_>, CDS<_>, VecStore<_>> = (&hyperast)
            .decompress_pair(&src, &dst)
            .into();

        mapper = GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
        let input_mappings = mapper.mapping.mappings.clone();

        let matches_before = mapper.mappings().len();

        mapper = XYBottomUpMatcher::<_, _, _, _>::match_it(mapper);

        let matches_after = mapper.mappings().len();

        let size_src = mapper.mapping.src_arena.iter().count();
        let size_dst = mapper.mapping.dst_arena.iter().count();
        let size = hyperast.node_store.len();
        let total_size = size_src + size_dst;

        println!("Size: {}, {}, {}", size, size_src, size_dst);
        println!("Matches: {}, {}", matches_before, matches_after);

        group.bench_function(BenchmarkId::from_parameter(&file_id), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let mut src_arena = Decompressible::<_, CompletePostOrder<_, u32>>::decompress(&hyperast, &src);
                    let mut dst_arena = Decompressible::<_, CompletePostOrder<_, u32>>::decompress(&hyperast, &dst);
                    let mapper = Mapper {
                        hyperast: &hyperast,
                        mapping: Mapping {
                            src_arena,
                            dst_arena,
                            mappings: input_mappings.clone(),
                        }
                    };

                    let start = Instant::now();
                    black_box(XYBottomUpMatcher::<_, _, _, _>::match_it(black_box(mapper)));
                    total += start.elapsed();
                }

                total
            })
        });

        let path = format!("/home/maciek/HyperAST/target/criterion/large_bench/{}/new/estimates.json", &file_id);
        let path = Path::new(&path);
        dbg!(path);
        while !path.exists() {
            sleep(Duration::from_millis(100));
        }
        let result = fs::read_to_string(&path).unwrap();

        writeln!(result_file, "{{
                \"file_name\": \"{before_path}\",
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

criterion_group!{
    name = benches;
    config = Criterion::default()
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2))
        .sample_size(10);
    targets = diff_benchmark
}
criterion_main!(benches);

#[allow(type_alias_bounds)]
type CDS<HAST: HyperASTShared> = Decompressible<HAST, CompletePostOrder<HAST::IdN, u32>>;
