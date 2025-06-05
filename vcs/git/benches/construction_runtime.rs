use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;

fn construction_group(c: &mut Criterion) {
    let mut group = c.benchmark_group("HyperAST Construction");

    struct Input {
        repo: hyperast_vcs_git::git::Repo,
        commit: &'static str,
        config: hyperast_vcs_git::processing::RepoConfig,
        fetch: bool,
    }

    let inputs: &[Input] = &[
        // Input {
        //     repo: hyperast_vcs_git::git::Forge::Github.repo("chromium", "chromium"),
        //     commit: "f461f9752e5918c5c87f2e3767bcb24945ee0fa0",
        //     config: hyperast_vcs_git::processing::RepoConfig::CppMake,
        //     fetch: false,
        // },
        Input {
            repo: hyperast_vcs_git::git::Forge::Github.repo("INRIA", "spoon"),
            commit: "56e12a0c0e0e69ea70863011b4f4ca3305e0542b",
            config: hyperast_vcs_git::processing::RepoConfig::JavaMaven,
            fetch: true,
        },
    ];

    for p in inputs.into_iter() {
        group.bench_with_input(BenchmarkId::new("HyperAST", p.repo.name()), &p, |b, p| {
            b.iter_batched(
                || {
                    let mut repositories = PreProcessedRepositories::default();
                    repositories.register_config(p.repo.clone(), p.config);
                    let repo = repositories
                        .get_config((&p.repo).clone())
                        .ok_or_else(|| "missing config for repository".to_string())
                        .unwrap();
                    let repository = if p.fetch {
                        repo.fetch()
                    } else {
                        repo.nofetch()
                    };
                    (repositories, repository)
                },
                |(mut repositories, repository)| {
                    let mut rw = single_commit(p.commit, &repository.repo).unwrap();
                    repositories.pre_process_chunk(&mut rw, &repository, usize::MAX)
                },
                BatchSize::PerIteration,
            )
        });
    }
    group.finish()
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

criterion_group!(
    name = construction;
    config = Criterion::default().sample_size(10).measurement_time(std::time::Duration::from_secs(10)).configure_from_args();
    targets = construction_group
);
criterion_main!(construction);
