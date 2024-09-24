use crate::{smells::globalize, SharedState};
use axum::{response::IntoResponse, Json};
use http::{HeaderMap, StatusCode};
use hyper_ast::{
    position::position_accessors::WithPreOrderOffsets, store::defaults::NodeIdentifier,
};
use hyper_ast_cvs_git::git::Oid;
use hyper_diff::{
    decompressed_tree_store::ShallowDecompressedTreeStore,
    matchers::mapping_store::MultiMappingStore,
};
use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Serialize, Deserialize, Clone)]
pub struct Param {
    user: String,
    name: String,
    commit: String,
}

#[derive(Serialize, Deserialize, Clone)]
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
    MatchingError(MatchingError<ComputeResult>),
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
    fn with(self, commit_oid: &Oid) -> ComputeResultIdentified {
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
) -> Result<ComputeResults, QueryingError> {
    let now = Instant::now();
    let Param { user, name, commit } = path.clone();
    let mut additional = commit.split("/");
    let commit = additional.next().unwrap();
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
        .map_err(|x| QueryingError::ProcessingError(x.to_string()))?;
    log::info!("done construction of {commits:?} in  {}", repo.spec);
    let language: tree_sitter::Language = language.clone();

    let query = if INCREMENTAL_QUERIES {
        hyper_ast_tsquery::Query::with_precomputed(
            &query,
            hyper_ast_gen_ts_java::language(),
            hyper_ast_cvs_git::java_processor::sub_queries(),
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
            return Ok(ComputeResults {
                prepare_time,
                matching_error_count,
                results,
            });
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
        log::info!("-st {}", oid);
        results.push(result);
    }
    log::info!("done querying of {commits:?} in  {}", repo.spec);
    Ok(ComputeResults {
        prepare_time,
        matching_error_count,
        results,
    })
}

pub fn streamed(mut state: SharedState, path: Param, content: Content) -> axum::response::Response {
    let now = Instant::now();

    let mut headers = HeaderMap::new();

    let (repo, commits) = match pre_repo(&mut state, &path, &content) {
        Ok((x, y)) => (x, y),
        Err(err) => {
            headers.insert(
                "error_parsing",
                serde_json::to_string(&err).unwrap().try_into().unwrap(),
            );

            return (StatusCode::BAD_REQUEST, headers, "").into_response();
        }
    };

    headers.insert("commits", commits.len().into());

    let pre_query = pre_query(&mut state, &path, &content);
    let Content {
        commits: mut proc_commit_limit,
        max_matches,
        timeout,
        ..
    } = content.clone();
    let timeout = std::time::Duration::from_millis(timeout);
    log::info!("done query construction");
    let prepare_time = now.elapsed().as_secs_f64();
    headers.insert("prepare_time", prepare_time.to_string().try_into().unwrap());
    let query = match pre_query {
        Ok(x) => x,
        Err(err) => {
            headers.insert(
                "error_query",
                serde_json::to_string(&err).unwrap().try_into().unwrap(),
            );
            return (StatusCode::BAD_REQUEST, headers, "").into_response();
        }
    };

    headers.insert(
        "table_head",
        serde_json::to_string(
            &(0..query.enabled_pattern_count())
                .map(|x| x.to_string())
                .collect::<Vec<_>>(),
        )
        .unwrap()
        .try_into()
        .unwrap(),
    );

    let it = commits
        .into_iter()
        .enumerate()
        .map_while(move |(i, commit_oid)| {
            if proc_commit_limit == 0 {
                return None;
            }
            if i >= proc_commit_limit {
                return None;
            }
            let mut oid = commit_oid.to_string();
            oid.truncate(6);
            log::trace!("start querying {} in  {}", oid, repo.spec);
            let repositories = state.repositories.read().unwrap();
            let commit = repositories.get_commit(&repo.config, &commit_oid).unwrap();
            let code = commit.ast_root;
            let stores = &repositories.processor.main_stores;
            let result = simple_aux(stores, code, &query, timeout, max_matches);
            let result = match result {
                Ok(inner) => Ok(inner.with(&commit_oid)),
                Err(err) => {
                    log::warn!("{:?}", err);
                    Err(err.map(|inner| inner.with(&commit_oid)))
                }
            };
            log::info!("done querying-st {}", oid);
            if result.is_err() {
                proc_commit_limit /= 2;
                if i == 0 {
                    proc_commit_limit = 0;
                    log::warn!(
                        "stopping early, because of error on first result {:?}",
                        result
                    );
                    // no need for a special error for first occ.,
                    // it is also obvious to the client when there is an error on first commit
                }
            }
            Some(result)
        });

    let st_vals = futures::stream::iter(it.map(|x| {
        match x {
            Ok(x) => serde_json::to_string(&x).map_err(|e| e.to_string()),
            Err(x) => serde_json::to_string(&x).map_err(|e| e.to_string()),
        }
        // x.map(|x| serde_json::to_string(&x).unwrap())
        //     .map_err(|x| serde_json::to_string(&x).unwrap())
    }));

    (
        StatusCode::OK,
        headers,
        axum::body::StreamBody::new(st_vals),
    )
        .into_response()
}

fn pre_repo(
    state: &mut SharedState,
    path: &Param,
    content: &Content,
) -> Result<(hyper_ast_cvs_git::processing::ConfiguredRepo2, Vec<Oid>), QueryingError> {
    let Param { user, name, commit } = path.clone();
    let Content {
        language,
        query,
        commits,
        max_matches,
        timeout,
    } = content.clone();
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

    Ok((repo, commits))
}

fn pre_query(
    state: &mut SharedState,
    path: &Param,
    content: &Content,
) -> Result<hyper_ast_tsquery::Query, QueryingError> {
    let Param { user, name, commit } = path.clone();
    let Content {
        language,
        query,
        commits,
        max_matches,
        timeout,
    } = content.clone();
    let config = if language == "Java" {
        hyper_ast_cvs_git::processing::RepoConfig::JavaMaven
    } else if language == "Cpp" {
        hyper_ast_cvs_git::processing::RepoConfig::JavaMaven
    } else {
        hyper_ast_cvs_git::processing::RepoConfig::Any
    };
    let language: tree_sitter::Language = hyper_ast_cvs_git::resolve_language(&language)
        .ok_or_else(|| QueryingError::MissingLanguage(language))?;
    let language: tree_sitter::Language = language.clone();
    let query = if INCREMENTAL_QUERIES {
        hyper_ast_tsquery::Query::with_precomputed(
            &query,
            hyper_ast_gen_ts_java::language(),
            hyper_ast_cvs_git::java_processor::sub_queries(),
        )
        .map(|x| x.1)
    } else {
        hyper_ast_tsquery::Query::new(&query, language)
    }
    .map_err(|e| QueryingError::ParsingError(e.to_string()))?;
    Ok(query)
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

#[derive(Serialize)]
pub struct ComputeResultsDifferential {
    pub prepare_time: f64,
    pub results: Vec<(crate::smells::CodeRange, crate::smells::CodeRange)>,
}
#[derive(Serialize, Deserialize, Clone)]
pub struct ParamDifferential {
    user: String,
    name: String,
    commit: String,
    baseline: String,
}

pub fn differential(
    query: Content,
    state: SharedState,
    path: ParamDifferential,
) -> Result<Json<ComputeResultsDifferential>, QueryingError> {
    let now = Instant::now();
    let ParamDifferential {
        user,
        name,
        commit,
        baseline,
    } = path.clone();
    let Content {
        language,
        query,
        max_matches,
        timeout,
        ..
    } = query;
    let timeout = std::time::Duration::from_millis(timeout);
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
    let commit = crate::utils::handle_pre_processing(&state, &mut repo, "", &commit, 1)
        .map_err(|x| QueryingError::ProcessingError(x.to_string()))?[0];
    let baseline = crate::utils::handle_pre_processing(&state, &mut repo, "", &baseline, 1)
        .map_err(|x| QueryingError::ProcessingError(x.to_string()))?[0];
    log::info!(
        "done construction of {commit:?} and {baseline:?} in  {}",
        repo.spec
    );
    let language: tree_sitter::Language = language.clone();

    let query = if INCREMENTAL_QUERIES {
        hyper_ast_tsquery::Query::with_precomputed(
            &query,
            hyper_ast_gen_ts_java::language(),
            hyper_ast_cvs_git::java_processor::sub_queries(),
        )
        .map(|x| x.1)
    } else {
        hyper_ast_tsquery::Query::new(&query, language)
    }
    .map_err(|e| QueryingError::ParsingError(e.to_string()))?;

    log::info!("done query construction");
    let prepare_time = now.elapsed().as_secs_f64();
    let current_tr;
    let baseline_results: Vec<_> = {
        let commit_oid = &baseline;
        let mut oid = commit_oid.to_string();
        oid.truncate(6);
        log::info!("start querying {}", oid);
        let repositories = state.repositories.read().unwrap();
        let commit = repositories.get_commit(&repo.config, commit_oid).unwrap();
        let code = commit.ast_root;
        current_tr = code;
        let stores = &repositories.processor.main_stores;
        let result = differential_aux(stores, code, &query, timeout, max_matches)
            .map_err(|e| QueryingError::MatchingError(e))?;

        // (p.make_position(stores), p.iter_offsets().collect())

        result
    };
    let other_tr;
    let results: Vec<_> = {
        let commit_oid = &commit;
        let mut oid = commit_oid.to_string();
        oid.truncate(6);
        log::info!("start querying {}", oid);
        let repositories = state.repositories.read().unwrap();
        let commit = repositories.get_commit(&repo.config, commit_oid).unwrap();
        let code = commit.ast_root;
        other_tr = code;
        let stores = &repositories.processor.main_stores;
        let result = differential_aux(stores, code, &query, timeout, max_matches)
            .map_err(|e| QueryingError::MatchingError(e))?;
        result
    };
    log::info!(
        "done querying of {commit:?} and {baseline:?} in  {}",
        repo.spec
    );
    if results.len() == baseline_results.len() {}
    log::info!(
        "lens results/baseline_results: {}/{}",
        results.len(),
        baseline_results.len()
    );

    let repositories = state.repositories.read().unwrap();
    let stores = &repositories.processor.main_stores;

    let hyperast = &hyper_ast_cvs_git::no_space::as_nospaces(stores);
    let (src_tree, dst_tree) =
        crate::utils::get_pair_simp(&state.partial_decomps, hyperast, &current_tr, &other_tr);
    let (src_tree, dst_tree) = (src_tree.get_mut(), dst_tree.get_mut());

    let mut mapper = hyper_diff::matchers::Mapper {
        hyperast,
        mapping: hyper_diff::matchers::Mapping {
            src_arena: src_tree,
            dst_arena: dst_tree,
            mappings: hyper_diff::matchers::mapping_store::VecStore::<u16>::default(),
        },
    };

    let subtree_mappings =
        { crate::matching::top_down(hyperast, mapper.mapping.src_arena, mapper.mapping.dst_arena) };

    log::info!("done top_down mapping");

    let baseline_results: Vec<_> = baseline_results
        .iter()
        .filter(|x| {
            log::info!("filtering");
            let (_, _, no_spaces_path_to_target) =
                hyper_ast::position::compute_position_with_no_spaces(
                    current_tr,
                    &mut x.iter_offsets(),
                    stores,
                );

            let mut src = mapper.src_arena.root();
            for i in no_spaces_path_to_target {
                if subtree_mappings.get_dsts(&src).is_empty() {
                } else {
                    let a = globalize(
                        &repo,
                        baseline,
                        (x.make_position(stores), x.iter_offsets().collect()),
                    );
                    log::info!("mapped: {a:?}");
                    return false;
                }
                use hyper_diff::decompressed_tree_store::LazyDecompressedTreeStore;
                let cs = mapper
                    .src_arena
                    .decompress_children(&hyperast.node_store, &src);
                if cs.is_empty() {
                    log::info!("empty");
                    return true;
                }
                src = cs[i as usize];
            }
            log::info!("mapped = {}", subtree_mappings.get_dsts(&src).is_empty());
            subtree_mappings.get_dsts(&src).is_empty()
        })
        .collect();
    log::info!("done filtering");

    let results = baseline_results
        .into_iter()
        .map(|p| {
            log::info!("globalizing");
            (
                globalize(
                    &repo,
                    baseline,
                    (p.make_position(stores), p.iter_offsets().collect()),
                ),
                globalize(
                    &repo,
                    commit,
                    (p.make_position(stores), p.iter_offsets().collect()),
                ),
            )
        })
        .collect();
    log::info!("done globalizing");

    Ok(Json(ComputeResultsDifferential {
        prepare_time,
        results,
    }))
}

fn differential_aux(
    stores: &hyper_ast::store::SimpleStores<hyper_ast_cvs_git::TStore>,
    code: NodeIdentifier,
    query: &hyper_ast_tsquery::Query,
    _timeout: std::time::Duration,
    _max_matches: u64,
) -> Result<
    Vec<hyper_ast::position::StructuralPosition<NodeIdentifier, u16>>,
    MatchingError<ComputeResult>,
> {
    let pos = hyper_ast::position::StructuralPosition::new(code);
    let cursor = hyper_ast_tsquery::hyperast::TreeCursor::new(stores, pos);
    let qcursor = query.matches(cursor);
    let now = Instant::now();
    assert_eq!(
        query.enabled_pattern_count(),
        1,
        "details on a single pattern"
    );
    let mut results = vec![];
    let rrr = query
        .capture_index_for_name("root")
        .expect("@root at the en of query pattern");
    for m in qcursor {
        let i = m.pattern_index;
        let i = query.enabled_pattern_index(i).unwrap();
        assert_eq!(i, 0, "details on a single pattern");
        let node = m
            .nodes_for_capture_index(rrr)
            .next()
            .expect("@root at the en of query pattern");
        results.push(node.pos.clone());
        let compute_time = now.elapsed();
        // if compute_time >= timeout {
        //     let compute_time = now.elapsed().as_secs_f64();
        //     return Err(MatchingError::TimeOut(ComputeResult {
        //         result,
        //         compute_time,
        //     }));
        // } else if result[i as usize] > max_matches {
        //     // TODO disable the pattern, return the new query
        //     let compute_time = now.elapsed().as_secs_f64();
        //     return Err(MatchingError::MaxMatches(ComputeResult {
        //         result,
        //         compute_time,
        //     }));
        // }

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
    Ok(results)
}
