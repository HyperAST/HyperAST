use std::io::stdin;

use clap::Parser;
use tracing_subscriber::EnvFilter;

#[derive(Parser)]
/// Measure metrics in source code histories
///
/// supported languages:
/// - Java
/// - Cpp (soon)
///
/// set the env variable RUST_LOG=debug to display logs during computation
struct Cli {
    /// The owner of the repository, eg. INRIA
    owner: String,
    /// The name of the repository, eg. spoon
    name: String,
    /// The start commit, eg. 56e12a0c0e0e69ea70863011b4f4ca3305e0542b
    commit: String,
    /// Number of commits to process
    #[clap(short, long, default_value_t = 10)]
    depth: usize,
    /// File containing a script to execute. Look at the examples first
    #[clap(short, long)]
    file: Option<std::path::PathBuf>,
    /// Examples scripts to compute:
    ///
    /// * size: the number of nodes in the syntax tree,
    ///
    /// * mcc: the McCabe cyclomatic complexity, or
    ///
    /// * Loc: the number of lines of code ignoring blank lines and comments
    #[clap(short, long)]
    example: Option<String>,
    /// Write the script directly in stdin
    #[clap(short, long)]
    interative: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(tracing::level_filters::LevelFilter::OFF.into())
                .from_env_lossy(),
        )
        .try_init()
        .unwrap();

    // let repo_spec = hyperast_vcs_git::git::Forge::Github.repo("graphhopper", "graphhopper");
    // let commit = "f5f2b7765e6b392c5e8c7855986153af82cc1abe";
    // let script = hyperast::scripting::lua_scripting::PREPRO_LOC.into();
    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo(&args.owner, &args.name);
    let config = hyperast_vcs_git::processing::RepoConfig::JavaMaven;
    if let Some(file) = args.file {
        let script = std::fs::read_to_string(file).unwrap();
        scripting(repo_spec, config, &args.commit, &script, args.depth)
    } else if let Some(example) = args.example {
        let script = match example.as_str() {
            "size" => hyperast::scripting::lua_scripting::PREPRO_SIZE_WITH_FINISH,
            "mcc" => hyperast::scripting::lua_scripting::PREPRO_MCC_WITH_FINISH,
            "LoC" => hyperast::scripting::lua_scripting::PREPRO_LOC,
            "none" => {
                let state = backend::AppState::default();
                state
                    .repositories
                    .write()
                    .unwrap()
                    .register_config(repo_spec.clone(), config);
                let repo = state
                    .repositories
                    .read()
                    .unwrap()
                    .get_config(repo_spec)
                    .ok_or_else(|| "missing config for repository".to_string())?;
                let repository = repo.fetch();
                log::debug!("done cloning {}", repository.spec);
                return _scripting(state, &args.commit, args.depth, repository);
            }
            x => {
                eprintln!("{x} is not an available example. Try: size, mcc, LoC");
                std::process::exit(1)
            }
        };
        scripting(repo_spec, config, &args.commit, &script, args.depth)
    } else if args.interative {
        let mut script = String::new();
        for l in stdin().lines() {
            let l = l.unwrap();
            if l.is_empty() {
                break;
            }
            script += &l;
        }
        scripting(repo_spec, config, &args.commit, &script, args.depth)
    } else {
        eprintln!(
            "You need to select an example, give a file, or write a script. Use -h to show help."
        );
        std::process::exit(1)
    }
}

fn scripting(
    repo_spec: hyperast_vcs_git::git::Repo,
    config: hyperast_vcs_git::processing::RepoConfig,
    commit: &str,
    script: &str,
    depth: usize,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let state = backend::AppState::default();
    state
        .repositories
        .write()
        .unwrap()
        .register_config_with_prepro(repo_spec.clone(), config, script.into());
    let repo = state
        .repositories
        .read()
        .unwrap()
        .get_config(repo_spec)
        .ok_or_else(|| "missing config for repository".to_string())?;
    let repository = repo.fetch();
    log::debug!("done cloning {}", repository.spec);
    _scripting(state, commit, depth, repository)
}

fn _scripting(
    state: backend::AppState,
    commit: &str,
    depth: usize,
    repository: hyperast_vcs_git::processing::ConfiguredRepo2,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut rw = hyperast_vcs_git::git::Builder::new(&repository.repo)
        .unwrap()
        .first_parents()
        .unwrap()
        .after(commit)?
        .walk()?
        .take(depth)
        .map(|x| x.unwrap());

    let mut commits =
        state
            .repositories
            .write()
            .unwrap()
            .pre_process_chunk(&mut rw, &repository, 1);
    for _ in 0..200 {
        if commits.is_empty() {
            break;
        }
        dbg!(commits.len());
        for oid in commits {
            after_prepared(&state, &repository, oid);
        }
        commits = state
            .repositories
            .write()
            .unwrap()
            .pre_process_chunk(&mut rw, &repository, 100);
    }
    Ok(())
}

fn after_prepared(
    state: &backend::AppState,
    repository: &hyperast_vcs_git::processing::ConfiguredRepo2,
    oid: hyperast_vcs_git::git::Oid,
) {
    let repositories = state.repositories.read().unwrap();
    let commit = repositories.get_commit(&repository.config, &oid).unwrap();
    let store = &state.repositories.read().unwrap().processor.main_stores;
    let n = store.node_store.resolve(commit.ast_root);
    use hyperast::types::WithStats;
    let Ok(dd) = n.get_component::<hyperast::scripting::DerivedData>() else {
        println!("{} {} N/A {}", &oid, commit.processing_time(), n.size());
        return;
    };
    println!(
        "{} {} {:?} {}",
        &oid,
        commit.processing_time(),
        &dd.0,
        n.size()
    );
}
