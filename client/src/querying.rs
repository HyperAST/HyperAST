use crate::SharedState;
use axum::Json;
use hyper_ast_gen_ts_tsquery::search::steped;
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Deserialize, Clone)]
pub struct Param {
    user: String,
    name: String,
    commit: String,
}

#[derive(Deserialize, Clone)]
pub struct Content {
    pub language: String,
    pub query: String,
    pub commits: usize,
}

#[derive(Debug, Serialize, Clone)]
pub enum QueryingError {
    MissingLanguage(String),
    ParsingError(String),
}

#[derive(Serialize)]
pub struct ComputeResults {
    pub prepare_time: f64,
    pub results: Vec<Result<ComputeResultIdentified, String>>,
}

#[derive(Serialize)]
pub struct ComputeResultIdentified {
    pub commit: String,
    #[serde(flatten)]
    pub inner: ComputeResult,
}

#[derive(Serialize)]
pub struct ComputeResult {
    pub compute_time: f64,
    pub result: Vec<u64>,
}

const INCREMENTAL_QUERIES: bool = true;

pub fn simple(
    query: Content,
    state: SharedState,
    path: Param,
) -> Result<Json<ComputeResults>, QueryingError> {
    let now = Instant::now();
    let Param { user, name, commit } = path.clone();
    let Content {
        language,
        query,
        commits,
    } = query;
    let language: tree_sitter::Language = hyper_ast_cvs_git::resolve_language(&language)
        .ok_or_else(|| QueryingError::MissingLanguage(language))?;
    let repo_spec = hyper_ast_cvs_git::git::Forge::Github.repo(user, name);
    let repo = state
        .repositories
        .write()
        .unwrap()
        .get_config(repo_spec.clone());
    let repo = match repo {
        Some(repo) => repo,
        None => {
            let configs = &mut state.repositories.write().unwrap();
            configs.register_config(
                repo_spec.clone(),
                hyper_ast_cvs_git::processing::RepoConfig::JavaMaven,
            );
            log::error!("missing config for {}", repo_spec);
            configs.get_config(repo_spec.clone()).unwrap()
        }
    };
    // .ok_or_else(|| ScriptingError::Other("missing config for repository".to_string()))?;
    let mut repo = repo.fetch();
    log::warn!("done cloning {}", &repo.spec);
    let commits = state
        .repositories
        .write()
        .unwrap()
        .pre_process_with_limit(&mut repo, "", &commit, commits)
        .unwrap();
    log::info!("done construction of {commits:?} in  {}", repo.spec);
    let language: tree_sitter::Language = language.clone();

    let query = if INCREMENTAL_QUERIES {
        hyper_ast_tsquery::Query::with_precomputed(
            &query,
            hyper_ast_gen_ts_java::language(),
            &hyper_ast_cvs_git::java_processor::SUB_QUERIES[0..1],
        )
        .map(|x| x.1)
    } else {
        hyper_ast_tsquery::Query::new(&query, language)
    }
    .map_err(|e| QueryingError::ParsingError(e.to_string()))?;

    log::info!("done query construction");
    let prepare_time = now.elapsed().as_secs_f64();
    let mut results = vec![];
    for commit_oid in &commits {
        let mut oid = commit_oid.to_string();
        oid.truncate(6);
        log::info!("start querying {}", oid);
        let result = simple_aux(&state, &repo, commit_oid, &query)
            .map(|inner| ComputeResultIdentified {
                commit: commit_oid.to_string(),
                inner,
            })
            .map_err(|err| format!("{:?}", err));
        log::info!("done querying {}", oid);
        results.push(result);
    }
    log::info!("done querying of {commits:?} in  {}", repo.spec);
    Ok(Json(ComputeResults {
        prepare_time,
        results,
    }))
}

fn simple_aux(
    state: &crate::AppState,
    repo: &hyper_ast_cvs_git::processing::ConfiguredRepo2,
    commit_oid: &hyper_ast_cvs_git::git::Oid,
    query: &hyper_ast_tsquery::Query,
) -> Result<ComputeResult, QueryingError> {
    let repositories = state.repositories.read().unwrap();
    let commit = repositories.get_commit(&repo.config, commit_oid).unwrap();
    let code = commit.ast_root;
    let stores = &repositories.processor.main_stores;

    let qcursor = query.matches(hyper_ast_tsquery::hyperast::TreeCursor::new(
        stores,
        hyper_ast::position::StructuralPosition::new(code),
    ));
    let now = Instant::now();
    let mut result = vec![0; query.pattern_count()];
    for m in qcursor {
        result[m.pattern_index.to_usize()] += 1;
        // dbg!(m.pattern_index);
        // dbg!(m.captures.len());
        // for c in &m.captures {
        //     let i = c.index;
        //     dbg!(i);
        //     let name = query.capture_name(i);
        //     dbg!(name);
        //     use hyper_ast::position::TreePath;
        //     let n = c.node.pos.node().unwrap();
        //     let n = hyper_ast::nodes::SyntaxSerializer::new(c.node.stores, *n);
        //     dbg!(n.to_string());
        // }
    }
    let compute_time = now.elapsed().as_secs_f64();
    Ok(ComputeResult {
        result,
        compute_time,
    })
}
