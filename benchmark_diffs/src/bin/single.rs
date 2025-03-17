use hyperast::utils::memusage_linux;
use hyperast_vcs_git::preprocessed::PreProcessedRepository;
use num_traits::ToPrimitive;

use hyperast_benchmark_diffs::{other_tools, postprocess::CompressedBfPostProcess};

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    use std::env;
    let args: Vec<String> = env::args().collect();
    log::warn!("args: {:?}", args);
    let repo_name = args
        .get(1)
        .expect("give an argument like openjdk/jdk or INRIA/spoon"); //"openjdk/jdk";//"INRIA/spoon";
    let before = args.get(2).map_or("", |x| x);
    let after = args.get(3).map_or("", |x| x);
    single(repo_name, before, after);
}

#[test]
fn aaa() {
    single(
        "apache/maven",
        "a02834611bad3442ad073b10f1dee2322916f1f3",
        "c3cf29438e3d65d6ee5c5726f8611af99d9a649a",
    )
}

#[test]
fn bbb() {
    single(
        "apache/maven",
        "14449e426aee2763d6435b63ef632b7c0b9ed767",
        "6fba7aa3c4d31d088df3ef682f7307b7c9a2f17c",
    )
}

fn single(repo_name: &str, before: &str, after: &str) {
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

    let mu = memusage_linux();
    let lazy = hyper_diff::algorithms::gumtree_lazy::diff(&hyperast, &src_tr, &dst_tr);
    let summarized_lazy = &lazy.summarize();
    use hyper_diff::algorithms::ComputeTime;
    let total_lazy_t: f64 = summarized_lazy.time();
    dbg!(summarized_lazy);
    log::warn!("ed+mappings size: {}", memusage_linux() - mu);
    log::warn!("done computing diff");
    println!(
        "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{}",
        src_s,
        dst_s,
        Into::<isize>::into(&commit_src.1.memory_used()),
        Into::<isize>::into(&commit_dst.1.memory_used()),
        time_src,
        time_dst,
        summarized_lazy.mappings,
        total_lazy_t,
        summarized_lazy.actions.map_or(-1, |x| x as isize),
    );
    let diff_algorithm = "Chawathe";
    // let gt_out_format = "COMPRESSED"; // JSON
    let gt_out_format = "JSON";
    // JSON
    let gt_out = other_tools::gumtree::subprocess(
        &hyperast,
        src_tr,
        dst_tr,
        "gumtree",
        diff_algorithm,
        (total_lazy_t * 10.).ceil().to_u64().unwrap(),
        gt_out_format,
    );
    if gt_out_format == "COMPRESSED" {
        if let Some(gt_out) = &gt_out {
            let pp = CompressedBfPostProcess::create(gt_out);
            let (pp, counts) = pp.counts();
            let (pp, gt_timings) = pp.performances();
            let valid = pp.validity_mappings(&lazy.mapper);
            dbg!(counts);
            dbg!(gt_timings);
            dbg!(valid.additional_mappings);
            dbg!(valid.missing_mappings);
        }
    } else if gt_out_format == "JSON" {
        if let Some(gt_out) = &gt_out {
            let pp = hyperast_benchmark_diffs::postprocess::SimpleJsonPostProcess::new(&gt_out);
            let gt_timings = pp.performances();
            let counts = pp.counts();
            let valid = pp.validity_mappings(&lazy.mapper);
            dbg!(counts);
            dbg!(gt_timings);
            dbg!(valid.additional_mappings.len());
            dbg!(valid.missing_mappings.len());
            // Some((gt_timings, counts, valid.map(|x| x.len())))
        }
    } else {
        unimplemented!("gt_out_format {} is not implemented", gt_out_format)
    };
}
