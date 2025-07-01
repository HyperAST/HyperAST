use hyperast::utils::memusage_linux;
use num_traits::ToPrimitive;

use hyperast_benchmark_diffs::{other_tools, postprocess::CompressedBfPostProcess};

use hyperast_benchmark_diffs::preprocess_repo::parse_repo;
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;
#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

fn main() {
    use std::env;
    let args: Vec<String> = env::args().collect();
    log::warn!("args: {:?}", args);
    let mut repo = args
        .get(1)
        .expect("give an argument like openjdk/jdk or INRIA/spoon") //"openjdk/jdk";//"INRIA/spoon";
        .split('/');
    let repo_user = repo.next().unwrap();
    let repo_name = repo.next().unwrap();
    let before = args.get(2).map_or("", |x| x);
    let after = args.get(3).map_or("", |x| x);
    let config = hyperast_vcs_git::processing::RepoConfig::JavaMaven;
    single(repo_user, repo_name, config, before, after);
}

#[test]
fn aaa() {
    single(
        "apache",
        "maven",
        hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        "a02834611bad3442ad073b10f1dee2322916f1f3",
        "c3cf29438e3d65d6ee5c5726f8611af99d9a649a",
    )
}

#[test]
fn bbb() {
    single(
        "apache",
        "maven",
        hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        "14449e426aee2763d6435b63ef632b7c0b9ed767",
        "6fba7aa3c4d31d088df3ef682f7307b7c9a2f17c",
    )
}

#[test]
fn test_histogram_bug() {
    single(
        "google",
        "gson",
        hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        "810e3560590bb807ed7113ccfff716aac21a3f33",
        "00ae39775708147e115512be5d4f92bee02e9b89",
    )
}

fn single(
    repo_user: &str,
    repo_name: &str,
    config: hyperast_vcs_git::processing::RepoConfig,
    before: &str,
    after: &str,
) {
    let mut repositories = PreProcessedRepositories::default();
    let (hyperast, src_tr, dst_tr) = parse_repo(
        &mut repositories,
        repo_user,
        repo_name,
        config,
        before,
        after,
    );

    let mu = memusage_linux();
    let lazy = hyper_diff::algorithms::gumtree_hybrid_lazy::diff_hybrid_lazy(
        &hyperast, &src_tr, &dst_tr, 50,
    );
    let summarized_lazy = &lazy.summarize();
    use hyper_diff::algorithms::ComputeTime;
    // let total_lazy_t: std::time::Duration = summarized_lazy.time();
    // dbg!(summarized_lazy);
    log::warn!("ed+mappings size: {}", memusage_linux() - mu);
    log::warn!("done computing diff");
    // println!(
    //     "{oid_src}/{oid_dst},{},{},{},{},{},{},{}",
    //     src_s,
    //     dst_s,
    //     Into::<isize>::into(&commit_src.memory_used()),
    //     Into::<isize>::into(&commit_dst.memory_used()),
    //     summarized_lazy.mappings,
    //     total_lazy_t,
    //     summarized_lazy.actions.map_or(-1, |x| x as isize),
    // );
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
        0, //(total_lazy_t * 10).as_secs_f64().ceil().to_u64().unwrap(),
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
