use hyperast::store::labels::LabelStore;
use hyperast::store::nodes::DefaultNodeIdentifier;
use hyperast::store::SimpleStores;
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;
use hyperast_vcs_git::no_space::NoSpaceNodeStoreWrapper;
use hyperast_vcs_git::TStore;

pub fn parse_repo<'a>(repositories: &'a mut PreProcessedRepositories,repo_user: &str, repo_name: &str, config: hyperast_vcs_git::processing::RepoConfig, before: &str, after: &str) -> (SimpleStores<TStore, NoSpaceNodeStoreWrapper<'a>, &'a LabelStore>, DefaultNodeIdentifier, DefaultNodeIdentifier) {
    let repo = hyperast_vcs_git::git::Forge::Github.repo(repo_user, repo_name);

    repositories.register_config(repo.clone(), config);

    let repo_configured = repositories
        .get_config((&repo).clone())
        .ok_or_else(|| "missing config for repository".to_string())
        .unwrap()
        .fetch(); // todo: not sure if this is necessary

    let mut rw_src = single_commit(before, &repo_configured.repo).unwrap();
    let oid_src_vec = repositories.pre_process_chunk(&mut rw_src, &repo_configured, usize::MAX);
    let oid_src = oid_src_vec.first().unwrap();

    let mut rw_dst = single_commit(after, &repo_configured.repo).unwrap();
    let oid_dst_vec = repositories.pre_process_chunk(&mut rw_dst, &repo_configured, usize::MAX);
    let oid_dst = oid_dst_vec.first().unwrap();


    let commit_src = repositories.get_commit(&repo_configured.config, &oid_src).unwrap();
    let time_src = commit_src.processing_time();
    let src_tr = commit_src.ast_root;


    let commit_dst = repositories.get_commit(&&repo_configured.config, &oid_dst).unwrap();
    let time_dst = commit_dst.processing_time();
    let dst_tr = commit_dst.ast_root;

    let stores = &repositories.processor.main_stores;

    let hyperast = hyperast_vcs_git::no_space::as_nospaces2(stores);

    (hyperast, src_tr, dst_tr)
}

fn single_commit<'repo>(
    commit: &str,
    repository: &'repo git2::Repository,
) -> Result<impl Iterator<Item = git2::Oid> + 'repo, git2::Error> {
    Ok(hyperast_vcs_git::git::Builder::new(repository)?
        .after(commit)?
        .first_parents()?
        .walk()?
        .take(1)
        .map(|x| x.expect("a valid commit oid")))
}
