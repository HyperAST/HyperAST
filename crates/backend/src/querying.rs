use crate::{smells::globalize, SharedState};
use axum::{response::IntoResponse, Json};
use http::{HeaderMap, StatusCode};
use hyper_diff::{
    decompressed_tree_store::ShallowDecompressedTreeStore,
    matchers::{mapping_store::MultiMappingStore, Decompressible},
};
use hyperast::{
    position::position_accessors::WithPreOrderOffsets,
    store::defaults::NodeIdentifier,
    types::{Children, Childrn, HyperAST, Typed, WithChildren, WithStats},
};
use hyperast_vcs_git::git::Oid;
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
    pub precomp: Option<String>,
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
        precomp,
        commits,
        max_matches,
        timeout,
    } = query;
    let timeout = std::time::Duration::from_millis(timeout);
    let mut proc_commit_limit = commits;
    let config = if language == "Java" {
        hyperast_vcs_git::processing::RepoConfig::JavaMaven
    } else if language == "Cpp" {
        hyperast_vcs_git::processing::RepoConfig::CppMake
    } else {
        hyperast_vcs_git::processing::RepoConfig::Any
    };
    let lang = &language;
    let language: tree_sitter::Language = hyperast_vcs_git::resolve_language(&language)
        .ok_or_else(|| QueryingError::MissingLanguage(language.to_string()))?;
    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo(user, name);
    let repo = state
        .repositories
        .read()
        .unwrap()
        .get_config(repo_spec.clone());
    let repo = match repo {
        Some(repo) => repo,
        None => {
            let configs = &mut state.repositories.write().unwrap();
            if let Some(precomp) = precomp {
                configs.register_config_with_prequeries(repo_spec.clone(), config, &[&precomp]);
            } else {
                // configs.register_config_alt_lang(repo_spec.clone(), config, "C");
                configs.register_config(repo_spec.clone(), config);
            }
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

    let precomputeds = INCREMENTAL_QUERIES
        .then(|| {
            state
                .repositories
                .write()
                .unwrap()
                .get_precomp_query(repo.config, lang)
        })
        .flatten();

    let query = if let Some(precomputeds) = precomputeds {
        hyperast_tsquery::Query::with_precomputed(&query, language, precomputeds).map(|x| x.1)
    } else {
        hyperast_tsquery::Query::new(&query, language)
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

    let language: tree_sitter::Language =
        match hyperast_vcs_git::resolve_language(&content.language) {
            Some(x) => x,
            None => {
                let err = QueryingError::MissingLanguage(content.language);
                headers.insert(
                    "error_parsing",
                    serde_json::to_string(&err).unwrap().try_into().unwrap(),
                );

                return (StatusCode::BAD_REQUEST, headers, "").into_response();
            }
        };

    let (repo, commits) = match pre_repo(&mut state, &path, &content) {
        Ok((x, y)) => (x, y),
        Err(err) => {
            headers.insert("error_parsing", err.to_string().try_into().unwrap());

            return (StatusCode::BAD_REQUEST, headers, "").into_response();
        }
    };

    headers.insert("commits", commits.len().into());

    let pre_query = pre_query(&mut state, &path, &content, repo.config);
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
        .into_iter() // TODO use chunks to reduce presure on state.repositories' lock, need some bench before doing this opt ;)
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
        axum::body::Body::from_stream(st_vals),
    )
        .into_response()
}

fn pre_repo(
    state: &mut SharedState,
    path: &Param,
    content: &Content,
) -> Result<(hyperast_vcs_git::processing::ConfiguredRepo2, Vec<Oid>), Box<dyn std::error::Error>> {
    let Param { user, name, commit } = path.clone();
    let mut additional = commit.split("/");
    let commit = additional.next().unwrap();
    let Content {
        language,
        query: _,
        precomp,
        commits,
        max_matches: _,
        timeout: _,
    } = content.clone();
    let config = if language == "Java" {
        hyperast_vcs_git::processing::RepoConfig::JavaMaven
    } else if language == "Cpp" {
        hyperast_vcs_git::processing::RepoConfig::CppMake
    } else {
        hyperast_vcs_git::processing::RepoConfig::Any
    };
    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo(user, name);
    let repo = state
        .repositories
        .read()
        .unwrap()
        .get_config(repo_spec.clone());
    let repo = match repo {
        Some(_) | None => {
            let configs = &mut state.repositories.write().unwrap();
            if let Some(precomp) = precomp {
                configs.register_config_with_prequeries(repo_spec.clone(), config, &[&precomp]);
            } else {
                configs.register_config(repo_spec.clone(), config);
            }
            log::error!("missing config for {}", repo_spec);
            configs.get_config(repo_spec.clone()).unwrap()
        }
    };
    let repo = repo.fetch();
    log::warn!("done cloning {}", &repo.spec);
    let afters = [commit].into_iter().chain(additional.into_iter());
    let rw = crate::utils::walk_commits_multi(&repo, afters)?.take(commits);
    assert!(state.repositories.try_write().is_ok());
    let commits = crate::utils::handle_pre_processing_aux(state, &repo, rw);
    log::info!("done construction of {commits:?} in  {}", repo.spec);

    Ok((repo, commits))
}

fn pre_query(
    state: &mut SharedState,
    path: &Param,
    content: &Content,
    repo_config: hyperast_vcs_git::processing::ParametrizedCommitProcessorHandle,
) -> Result<hyperast_tsquery::Query, QueryingError> {
    let Param { user, name, commit } = path.clone();
    let mut additional = commit.split("/");
    let commit = additional.next().unwrap();
    let Content {
        language,
        query,
        precomp: _,
        commits: _,
        max_matches: _,
        timeout: _,
    } = &content;
    let config = if language == "Java" {
        hyperast_vcs_git::processing::RepoConfig::JavaMaven
    } else if language == "Cpp" {
        hyperast_vcs_git::processing::RepoConfig::CppMake
    } else {
        hyperast_vcs_git::processing::RepoConfig::Any
    };
    let lang = &language;
    let language: tree_sitter::Language = hyperast_vcs_git::resolve_language(&language)
        .ok_or_else(|| QueryingError::MissingLanguage(language.to_string()))?;
    let language: tree_sitter::Language = language.clone();

    let precomputeds = INCREMENTAL_QUERIES.then(|| {
        state
            .repositories
            .write()
            .unwrap()
            .get_precomp_query(repo_config, lang)
    });
    let query = if let Some(Some(precomputeds)) = precomputeds {
        hyperast_tsquery::Query::with_precomputed(&query, language, precomputeds).map(|x| x.1)
    } else {
        hyperast_tsquery::Query::new(&query, language)
    }
    .map_err(|e| QueryingError::ParsingError(e.to_string()))?;
    Ok(query)
}

fn simple_aux(
    stores: &hyperast::store::SimpleStores<hyperast_vcs_git::TStore>,
    code: NodeIdentifier,
    query: &hyperast_tsquery::Query,
    timeout: std::time::Duration,
    max_matches: u64,
) -> Result<ComputeResult, MatchingError<ComputeResult>> {
    let pos = hyperast::position::StructuralPosition::new(code);
    let cursor = hyperast_tsquery::hyperast_cursor::TreeCursor::new(stores, pos);
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
        //     use hyperast::position::TreePath;
        //     let n = c.node.pos.node().unwrap();
        //     let n = hyperast::nodes::SyntaxSerializer::new(c.node.stores, *n);
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
        precomp,
        max_matches,
        timeout,
        ..
    } = query;
    let timeout = std::time::Duration::from_millis(timeout);
    let config = if language == "Java" {
        hyperast_vcs_git::processing::RepoConfig::JavaMaven
    } else if language == "Cpp" {
        hyperast_vcs_git::processing::RepoConfig::CppMake
    } else {
        hyperast_vcs_git::processing::RepoConfig::Any
    };
    let lang = &language;
    let language: tree_sitter::Language = hyperast_vcs_git::resolve_language(&language)
        .ok_or_else(|| QueryingError::MissingLanguage(language.to_string()))?;
    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo(user, name);
    let repo = state
        .repositories
        .read()
        .unwrap()
        .get_config(repo_spec.clone());
    let repo = match repo {
        Some(repo) => repo,
        None => {
            let configs = &mut state.repositories.write().unwrap();
            if let Some(precomp) = precomp {
                configs.register_config_with_prequeries(repo_spec.clone(), config, &[&precomp]);
            } else {
                configs.register_config(repo_spec.clone(), config);
            }
            log::error!("missing config for {}", repo_spec);
            configs.get_config(repo_spec.clone()).unwrap()
        }
    };
    let mut repo = repo.fetch();
    log::info!("done cloning {}", &repo.spec);
    let commit = crate::utils::handle_pre_processing(&state, &mut repo, "", &commit, 1)
        .map_err(|x| QueryingError::ProcessingError(x.to_string()))?[0];
    let baseline = crate::utils::handle_pre_processing(&state, &mut repo, "", &baseline, 1)
        .map_err(|x| QueryingError::ProcessingError(x.to_string()))?[0];
    log::info!(
        "done construction of {commit:?} and {baseline:?} in  {}",
        repo.spec
    );
    let language: tree_sitter::Language = language.clone();

    let precomputeds = INCREMENTAL_QUERIES.then(|| {
        state
            .repositories
            .write()
            .unwrap()
            .get_precomp_query(repo.config, lang)
    });
    let query = if let Some(Some(precomputeds)) = precomputeds {
        hyperast_tsquery::Query::with_precomputed(&query, language, precomputeds).map(|x| x.1)
    } else {
        hyperast_tsquery::Query::new(&query, language)
    }
    .map_err(|e| QueryingError::ParsingError(e.to_string()))?
    .with_one_pattern_enabled(0)
    .map_err(|_| {
        QueryingError::ParsingError("exactly one enabled pattern is expected".to_string())
    })?;

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
        "lengths results/baseline_results: {}/{}",
        results.len(),
        baseline_results.len()
    );

    let repositories = state.repositories.read().unwrap();
    let stores = &repositories.processor.main_stores;

    let hyperast = &hyperast_vcs_git::no_space::as_nospaces2(stores);
    let (src_tree, dst_tree) =
        crate::utils::get_pair_simp(&state.partial_decomps, hyperast, &current_tr, &other_tr);
    let (src_tree, dst_tree) = (src_tree.get_mut(), dst_tree.get_mut());
    let src_tree = Decompressible {
        hyperast,
        decomp: src_tree,
    };
    let dst_tree = Decompressible {
        hyperast,
        decomp: dst_tree,
    };

    let mut mapper = hyper_diff::matchers::Mapper {
        hyperast,
        mapping: hyper_diff::matchers::Mapping {
            src_arena: src_tree,
            dst_arena: dst_tree,
            mappings: hyper_diff::matchers::mapping_store::VecStore::<u16>::default(),
        },
    };

    let subtree_mappings = {
        crate::matching::top_down(
            hyperast,
            mapper.mapping.src_arena.decomp,
            mapper.mapping.dst_arena.decomp,
        )
    };

    log::info!("done top_down mapping");

    let baseline_results: Vec<_> = baseline_results
        .iter()
        .filter(|x| {
            log::debug!("filtering");
            let (_, _, no_spaces_path_to_target) =
                hyperast::position::compute_position_with_no_spaces(
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
                    log::debug!("mapped: {a:?}");
                    return false;
                }
                use hyper_diff::decompressed_tree_store::LazyDecompressedTreeStore;
                let cs = mapper.src_arena.decompress_children(&src);
                if cs.is_empty() {
                    log::debug!("empty");
                    return true;
                }
                // Gracefully handling possibly wrong param
                // before: // src = cs[i as usize];
                let Some(s) = cs.get(i as usize) else {
                    let a = globalize(
                        &repo,
                        baseline,
                        (x.make_position(stores), x.iter_offsets().collect()),
                    );
                    let id = mapper.src_arena.original(&src);
                    let t = hyperast.resolve_type(&id);
                    let (_, _, no_spaces_path_to_target) =
                        hyperast::position::compute_position_with_no_spaces(
                            current_tr,
                            &mut x.iter_offsets(),
                            stores,
                        );
                    log::error!(
                        "no such child: {a:?} {t:?} {} {} {:?}",
                        cs.len(),
                        i,
                        no_spaces_path_to_target
                    );
                    return true;
                };
                src = *s;
            }
            log::debug!("mapped = {}", subtree_mappings.get_dsts(&src).is_empty());
            subtree_mappings.get_dsts(&src).is_empty()
        })
        .collect();
    log::info!("done filtering evolutions");

    let results = baseline_results
        .into_iter()
        .map(|p| {
            log::debug!("globalizing");
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

    log::info!(
        "done finding evolutions of {commit:?} and {baseline:?} in  {}",
        repo.spec
    );

    Ok(Json(ComputeResultsDifferential {
        prepare_time,
        results,
    }))
}

fn differential_aux(
    stores: &hyperast::store::SimpleStores<hyperast_vcs_git::TStore>,
    code: NodeIdentifier,
    query: &hyperast_tsquery::Query,
    _timeout: std::time::Duration,
    _max_matches: u64,
) -> Result<
    Vec<hyperast::position::StructuralPosition<NodeIdentifier, u16>>,
    MatchingError<ComputeResult>,
> {
    let pos = hyperast::position::StructuralPosition::new(code);
    let cursor = hyperast_tsquery::hyperast_cursor::TreeCursor::new(stores, pos);
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
        //     use hyperast::position::TreePath;
        //     let n = c.node.pos.node().unwrap();
        //     let n = hyperast::nodes::SyntaxSerializer::new(c.node.stores, *n);
        //     dbg!(n.to_string());
        // }
    }
    let compute_time = now.elapsed().as_secs_f64();
    Ok(results)
}
