use std::fmt::Debug;
use std::hint::black_box;
use std::path::{Path, PathBuf};
use num_traits::ToPrimitive;
use hyper_diff::algorithms;
use hyperast::store::SimpleStores;
use hyperast::types;
use hyperast::types::{HyperAST, NodeId};
use hyperast::utils::memusage_linux;
use hyperast_benchmark_diffs::preprocess::parse_string_pair;
use hyperast_vcs_git::preprocessed::PreProcessedRepository;

pub(crate) fn run_diff_commit(repo_name: &str, before: &str, after: &str, algorithm: &str, max_size: usize, sim_threshold: f64) -> usize {
    let mut preprocessed = PreProcessedRepository::new(&repo_name);
    let oid_src = preprocessed.pre_process_single(
        &mut hyperast_vcs_git::git::fetch_github_repository(&preprocessed.name),
        before,
        "",
    );
    let oid_dst = preprocessed.pre_process_single(
        &mut hyperast_vcs_git::git::fetch_github_repository(&preprocessed.name),
        after,
        "",
    );
    log::warn!("diff of {oid_src} and {oid_dst}");

    let stores = &preprocessed.processor.main_stores;

    let commit_src = preprocessed.commits.get_key_value(&oid_src).unwrap();
    let time_src = commit_src.1.processing_time();
    let src_tr = commit_src.1.ast_root;
    use hyperast::types::WithStats;
    let src_s = stores.node_store.resolve(src_tr).size();

    let commit_dst = preprocessed.commits.get_key_value(&oid_dst).unwrap();
    let time_dst = commit_dst.1.processing_time();
    let dst_tr = commit_dst.1.ast_root;
    let dst_s = stores.node_store.resolve(dst_tr).size();

    let hyperast = hyperast_vcs_git::no_space::as_nospaces2(stores);

    run_diff_trees(&hyperast, &src_tr, &dst_tr, algorithm, max_size, sim_threshold)

    // let mu = memusage_linux();
    //
    // let lazy = algorithms::gumtree_hybrid_lazy::diff_hybrid_lazy::<_>(&hyperast, &src_tr, &dst_tr, max_size);
    //
    // let summarized_lazy = &lazy.summarize();
    // use hyper_diff::algorithms::ComputeTime;
    // let total_lazy_t: f64 = summarized_lazy.time();
    // dbg!(summarized_lazy);
    // log::warn!("ed+mappings size: {}", memusage_linux() - mu);
    // log::warn!("done computing diff");
    // println!(
    //     "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{}",
    //     src_s,
    //     dst_s,
    //     Into::<isize>::into(&commit_src.1.memory_used()),
    //     Into::<isize>::into(&commit_dst.1.memory_used()),
    //     time_src,
    //     time_dst,
    //     summarized_lazy.mappings,
    //     total_lazy_t,
    //     summarized_lazy.actions.map_or(-1, |x| x as isize),
    // );
    //
}

pub fn run_diff_file(src_path: &Path, dst_path: &Path, algorithm: &str, max_size: usize, sim_threshold: f64) -> usize {
    let mut stores = SimpleStores::<hyperast_gen_ts_java::types::TStore>::default();
    let mut md_cache = Default::default();

    let src = std::fs::read_to_string(src_path)
        .expect("Failed to read src file");
    let dst = std::fs::read_to_string(dst_path)
        .expect("Failed to read gumtree output file");

    let (src_tr, dst_tr) =
        parse_string_pair(&mut stores, &mut md_cache, black_box(&*src), black_box(&*dst));

    run_diff_trees(&stores, &src_tr.local.compressed_node, &dst_tr.local.compressed_node, algorithm, max_size, sim_threshold)
}

pub fn run_diff_trees<HAST: HyperAST + Copy>(stores: HAST, src_tr: &HAST::IdN, dst_tr: &HAST::IdN, algorithm: &str, max_size: usize, sim_threshold: f64) -> usize
where
    HAST::IdN: Clone + Debug + Eq,
    HAST::IdN: NodeId<IdN = HAST::IdN>,
    HAST::Idx: hyperast::PrimInt,
    HAST::Label: Debug + Clone + Copy + Eq,
    for<'t> <HAST as hyperast::types::AstLending<'t>>::RT: types::WithHashs + types::WithStats,
{
    dbg!(src_tr);
    dbg!(dst_tr);

    let diff_result= match algorithm {
        // "hybrid" => algorithms::gumtree_hybrid::diff_hybrid(
        //     black_box(stores),
        //     black_box(src_tr),
        //     black_box(dst_tr),
        //     max_size
        // ),
        "hybrid" => algorithms::gumtree_hybrid::diff_hybrid(
            black_box(stores),
            black_box(src_tr),
            black_box(dst_tr),
            max_size
        ),
        "simple" => algorithms::gumtree_hybrid::diff_hybrid(
            black_box(stores),
            black_box(src_tr),
            black_box(dst_tr),
            0
        ),
        // "greedy" => algorithms::gumtree::diff(
        //     black_box(stores),
        //     black_box(src_tr),
        //     black_box(dst_tr),
        //     max_size,
        //     DEFAULT_SIM_THRESHOLD
        // ),
        "greedy" => algorithms::gumtree::diff(
            black_box(stores),
            black_box(src_tr),
            black_box(dst_tr),
            max_size,
            sim_threshold,
        ),
        _ => panic!("Unknown function")
    };

    let actions_len = &diff_result.summarize().actions.map_or(-1, |x| x as isize);
    
    dbg!(actions_len);

    black_box(actions_len.to_usize().unwrap())
}

fn find_java_files(dir: &Path, root: &Path) -> Vec<PathBuf> {
    let mut java_files = Vec::new();

    if dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let path = entry.path();
                    if path.is_dir() {
                        java_files.extend(find_java_files(&path, &root));
                    } else if path.extension().and_then(|ext| ext.to_str()) == Some("java") {
                        if let Ok(rel_path) = path.strip_prefix(root) {
                            java_files.push(rel_path.to_path_buf());
                        }
                    }
                }
            }
        }
    }

    java_files
}

pub(crate) fn run_diff_folder(root: &Path, algorithm: &str, max_size: usize, sim_threshold: f64) -> usize {
    let before_dir = root.join("before");
    let after_dir = root.join("after");

    let mut total: usize = 0;
    let paths = find_java_files(&before_dir, &before_dir);
    for path in &paths {
        let before_path = before_dir.join(&path);
        let after_path = after_dir.join(&path);
        total += run_diff_file(&before_path, &after_path, algorithm, max_size, sim_threshold);
    }
    total
}

#[test]
fn run_diff_defects4j_test() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("datasets/defects4j");

    let total = run_diff_folder(&root,"classic", 10, 0.5);
    dbg!(total);
}

// #[test]
// fn test_run_diff_file() {
//     let total = run_diff_file("/var/home/alex/Projects/RP/datasets/defects4j/before/Mockito/9/src_org_mockito_internal_stubbing_answers_CallsRealMethods.java",
//                               "/var/home/alex/Projects/RP/datasets/defects4j/after/Mockito/9/src_org_mockito_internal_stubbing_answers_CallsRealMethods.java",
//     "classic", 10, 0.5);
//     dbg!(total);
// }