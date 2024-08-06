use crate::SharedState;
use axum::Json;
use hyper_ast::store::defaults::NodeIdentifier;
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
    // TODO disable the incriminated pattern for subsequent matches
    /// checked per individual match
    /// if triggered on first search (ie. first commit searched) it return directly
    /// if triggered later, divide the numer of commits remaining to analyze by 2 each time (ie. `commits`` field)
    #[serde(default = "default_max_matches")]
    pub max_matches: u64,
    /// checked each match (in milli seconds)
    #[serde(default = "default_timeout")]
    pub timeout: u64,
}

fn default_max_matches() -> u64 {
    500
}

fn default_timeout() -> u64 {
    1000
}

#[derive(Serialize)]
pub enum QueryingError {
    ProcessingError(String),
    MissingLanguage(String),
    ParsingError(String),
    MatchingErrOnFirst(MatchingError<ComputeResultIdentified>),
}

#[derive(Debug, Serialize, Clone)]
pub enum MatchingError<T> {
    TimeOut(T),
    MaxMatches(T),
}
impl<T> MatchingError<T> {
    fn map<U>(self, f: impl Fn(T) -> U) -> MatchingError<U> {
        match self {
            MatchingError::TimeOut(x) => MatchingError::TimeOut(f(x)),
            MatchingError::MaxMatches(x) => MatchingError::MaxMatches(f(x)),
        }
    }
}

#[derive(Serialize)]
pub struct ComputeResults {
    pub prepare_time: f64,
    pub matching_error_count: usize,
    pub results: Vec<Result<ComputeResultIdentified, MatchingError<ComputeResultIdentified>>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct ComputeResultIdentified {
    pub commit: String,
    #[serde(flatten)]
    pub inner: ComputeResult,
}

#[derive(Debug, Serialize, Clone)]
pub struct ComputeResult {
    pub compute_time: f64,
    pub result: Vec<u64>,
}
impl ComputeResult {
    fn with(self, commit_oid: &hyper_ast_cvs_git::git::Oid) -> ComputeResultIdentified {
        ComputeResultIdentified {
            commit: commit_oid.to_string(),
            inner: self,
        }
    }
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
        max_matches,
        timeout,
    } = query;
    let timeout = std::time::Duration::from_millis(timeout);
    let mut proc_commit_limit = commits;
    let config = if language == "Java" {
        hyper_ast_cvs_git::processing::RepoConfig::JavaMaven
    } else if language == "Cpp" {
        hyper_ast_cvs_git::processing::RepoConfig::JavaMaven
    } else {
        hyper_ast_cvs_git::processing::RepoConfig::Any
    };
    let language: tree_sitter::Language = hyper_ast_cvs_git::resolve_language(&language)
        .ok_or_else(|| QueryingError::MissingLanguage(language))?;
    let repo_spec = hyper_ast_cvs_git::git::Forge::Github.repo(user, name);
    let repo = state
        .repositories
        .read()
        .unwrap()
        .get_config(repo_spec.clone());
    let repo = match repo {
        Some(repo) => repo,
        None => {
            let configs = &mut state.repositories.write().unwrap();
            configs.register_config(repo_spec.clone(), config);
            log::error!("missing config for {}", repo_spec);
            configs.get_config(repo_spec.clone()).unwrap()
        }
    };
    let mut repo = repo.fetch();
    log::warn!("done cloning {}", &repo.spec);
    let commits = crate::utils::handle_pre_processing(&state, &mut repo, "", &commit, commits)
        .map_err(|x| QueryingError::ProcessingError(x))?;
    log::info!("done construction of {commits:?} in  {}", repo.spec);
    let language: tree_sitter::Language = language.clone();

    let query = if INCREMENTAL_QUERIES {
        hyper_ast_tsquery::Query::with_precomputed(
            &query,
            hyper_ast_gen_ts_java::language(),
            hyper_ast_cvs_git::java_processor::SUB_QUERIES,
        )
        .map(|x| x.1)
    } else {
        hyper_ast_tsquery::Query::new(&query, language)
    }
    .map_err(|e| QueryingError::ParsingError(e.to_string()))?;

    log::info!("done query construction");
    let prepare_time = now.elapsed().as_secs_f64();
    let mut results = vec![];
    let mut matching_error_count = 0;
    for commit_oid in &commits {
        if results.len() > proc_commit_limit {
            return Ok(Json(ComputeResults {
                prepare_time,
                matching_error_count,
                results,
            }));
        }
        let mut oid = commit_oid.to_string();
        oid.truncate(6);
        log::info!("start querying {}", oid);
        let repositories = state.repositories.read().unwrap();
        let commit = repositories.get_commit(&repo.config, commit_oid).unwrap();
        let code = commit.ast_root;
        let stores = &repositories.processor.main_stores;
        let result = simple_aux(stores, code, &query, timeout, max_matches);
        let result = match result {
            Ok(inner) => Ok(inner.with(commit_oid)),
            Err(err) if results.is_empty() => {
                return Err(QueryingError::MatchingErrOnFirst(
                    err.map(|inner| inner.with(commit_oid)),
                ))
            }
            Err(err) => {
                matching_error_count += 1;
                proc_commit_limit /= 2;
                Err(err.map(|inner| inner.with(commit_oid)))
            }
        };
        log::info!("done querying {}", oid);
        results.push(result);
    }
    log::info!("done querying of {commits:?} in  {}", repo.spec);
    Ok(Json(ComputeResults {
        prepare_time,
        matching_error_count,
        results,
    }))
}

fn simple_aux(
    stores: &hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    code: NodeIdentifier,
    query: &hyper_ast_tsquery::Query,
    timeout: std::time::Duration,
    max_matches: u64,
) -> Result<ComputeResult, MatchingError<ComputeResult>> {
    let pos = hyper_ast::position::StructuralPosition::new(code);
    let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(stores, pos);
    let qcursor = query.matches(cursor);
    let now = Instant::now();
    let mut result = vec![0; query.enabled_pattern_count()];
    for m in qcursor {
        let i = m.pattern_index;
        let i = query.enabled_pattern_index(i).unwrap();
        result[i as usize] += 1;
        let compute_time = now.elapsed();
        if compute_time >= timeout {
            let compute_time = now.elapsed().as_secs_f64();
            return Err(MatchingError::TimeOut(ComputeResult {
                result,
                compute_time,
            }));
        } else if result[i as usize] > max_matches {
            // TODO disable the pattern, return the new query
            let compute_time = now.elapsed().as_secs_f64();
            return Err(MatchingError::MaxMatches(ComputeResult {
                result,
                compute_time,
            }));
        }

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
