use std::fmt::Debug;
use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hyper_diff::decompressed_tree_store::{CompletePostOrder, ContiguousDescendants, DecompressedTreeStore, DecompressedWithParent};
use hyper_diff::matchers::heuristic::gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
use hyper_diff::tree::tree_path::{CompressedTreePath, SimpleTreePath, TreePath};
use hyperast::store::SimpleStores;
use hyperast::types::{DecompressedFrom, HyperAST, HyperASTShared, NodeId};
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use walkdir::{DirEntry, WalkDir};
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};
use hyper_diff::actions::Actions;
use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
use hyper_diff::matchers::{Decompressible, Mapper, Mapping};
use hyper_diff::matchers::mapping_store::{DefaultMultiMappingStore, MappingStore, MonoMappingStore, VecStore};
use hyper_diff::matchers::heuristic::gt::greedy_subtree_matcher::GreedySubtreeMatcher;

type M = VecStore<u32>;
type MM = DefaultMultiMappingStore<u32>;

const ALGORITHM_NAME: &str = "HyperDiff1000";

fn dataset_files() -> Vec<PathBuf> {
    WalkDir::new("/home/maciek/HyperAST/gt_datasets/datasets/").into_iter()
        .filter_entry(|e| {
            !(e.file_name() == "after")
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn file_id(path: &PathBuf) -> String {
    let filename: String = path.file_name().unwrap().to_str().unwrap().to_string();
    let filename = filename[filename.len().saturating_sub(40)..].to_string();
    let parent_path = path.parent().unwrap().to_string_lossy().to_string();

    let file_id = format!("{parent_path}/{filename}");
    let file_id = file_id.replace("/", "-");
    file_id[file_id.len().saturating_sub(60)..].to_string()
}

fn compute_metadata(before_entry: &PathBuf) -> usize {
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

    let mut _src_arena = Decompressible::<_, LazyPostOrder<_, u32>>::decompress(&hyperast, src);
    let mut _dst_arena = Decompressible::<_, LazyPostOrder<_, u32>>::decompress(&hyperast, dst);

    let src_arena = _src_arena.as_mut();
    let dst_arena = _dst_arena.as_mut();

    let size_src = src_arena.iter().count();
    let size_dst = dst_arena.iter().count();
    let size = hyperast.node_store.len();
    let total_size = size_src + size_dst;

    println!("Size: {}, {}, {}", size, size_src, size_dst);

    total_size
}

fn compile_results() {
    let mut result_file = File::create("results.json").unwrap();
    writeln!(result_file, "[").expect("could not write");

    for before_entry in dataset_files() {

        let before_path = before_entry.to_string_lossy().to_string();

        let path = format!("/home/maciek/HyperAST/target/criterion/large_bench/{ALGORITHM_NAME}/{}/new/estimates.json", file_id(&before_entry));
        let path = Path::new(&path);
        dbg!(path);

        if let Ok(result) = fs::read_to_string(&path) {
            let metadata_result = catch_unwind(AssertUnwindSafe(|| {
                compute_metadata(&before_entry)
            }));
            let total_size = match metadata_result {
                Ok(result) => result,
                Err(_) => {
                    eprintln!("Could not compile {}", before_entry.to_string_lossy().to_string());
                    continue;
                }
            };

            writeln!(result_file, "{{
                \"file_name\": \"{before_path}\",
                \"size\": {total_size},
                \"matches_before\": -1,
                \"script_length_before\": -1,
                \"matches_after\": -1,
                \"script_length_after\": -1,
                \"criterion\": {result}
            }},"
            ).expect("could not write");
        } else {
            println!("Could not read file {}", path.display());
        }
    }

    writeln!(result_file, "]").expect("could not write");
}

fn diff_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_bench");

    dbg!(dataset_files());
    
    let dataset_size = dataset_files().len();
    let mut progress = 0;
    
    for before_entry in dataset_files() {
        progress += 1;
        println!("Processing {}/{}", progress, dataset_size);
    
        let path = format!(
            "/home/maciek/HyperAST/target/criterion/large_bench/{ALGORITHM_NAME}/{}/new/estimates.json",
            file_id(&before_entry)
        );
        if Path::new(&path).exists() {
            println!("Results exists, skipping: {}", path);
            continue;
        }
    
        let after_entry = PathBuf::from(before_entry.to_string_lossy().replace("before", "after"));
    
        let before_content = fs::read_to_string(&before_entry).unwrap();
        let after_content = fs::read_to_string(&after_entry).unwrap();
    
        let mut hyperast = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
        let mut md_cache = Default::default();
    
        let result = catch_unwind(AssertUnwindSafe(|| {
            parse_string_pair(
                &mut hyperast,
                &mut md_cache,
                black_box(&before_content),
                black_box(&after_content),
            )
        }));
        let (src_tr, dst_tr) = match result {
            Ok(pair) => pair,
            Err(_) => {
                eprintln!("Could not parse {}", before_entry.to_string_lossy().to_string());
                continue;
            }
        };

        let src = &src_tr.local.compressed_node;
        let dst = &dst_tr.local.compressed_node;

        group.bench_function(BenchmarkId::new(ALGORITHM_NAME, file_id(&before_entry)), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let mut _src_arena = Decompressible::<_, LazyPostOrder<_, u32>>::decompress(&hyperast, &src);
                    let mut _dst_arena = Decompressible::<_, LazyPostOrder<_, u32>>::decompress(&hyperast, &dst);

                    let src_arena = _src_arena.as_mut();
                    let dst_arena = _dst_arena.as_mut();
                    let mapper: Mapper<
                        _, Decompressible<_, &mut LazyPostOrder<<&SimpleStores<hyperast_gen_ts_java::types::TStore> as HyperASTShared>::IdN, u32>>,
                        Decompressible<_, &mut LazyPostOrder<<&SimpleStores<hyperast_gen_ts_java::types::TStore> as HyperASTShared>::IdN, u32>>, M
                    > = Mapper {
                        hyperast: &hyperast,
                        mapping: Mapping {
                            src_arena,
                            dst_arena,
                            mappings: Default::default(),
                        }
                    };
                    

                    let start = Instant::now();
                    let mapper = LazyGreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
                    total += start.elapsed();
                }

                total
            })
        });

        group.bench_function(BenchmarkId::new("GumtreeDiff1000", file_id(&before_entry)), |b| {
            b.iter_custom(|iters| {
                let mut total = Duration::ZERO;
                for _ in 0..iters {
                    let mapper: Mapper<_, CDS<_>, CDS<_>, VecStore<_>> = (&hyperast)
                        .decompress_pair(&src, &dst)
                        .into();

                    let start = Instant::now();
                    let mapper = GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
                    total += start.elapsed();
                }

                total
            })
        });
    }

    group.finish();

    compile_results();
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
