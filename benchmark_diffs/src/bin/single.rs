use std::time::Duration;

use hyperast::utils::memusage_linux;
use hyperast_benchmark_diffs::{other_tools, postprocess::CompressedBfPostProcess};
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;
use num_traits::ToPrimitive;

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

    let repo = hyperast_vcs_git::git::Forge::Github.repo(repo_user, repo_name);

    repositories.register_config(repo.clone(), config);

    let repo_configured = repositories
        .get_config((&repo).clone())
        .ok_or_else(|| "missing config for repository".to_string())
        .unwrap()
        .fetch(); // todo: not sure if this is necessary

    let oid_src = repositories
        .pre_process_with_limit(&repo_configured, "", before, 1)
        .unwrap()[0];
    let oid_dst = repositories
        .pre_process_with_limit(&repo_configured, "", after, 1)
        .unwrap()[0];

    log::warn!("diff of {oid_src} and {oid_dst}");

    let stores = &repositories.processor.main_stores;

    let commit_src = repositories
        .get_commit(&repo_configured.config, &oid_src)
        .unwrap();
    let time_src = commit_src.processing_time();
    let src_tr = commit_src.ast_root;
    use hyperast::types::WithStats;
    let src_s = stores.node_store.resolve(src_tr).size();

    let commit_dst = repositories
        .get_commit(&repo_configured.config, &oid_dst)
        .unwrap();
    let time_dst = commit_dst.processing_time();
    let dst_tr = commit_dst.ast_root;
    let dst_s = stores.node_store.resolve(dst_tr).size();

    let hyperast = hyperast_vcs_git::no_space::as_nospaces2(stores);

    let mu = memusage_linux();
    let lazy =
        hyper_diff::algorithms::gumtree_hybrid_lazy::diff_with_hyperparameters::<_, 1, 50, 1, 2>(
            &hyperast, &src_tr, &dst_tr,
        );
    let summarized_lazy = &lazy.summarize();
    let total_lazy_t: Duration = summarized_lazy.exec_data.sum().unwrap();
    dbg!(summarized_lazy);
    log::warn!("ed+mappings size: {}", memusage_linux() - mu);
    log::warn!("done computing diff");
    println!(
        "{oid_src}/{oid_dst},{},{},{},{},{},{},{},{},{}",
        src_s,
        dst_s,
        Into::<isize>::into(&commit_src.memory_used()),
        Into::<isize>::into(&commit_dst.memory_used()),
        time_src,
        time_dst,
        summarized_lazy.mappings,
        total_lazy_t.as_secs_f64(),
        summarized_lazy.actions.map_or(-1, |x| x as isize),
    );
    let diff_algorithm = "Chawathe";
    // let gt_out_format = "COMPRESSED"; // JSON
    let gt_out_format = "JSON";
    // JSON
    let timeout = total_lazy_t.mul_f64(10.).as_secs_f64();
    let gt_out = other_tools::gumtree::subprocess(
        &hyperast,
        src_tr,
        dst_tr,
        "gumtree",
        diff_algorithm,
        timeout.ceil().to_u64().unwrap(),
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
