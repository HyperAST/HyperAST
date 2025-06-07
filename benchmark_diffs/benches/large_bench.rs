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
use hyper_diff::tree::tree_path::{CompressedTreePath, SimpleTreePath, TreePath};
use hyperast::store::SimpleStores;
use hyperast_gen_ts_java::types::TStore;
use hyperast::{types, PrimInt};
use hyperast::types::{DecompressedFrom, HyperAST, HyperASTShared, NodeId};
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use hyperast_gen_ts_java::legion_with_refs;
use walkdir::{DirEntry, WalkDir};
use std::io::Write;
use std::panic::{catch_unwind, AssertUnwindSafe};
use hyper_diff::actions::action_vec::ActionsVec;
use hyper_diff::actions::Actions;
use hyper_diff::actions::script_generator2::{ScriptGenerator, SimpleAction};
use hyper_diff::decompressed_tree_store::bfs_wrapper::SimpleBfsMapper;
use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
use hyper_diff::matchers::heuristic::gt::simple_bottom_up_matcher3::SimpleBottomUpMatcher3;
use hyper_diff::matchers::heuristic::gt::xy_bottom_up_matcher::XYBottomUpMatcher;
use hyperast::store::defaults::NodeIdentifier;
use hyperast::store::labels::DefaultLabelIdentifier;
use hyperast_gen_ts_java::legion_with_refs::FNode;

const ALGORITHM_NAME: &str = "SimpleBottomUpMatcher3";

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

fn compute_metadata(before_entry: &PathBuf) -> (usize, usize, usize, usize, usize) {
    let after_entry = PathBuf::from(before_entry.to_string_lossy().replace("before", "after"));

    let before_content = fs::read_to_string(before_entry).unwrap();
    let after_content = fs::read_to_string(after_entry).unwrap();

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
            return (0, 0, 0, 0, 0);
        }
    };

    let src = &src_tr.local.compressed_node;
    let dst = &dst_tr.local.compressed_node;

    let mut mapper: Mapper<_, CDS<_>, CDS<_>, VecStore<_>> = (&hyperast)
        .decompress_pair(&src, &dst)
        .into();

    mapper = GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);

    let matches_before = mapper.mappings().len();

    let mut src_arena = Decompressible::<_, CompletePostOrder<_, u32>>::decompress(&hyperast, &src);
    let mut dst_arena = Decompressible::<_, CompletePostOrder<_, u32>>::decompress(&hyperast, &dst);
    let script_mapper = Mapper {
        hyperast: &hyperast,
        mapping: Mapping {
            src_arena,
            dst_arena,
            mappings: mapper.mapping.mappings.clone(),
        }
    };
    let script_mapper = script_mapper.map(
        |x| x,
        // the dst side has to be traversed in bfs for chawathe
        |dst_arena| SimpleBfsMapper::with_store(&hyperast, dst_arena),
    );
    let script_length_before: ActionsVec<SimpleAction<DefaultLabelIdentifier, SimpleTreePath<u16>, NodeIdentifier>> = ScriptGenerator::compute_actions(&hyperast, &script_mapper.mapping).ok().unwrap();
    let script_length_before = script_length_before.len();

    mapper = SimpleBottomUpMatcher3::<_, _, _, _>::match_it(mapper);

    let matches_after = mapper.mappings().len();

    let mut src_arena = Decompressible::<_, CompletePostOrder<_, u32>>::decompress(&hyperast, &src);
    let mut dst_arena = Decompressible::<_, CompletePostOrder<_, u32>>::decompress(&hyperast, &dst);
    let script_mapper = Mapper {
        hyperast: &hyperast,
        mapping: Mapping {
            src_arena,
            dst_arena,
            mappings: mapper.mapping.mappings.clone(),
        }
    };
    let script_mapper = script_mapper.map(
        |x| x,
        // the dst side has to be traversed in bfs for chawathe
        |dst_arena| SimpleBfsMapper::with_store(&hyperast, dst_arena),
    );
    let script_length_after: ActionsVec<SimpleAction<DefaultLabelIdentifier, SimpleTreePath<u16>, NodeIdentifier>> = ScriptGenerator::compute_actions(&hyperast, &script_mapper.mapping).ok().unwrap();
    let script_length_after = script_length_after.len();

    let size_src = mapper.mapping.src_arena.iter().count();
    let size_dst = mapper.mapping.dst_arena.iter().count();
    let size = hyperast.node_store.len();
    let total_size = size_src + size_dst;

    println!("Size: {}, {}, {}", size, size_src, size_dst);
    println!("Matches: {}, {}", matches_before, matches_after);
    println!("Script length: {}, {}", script_length_before, script_length_after);

    (total_size, matches_before, matches_after, script_length_before, script_length_after)
}

fn compile_results() {
    let mut result_file = File::create("results.json").unwrap();
    writeln!(result_file, "[").expect("could not write");

    for before_entry in dataset_files() {

        let before_path = before_entry.to_string_lossy().to_string();

        let path = format!("/home/maciek/HyperAST/target/criterion/large_bench/{ALGORITHM_NAME}/{}/new/estimates.json", file_id(&before_entry));
        let path = Path::new(&path);
        dbg!(path);

        let (total_size, matches_before, matches_after, script_length_before, script_length_after) = compute_metadata(&before_entry);
        if let Ok(result) = fs::read_to_string(&path) {
            writeln!(result_file, "{{
                \"file_name\": \"{before_path}\",
                \"size\": {total_size},
                \"matches_before\": {matches_before},
                \"script_length_before\": {script_length_before},
                \"matches_after\": {matches_after},
                \"script_length_after\": {script_length_after},
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

    for before_entry in dataset_files() {
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

        let mut mapper: Mapper<_, CDS<_>, CDS<_>, VecStore<_>> = (&hyperast)
            .decompress_pair(&src, &dst)
            .into();

        mapper = GreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
        let input_mappings = mapper.mapping.mappings.clone();

        group.bench_function(BenchmarkId::new(ALGORITHM_NAME, file_id(&before_entry)), |b| {
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
                    SimpleBottomUpMatcher3::<_, _, _, _>::match_it(black_box(mapper));
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


