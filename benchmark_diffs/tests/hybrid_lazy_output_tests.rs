use hyperast_benchmark_diffs::preprocess_repo::parse_repo;
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;

#[test]
fn test_simple() {
    compare_lazy_hybrid_vs_lazy(
        "apache", "maven",
        hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        "a02834611bad3442ad073b10f1dee2322916f1f3",
        "c3cf29438e3d65d6ee5c5726f8611af99d9a649a",
        100
    )
}

#[test]
fn test_histogram_bug() {
    compare_lazy_hybrid_vs_lazy(
        "google",
        "gson",
        hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        "810e3560590bb807ed7113ccfff716aac21a3f33",
        "00ae39775708147e115512be5d4f92bee02e9b89",
        100
    )
}

#[test]
fn test_maxsize_0() {
    compare_lazy_hybrid_vs_lazy(
        "apache", "maven",
        hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        "a02834611bad3442ad073b10f1dee2322916f1f3",
        "c3cf29438e3d65d6ee5c5726f8611af99d9a649a",
        0
    )
}

#[test]
fn test_maxsize_1000() {
    compare_lazy_hybrid_vs_lazy(
        "apache", "maven",
        hyperast_vcs_git::processing::RepoConfig::JavaMaven,
        "a02834611bad3442ad073b10f1dee2322916f1f3",
        "c3cf29438e3d65d6ee5c5726f8611af99d9a649a",
        1000
    )
}

fn compare_lazy_hybrid_vs_lazy(repo_user: &str, repo_name: &str, config: hyperast_vcs_git::processing::RepoConfig, before: &str, after: &str, max_size: usize) {
    let mut repositories = PreProcessedRepositories::default();
    let (hyperast, src_tr, dst_tr) = parse_repo(
        &mut repositories,
        repo_user,
        repo_name,
        config,
        before,
        after
    );

    let greedy = hyper_diff::algorithms::gumtree_hybrid::diff_hybrid(&hyperast, &src_tr, &dst_tr, max_size);
    let summarized_greedy = &greedy.summarize();
    dbg!(summarized_greedy);

    let lazy = hyper_diff::algorithms::gumtree_hybrid_lazy::diff_hybrid_lazy(&hyperast, &src_tr, &dst_tr, max_size);
    let summarized_lazy = &lazy.summarize();
    dbg!(summarized_lazy);

    dbg!(&greedy.actions);
    dbg!(&lazy.actions);

    assert_eq!(summarized_greedy.actions.map_or(-1, |x| x as isize), summarized_lazy.actions.map_or(-1, |x| x as isize));
    assert_eq!(summarized_greedy.mappings, summarized_lazy.mappings);
}