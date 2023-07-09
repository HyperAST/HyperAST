use std::{fmt::Debug, hash::BuildHasher, thread::sleep, time::Duration};

use axum::{response::IntoResponse, Json};
use enumset::{EnumSet, EnumSetType};
use hyper_ast::{
    position::{
        compute_position, compute_position_and_nodes, compute_position_with_no_spaces,
        compute_range, path_with_spaces, resolve_range,
    },
    store::{defaults::NodeIdentifier, nodes::legion::HashedNodeRef, SimpleStores},
    types::{
        self, HyperAST, IterableChildren, NodeStore, Typed, WithChildren, WithHashs, WithStats,
    },
};
use hyper_ast_cvs_git::{
    git::Repo, multi_preprocessed, preprocessed::child_at_path_tracked, TStore, processing::ConfiguredRepoTrait,
};
use hyper_diff::{
    decompressed_tree_store::{
        DecompressedWithParent, LazyDecompressedTreeStore, PersistedNode,
        ShallowDecompressedTreeStore,
    },
    matchers::{
        mapping_store::{self, MonoMappingStore, MultiMappingStore},
        Mapper,
    },
};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_bool_from_anything;
use tokio::time::Instant;

use crate::{
    changes::{self, DstChanges, SrcChanges},
    matching, no_space,
    utils::get_pair_simp,
    ConfiguredRepoHandle, MappingAloneCache, PartialDecompCache, SharedState,
};

#[derive(Deserialize, Clone, Debug)]
pub struct TrackingParam {
    user: String,
    name: String,
    commit: String,
    file: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TrackingAtPathParam {
    user: String,
    name: String,
    commit: String,
    path: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct TrackingQuery {
    start: Option<usize>,
    end: Option<usize>,
    before: Option<String>,
    #[serde(flatten)]
    flags: Flags,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Debug)]
#[serde(default)]
pub(crate) struct Flags {
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) upd: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) child: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) parent: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) exact_child: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) exact_parent: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) sim_child: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) sim_parent: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) meth: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) typ: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) top: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) file: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) pack: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) dependency: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) dependent: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) references: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub(crate) declaration: bool,
}

impl Flags {
    fn some(&self) -> bool {
        self.upd
            || self.child
            || self.parent
            || self.exact_child
            || self.exact_parent
            || self.sim_child
            || self.sim_parent
            || self.meth
            || self.typ
            || self.top
            || self.file
            || self.pack
            || self.dependency
            || self.dependent
            || self.references
            || self.declaration
    }
}

#[derive(EnumSetType, Debug)]
pub enum FlagsE {
    Upd,
    Child,
    Parent,
    ExactChild,
    ExactParent,
    SimChild,
    SimParent,
    Meth,
    Typ,
    Top,
    File,
    Pack,
    Dependency,
    Dependent,
    References,
    Declaration,
}

impl Into<EnumSet<FlagsE>> for &Flags {
    fn into(self) -> EnumSet<FlagsE> {
        let mut r = EnumSet::new();
        if self.upd {
            r.insert(FlagsE::Upd);
        }
        if self.child {
            r.insert(FlagsE::Child);
        }
        if self.parent {
            r.insert(FlagsE::Parent);
        }
        if self.exact_child {
            r.insert(FlagsE::ExactChild);
        }
        if self.exact_parent {
            r.insert(FlagsE::ExactParent);
        }
        if self.sim_child {
            r.insert(FlagsE::SimChild);
        }
        if self.sim_parent {
            r.insert(FlagsE::SimParent);
        }
        if self.meth {
            r.insert(FlagsE::Meth);
        }
        if self.typ {
            r.insert(FlagsE::Typ);
        }
        if self.top {
            r.insert(FlagsE::Top);
        }
        if self.file {
            r.insert(FlagsE::File);
        }
        if self.pack {
            r.insert(FlagsE::Pack);
        }
        if self.dependency {
            r.insert(FlagsE::Dependency);
        }
        if self.dependent {
            r.insert(FlagsE::Dependent);
        }
        if self.references {
            r.insert(FlagsE::References);
        }
        if self.declaration {
            r.insert(FlagsE::Declaration);
        }
        r
    }
}

#[derive(Deserialize, Serialize)]
pub struct TrackingResult {
    pub compute_time: f64,
    commits_processed: usize,
    src: PieceOfCode,
    intermediary: Option<PieceOfCode>,
    fallback: Option<PieceOfCode>,
    matched: Vec<PieceOfCode>,
}

impl IntoResponse for TrackingResult {
    fn into_response(self) -> axum::response::Response {
        let mut resp = serde_json::to_string(&self).unwrap().into_response();
        let headers = resp.headers_mut();
        headers.insert(
            "Server-Timing",
            format!("track;desc=\"Compute Time\";dur={}", self.compute_time)
                .parse()
                .unwrap(),
        );
        resp
    }
}

impl TrackingResult {
    pub(crate) fn with_changes(
        self,
        (src_changes, dst_changes): (SrcChanges, DstChanges),
    ) -> TrackingResultWithChanges {
        TrackingResultWithChanges {
            track: self,
            src_changes,
            dst_changes,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct TrackingResultWithChanges {
    pub track: TrackingResult,
    src_changes: SrcChanges,
    dst_changes: DstChanges,
}

impl IntoResponse for TrackingResultWithChanges {
    fn into_response(self) -> axum::response::Response {
        let mut resp = serde_json::to_string(&self).unwrap().into_response();
        let headers = resp.headers_mut();
        headers.insert(
            "Server-Timing",
            format!(
                "track;desc=\"Compute Time\";dur={}",
                self.track.compute_time
            )
            .parse()
            .unwrap(),
        );
        resp
    }
}

// impl Display for TrackingResult {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         writeln!()
//     }
// }

#[derive(Deserialize, Serialize, Debug)]
pub struct PieceOfCode {
    user: String,
    name: String,
    commit: String,
    path: Vec<usize>,
    #[serde(serialize_with = "custom_ser")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    path_ids: Vec<NodeIdentifier>, // WARN this is not fetched::NodeIdentifier
    file: String,
    start: usize,
    end: usize,
}

fn custom_ser<S>(x: &Vec<NodeIdentifier>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(x.len()))?;
    for element in x {
        let id: u64 = unsafe { std::mem::transmute(*element) };
        seq.serialize_element(&id)?;
    }
    seq.end()
}

// impl Display for PieceOfCode {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         todo!()
//     }
// }

const MAX_NODES: usize = 200 * 4_000_000;

#[derive(Deserialize, Serialize)]
pub struct TrackingError {
    pub compute_time: f64,
    commits_processed: usize,
    node_processed: usize,
    message: String,
}

impl IntoResponse for TrackingError {
    fn into_response(self) -> axum::response::Response {
        let mut resp = Json(self).into_response();
        *resp.status_mut() = http::StatusCode::FORBIDDEN;
        resp
    }
}

pub fn track_code(
    state: SharedState,
    path: TrackingParam,
    query: TrackingQuery,
) -> Result<TrackingResult, TrackingError> {
    let now = Instant::now();
    let TrackingParam {
        user,
        name,
        commit,
        file,
    } = path;
    let TrackingQuery {
        start,
        end,
        before,
        flags,
    } = query;
    let repo_specifier = hyper_ast_cvs_git::git::Forge::Github.repo(user, name);
    let repo_handle = state
        .repositories
        .write()
        .unwrap()
        .get_config(repo_specifier)
        .ok_or_else(|| TrackingError {
            compute_time: now.elapsed().as_secs_f64(),
            commits_processed: 0,
            node_processed: 0,
            message: "missing config for repository".to_string(),
        })?;
    let mut repository = repo_handle.fetch();
    log::warn!("done cloning {}", repository.spec);
    // let mut get_mut = state.write().unwrap();
    // let state = get_mut.deref_mut();
    let mut commit = commit.clone();
    let mut node_processed = 0;
    let mut commits_processed = 1;
    let mut file = file;
    let mut start = start;
    let mut end = end;
    let mut source = None;
    while node_processed < MAX_NODES {
        commits_processed += 1;
        let commits = state
            .repositories
            .write()
            .unwrap()
            .pre_process_with_limit(&mut repository, "", &commit,2)
            .map_err(|e| TrackingError {
                compute_time: now.elapsed().as_secs_f64(),
                commits_processed: 0,
                node_processed: 0,
                message: e.to_string(),
            })?;
        log::warn!("done construction of {commits:?} in {}", repository.spec);
        let src_oid = commits[0];
        let dst_oid = commits[1];
        match aux(
            state.clone(),
            &repository,
            src_oid,
            dst_oid,
            &file,
            start,
            end,
            &flags,
        ) {
            MappingResult::Direct { src: aaa, matches } => {
                let aaa = aaa.globalize(repository.spec, commit);
                let (src, intermediary) = if let Some(src) = source {
                    (src, Some(aaa))
                } else {
                    (aaa, None)
                };
                return Ok(TrackingResult {
                    compute_time: now.elapsed().as_secs_f64(),
                    commits_processed,
                    src,
                    intermediary,
                    fallback: None,
                    matched: matches,
                }
                .into());
            }
            MappingResult::Missing { src: aaa, fallback } => {
                let aaa = aaa.globalize(repository.spec, commit);
                let (src, intermediary) = if let Some(src) = source {
                    (src, Some(aaa))
                } else {
                    (aaa, None)
                };
                return Ok(TrackingResult {
                    compute_time: now.elapsed().as_secs_f64(),
                    commits_processed,
                    src,
                    intermediary,
                    fallback: Some(fallback),
                    matched: vec![],
                }
                .into());
            }
            MappingResult::Error(err) => Err(TrackingError {
                compute_time: now.elapsed().as_secs_f64(),
                commits_processed,
                node_processed,
                message: err,
            })?,
            MappingResult::Skipped { nodes, src, next } => {
                node_processed += nodes;
                dbg!(src_oid, dst_oid);
                if source.is_none() {
                    source = Some(src.globalize(repository.spec.clone(), commit));
                }
                commit = dst_oid.to_string();
                if next.len() > 1 {
                    log::error!("multiple matches")
                }
                if next.is_empty() {
                    unreachable!()
                } else {
                    let next = &next[0];
                    dbg!(next);
                    file = next.file.to_string();
                    start = Some(next.start);
                    end = Some(next.end);
                }
            }
        }
    }
    Err(TrackingError {
        compute_time: now.elapsed().as_secs_f64(),
        commits_processed,
        node_processed,
        message: format!("reached max number of diffed nodes: (ie. {})", MAX_NODES),
    })
}

pub(crate) fn track_code_at_path(
    state: SharedState,
    path: TrackingAtPathParam,
    query: TrackingQuery,
) -> Result<TrackingResult, TrackingError> {
    let now = Instant::now();
    let TrackingQuery {
        start,
        end,
        before,
        flags,
    } = query;
    let TrackingAtPathParam {
        user,
        name,
        commit,
        path,
    } = path;
    let repo_specifier = hyper_ast_cvs_git::git::Forge::Github.repo(user, name);
    let repository = state
        .repositories
        .write()
        .unwrap()
        .get_config(repo_specifier)
        .ok_or_else(|| TrackingError {
            compute_time: now.elapsed().as_secs_f64(),
            commits_processed: 0,
            node_processed: 0,
            message: "missing config for repository".to_string(),
        })?;
    let mut repository = repository.fetch();
    log::warn!("done cloning {}", repository.spec);
    // let mut get_mut = state.write().unwrap();
    // let state = get_mut.deref_mut();
    let mut commit = commit.clone();
    let mut node_processed = 0;
    let mut commits_processed = 1;
    let mut path: Vec<_> = path.split("/").filter_map(|x| x.parse().ok()).collect();
    let mut source = None;
    while node_processed < MAX_NODES {
        commits_processed += 1;
        let commits = state
            .repositories
            .write()
            .unwrap()
            .pre_process_with_limit(&mut repository, "", &commit, 2)
            .map_err(|e| TrackingError {
                compute_time: now.elapsed().as_secs_f64(),
                commits_processed: 0,
                node_processed: 0,
                message: e.to_string(),
            })?;
        log::warn!("done construction of {commits:?} in {}", repository.spec);
        let src_oid = commits[0];
        let dst_oid = if let Some(before) = &before {
            let commits = state
                .repositories
                .write()
                .unwrap()
                .pre_process_with_limit(&mut repository, "", before, 2)
                .map_err(|e| TrackingError {
                    compute_time: now.elapsed().as_secs_f64(),
                    commits_processed: 0,
                    node_processed: 0,
                    message: e.to_string(),
                })?;
            commits[0]
        } else {
            commits[1]
        };
        match aux2(state.clone(), &repository, src_oid, dst_oid, &path, &flags) {
            MappingResult::Direct { src: aaa, matches } => {
                let aaa = aaa.globalize(repository.spec, commit);
                let (src, intermediary) = if let Some(src) = source {
                    (src, Some(aaa))
                } else {
                    (aaa, None)
                };
                return Ok(TrackingResult {
                    compute_time: now.elapsed().as_secs_f64(),
                    commits_processed,
                    src,
                    intermediary,
                    fallback: None,
                    matched: matches,
                });
            }
            MappingResult::Missing { src: aaa, fallback } => {
                let aaa = aaa.globalize(repository.spec, commit);
                let (src, intermediary) = if let Some(src) = source {
                    (src, Some(aaa))
                } else {
                    (aaa, None)
                };
                return Ok(TrackingResult {
                    compute_time: now.elapsed().as_secs_f64(),
                    commits_processed,
                    src,
                    intermediary,
                    fallback: Some(fallback),
                    matched: vec![],
                });
            }
            MappingResult::Error(err) => Err(TrackingError {
                compute_time: now.elapsed().as_secs_f64(),
                commits_processed,
                node_processed,
                message: err,
            })?,
            MappingResult::Skipped { nodes, src, next } => {
                // TODO handle cases where there is no more commits
                if before.is_some() {
                    let aaa = src.globalize(repository.spec, commit);
                    let (src, intermediary) = if let Some(src) = source {
                        (src, Some(aaa))
                    } else {
                        (aaa, None)
                    };
                    return Ok(TrackingResult {
                        compute_time: now.elapsed().as_secs_f64(),
                        commits_processed,
                        src,
                        intermediary,
                        fallback: None,
                        matched: next,
                    });
                }
                node_processed += nodes;
                dbg!(src_oid, dst_oid);
                if source.is_none() {
                    source = Some(src.globalize(repository.spec.clone(), commit));
                }
                // commit = dst_oid.to_string();

                if next.len() > 1 {
                    log::error!("multiple matches")
                }
                if next.is_empty() {
                    unreachable!()
                } else {
                    let next = &next[0];
                    dbg!(next);
                    path = next.path.clone();
                    commit = next.commit.clone();
                }
            }
        }
    }
    Err(TrackingError {
        compute_time: now.elapsed().as_secs_f64(),
        commits_processed,
        node_processed,
        message: format!("reached max number of diffed nodes: (ie. {})", MAX_NODES),
    })
}

/// track in past for now
pub(crate) fn track_code_at_path_with_changes(
    state: SharedState,
    path: TrackingAtPathParam,
    query: TrackingQuery,
) -> Result<TrackingResultWithChanges, TrackingError> {
    let now = Instant::now();
    let TrackingQuery {
        start: _,
        end: _,
        before,
        flags,
    } = query;
    let TrackingAtPathParam {
        user,
        name,
        commit,
        path,
    } = path;
    let repo_spec = hyper_ast_cvs_git::git::Forge::Github.repo(user, name);
    let configs = state.clone();
    let repo_handle = state
        .repositories
        .write()
        .unwrap()
        .get_config(repo_spec).ok_or_else(|| TrackingError {
        compute_time: now.elapsed().as_secs_f64(),
        commits_processed: 0,
        node_processed: 0,
        message: "missing config for repository".to_string(),
    })?;
    let mut repository = repo_handle.fetch();
    log::warn!("done cloning {}", repository.spec);
    let mut ori_oid = None;
    let mut commit = commit.clone();
    let mut node_processed = 0;
    let mut commits_processed = 1;
    let mut path: Vec<_> = path.split("/").filter_map(|x| x.parse().ok()).collect();
    let mut source = None;
    while node_processed < MAX_NODES {
        commits_processed += 1;
        let commits = state
            .repositories
            .write()
            .unwrap()
            .pre_process_with_limit(&mut repository, "", &commit,2)
            .map_err(|e| TrackingError {
                compute_time: now.elapsed().as_secs_f64(),
                commits_processed: 0,
                node_processed: 0,
                message: e.to_string(),
            })?;
        log::warn!(
            "done construction of {commits:?} in {}",
            repository.spec.user
        );
        let src_oid = commits[0];
        if ori_oid.is_none() {
            ori_oid = Some(src_oid);
        }
        let Some(&dst_oid) = commits.get(1) else {
            return Err(TrackingError {
                compute_time: now.elapsed().as_secs_f64(),
                commits_processed,
                node_processed,
                message: "this commit has no parent".into(),
            });
        };
        match aux2(state.clone(), &repository, src_oid, dst_oid, &path, &flags) {
            MappingResult::Direct { src: aaa, matches } => {
                let changes =
                    changes::added_deleted(state, &repository, dst_oid, ori_oid.unwrap())
                        .map_err(|err| TrackingError {
                            compute_time: now.elapsed().as_secs_f64(),
                            commits_processed,
                            node_processed,
                            message: err,
                        })?;
                let aaa = aaa.globalize(repository.spec, commit);
                let (src, intermediary) = if let Some(src) = source {
                    (src, Some(aaa))
                } else {
                    (aaa, None)
                };
                let tracking_result = TrackingResult {
                    compute_time: now.elapsed().as_secs_f64(),
                    commits_processed,
                    src,
                    intermediary,
                    fallback: None,
                    matched: matches,
                };
                return Ok(tracking_result.with_changes(changes));
            }
            MappingResult::Missing { src, fallback } => {
                let changes =
                    changes::added_deleted(state, &repository, dst_oid, ori_oid.unwrap())
                        .map_err(|err| TrackingError {
                            compute_time: now.elapsed().as_secs_f64(),
                            commits_processed,
                            node_processed,
                            message: err,
                        })?;
                let aaa = src.globalize(repository.spec, commit);
                let (src, intermediary) = if let Some(src) = source {
                    (src, Some(aaa))
                } else {
                    (aaa, None)
                };
                let tracking_result = TrackingResult {
                    compute_time: now.elapsed().as_secs_f64(),
                    commits_processed,
                    src,
                    intermediary,
                    fallback: Some(fallback),
                    matched: vec![],
                };
                return Ok(tracking_result.with_changes(changes));
            }
            MappingResult::Error(err) => Err(TrackingError {
                compute_time: now.elapsed().as_secs_f64(),
                commits_processed,
                node_processed,
                message: err,
            })?,
            MappingResult::Skipped { nodes, src, next } => {
                dbg!(nodes);
                node_processed += nodes;
                dbg!(src_oid, dst_oid);
                if commits.len() < 3 {
                    // NOTE there is no parent commit to dst_commit, thus we should stop now
                    let changes =
                        changes::added_deleted(state, &repository, dst_oid, ori_oid.unwrap())
                            .map_err(|err| TrackingError {
                                compute_time: now.elapsed().as_secs_f64(),
                                commits_processed,
                                node_processed,
                                message: err,
                            })?;
                    let aaa = src.globalize(repository.spec, commit);
                    let (src, intermediary) = if let Some(src) = source {
                        (src, Some(aaa))
                    } else {
                        (aaa, None)
                    };
                    let tracking_result = TrackingResult {
                        compute_time: now.elapsed().as_secs_f64(),
                        commits_processed,
                        src,
                        intermediary,
                        fallback: None,
                        matched: next,
                    };
                    return Ok(tracking_result.with_changes(changes));
                }
                if source.is_none() {
                    source = Some(src.globalize(repository.spec.clone(), commit));
                }
                // commit = dst_oid.to_string();

                if next.len() > 1 {
                    log::error!("multiple matches")
                }
                if next.is_empty() {
                    unreachable!()
                } else {
                    let next = &next[0]; // TODO stop on branching ?
                    dbg!(next);
                    path = next.path.clone();
                    commit = next.commit.clone();
                }
            }
        }
    }
    Err(TrackingError {
        compute_time: now.elapsed().as_secs_f64(),
        commits_processed,
        node_processed,
        message: format!("reached max number of diffed nodes: (ie. {})", MAX_NODES),
    })
}

enum MappingResult {
    Direct {
        src: LocalPieceOfCode,
        matches: Vec<PieceOfCode>,
    },
    Missing {
        src: LocalPieceOfCode,
        fallback: PieceOfCode,
    },
    Error(String),
    Skipped {
        nodes: usize,
        src: LocalPieceOfCode,
        next: Vec<PieceOfCode>,
    },
}

struct LocalPieceOfCode {
    file: String,
    start: usize,
    end: usize,
    path: Vec<usize>,
    path_ids: Vec<NodeIdentifier>,
}

impl LocalPieceOfCode {
    pub(crate) fn globalize(self, spec: Repo, commit: impl ToString) -> PieceOfCode {
        let LocalPieceOfCode {
            file,
            start,
            end,
            path,
            path_ids,
        } = self;
        let commit = commit.to_string();
        PieceOfCode {
            user: spec.user,
            name: spec.name,
            commit,
            path,
            path_ids,
            file,
            start,
            end,
        }
    }
}

fn aux(
    state: std::sync::Arc<crate::AppState>,
    repo_handle: &impl ConfiguredRepoTrait<Config = hyper_ast_cvs_git::processing::ParametrizedCommitProcessorHandle>,
    src_oid: hyper_ast_cvs_git::git::Oid,
    dst_oid: hyper_ast_cvs_git::git::Oid,
    file: &String,
    start: Option<usize>,
    end: Option<usize>,
    flags: &Flags,
) -> MappingResult {
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories
        .get_commit(repo_handle.config(),&src_oid)
        .unwrap();
    let src_tr = commit_src.ast_root;
    let commit_dst = repositories
    .get_commit(&repo_handle.config(),&dst_oid)
        .unwrap();
    let dst_tr = commit_dst.ast_root;
    let stores = &repositories.processor.main_stores;
    let node_store = &stores.node_store;

    // let size = node_store.resolve(src_tr).size();
    log::error!("searching for {file}");
    let file_node =
        child_at_path_tracked(&repositories.processor.main_stores, src_tr, file.split("/"));

    let Some((file_node, offsets_to_file)) = file_node else {
        return MappingResult::Error("not found".into());
    };

    dbg!(&offsets_to_file);
    let mut path_to_target = vec![];
    let (node, offsets_in_file) = resolve_range(file_node, start.unwrap_or(0), end, stores);
    path_to_target.extend(offsets_to_file.iter().map(|x| *x as u16));
    dbg!(&node);
    dbg!(&offsets_in_file);
    let aaa = node_store.resolve(file_node);
    dbg!(aaa.try_get_bytes_len(0));
    path_to_target.extend(offsets_in_file.iter().map(|x| *x as u16));

    let (start, end, target_node) =
        compute_range(file_node, &mut offsets_in_file.into_iter(), stores);
    dbg!(start, end);
    dbg!(&target_node);
    let no_spaces_path_to_target = if false {
        // TODO use this version
        use hyper_ast::position;
        use position::offsets;
        let src = offsets::OffsetsRef::from(path_to_target.as_slice());
        let src = src.with_root(src_tr);
        let src = src.with_store(stores);
        let no_spaces_path_to_target: offsets::Offsets<_, position::tags::TopDownNoSpace> =
            src.compute_no_spaces::<_, offsets::Offsets<_, _>>();
        no_spaces_path_to_target.into()
    } else {
        let (_, _, no_spaces_path_to_target) =
            compute_position_with_no_spaces(src_tr, &mut path_to_target.iter().map(|x| *x), stores);
        no_spaces_path_to_target
    };
    let dst_oid = dst_oid; // WARN not sure what I was doing there commit_dst.clone();
    aux_aux(
        repo_handle,
        src_tr,
        dst_tr,
        path_to_target,
        no_spaces_path_to_target,
        flags,
        start,
        end,
        &state.partial_decomps,
        &state.mappings_alone,
        repositories,
        dst_oid,
        target_node,
    )
}

fn aux2(
    state: std::sync::Arc<crate::AppState>,
    repo_handle: &impl ConfiguredRepoTrait<Config = hyper_ast_cvs_git::processing::ParametrizedCommitProcessorHandle>,
    src_oid: hyper_ast_cvs_git::git::Oid,
    dst_oid: hyper_ast_cvs_git::git::Oid,
    path: &[usize],
    flags: &Flags,
) -> MappingResult {
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories
        .get_commit(repo_handle.config(),&src_oid)
        .unwrap();
    let src_tr = commit_src.ast_root;
    let commit_dst = repositories
        .get_commit(repo_handle.config(),&dst_oid)
        .unwrap();
    let dst_tr = commit_dst.ast_root;
    let stores = &repositories.processor.main_stores;
    let node_store = &stores.node_store;

    let path_to_target: Vec<_> = path.iter().map(|x| *x as u16).collect();
    dbg!(&path_to_target);
    let (pos, target_node, no_spaces_path_to_target) = if false {
        // NOTE trying stuff
        // TODO use this version
        use hyper_ast::position;
        use position::file_and_offset;
        use position::offsets;
        use position::offsets_and_nodes;
        let src = offsets::OffsetsRef::from(path_to_target.as_slice());
        let src = src.with_root(src_tr);
        let src = src.with_store(stores);
        // let no_spaces_path_to_target: offsets::Offsets<_, position::tags::TopDownNoSpace> =
        //     src.compute_no_spaces::<_, offsets::Offsets<_, _>>();
        let (pos, path): (
            position::Position,
            offsets_and_nodes::SolvedStructuralPosition<_, _, position::tags::TopDownNoSpace>,
        ) = src.compute_no_spaces::<_, position::CompoundPositionPreparer<
            position::Position,
            offsets_and_nodes::StructuralPosition<_, _, position::tags::TopDownNoSpace>,
        >>();
        // no_spaces_path_to_target.into()
        let (node, path) = path.into();
        (pos, node, path)
    } else {
        compute_position_with_no_spaces(src_tr, &mut path_to_target.iter().map(|x| *x), stores)
    };
    dbg!(&path_to_target, &no_spaces_path_to_target);
    let range = pos.range();
    let dst_oid = dst_oid; // WARN not sure what I was doing there commit_dst.clone();
    aux_aux(
        repo_handle,
        src_tr,
        dst_tr,
        path_to_target,
        no_spaces_path_to_target,
        flags,
        range.start,
        range.end,
        &state.partial_decomps,
        &state.mappings_alone,
        repositories,
        dst_oid,
        target_node,
    )
}

fn aux_aux(
    repo_handle: &impl ConfiguredRepoTrait,
    src_tr: NodeIdentifier,
    dst_tr: NodeIdentifier,
    path_to_target: Vec<u16>,
    no_spaces_path_to_target: Vec<u16>,
    flags: &Flags,
    start: usize,
    end: usize,
    partial_decomps: &PartialDecompCache,
    mappings_alone: &MappingAloneCache,
    repositories: std::sync::RwLockReadGuard<multi_preprocessed::PreProcessedRepositories>,
    dst_oid: hyper_ast_cvs_git::git::Oid,
    target_node: NodeIdentifier,
) -> MappingResult {
    let with_spaces_stores = &repositories.processor.main_stores;
    let stores = &no_space::as_nospaces(with_spaces_stores);
    let node_store = &stores.node_store;
    // NOTE: persists mappings, could also easily persist diffs,
    // but some compression on mappins could help
    // such as, not storing the decompression arenas
    // or encoding mappings more efficiently considering that most slices could simply by represented as ranges (ie. mapped identical subtrees)
    // let mapper = lazy_mapping(repos, &mut state.mappings, src_tr, dst_tr);

    dbg!(src_tr, dst_tr);
    if src_tr == dst_tr {
        let src_size = stores.node_store.resolve(src_tr).size();
        let dst_size = stores.node_store.resolve(dst_tr).size();
        let nodes = src_size + dst_size;
        let (pos, path_ids) = compute_position_and_nodes(
            dst_tr,
            &mut path_to_target.iter().copied(),
            with_spaces_stores,
        );
        dbg!();
        let range = pos.range();
        let matches = vec![PieceOfCode {
            user: repo_handle.spec().user.clone(),
            name: repo_handle.spec().name.clone(),
            commit: dst_oid.to_string(),
            file: pos.file().to_str().unwrap().to_string(),
            start: range.start,
            end: range.end,
            path: path_to_target.iter().map(|x| *x as usize).collect(),
            path_ids: path_ids.clone(),
        }];
        let src = LocalPieceOfCode {
            file: pos.file().to_string_lossy().to_string(),
            start,
            end,
            path: path_to_target.iter().map(|x| *x as usize).collect(),
            path_ids,
        };
        if flags.some() {
            return MappingResult::Skipped {
                nodes,
                src,
                next: matches,
            };
        } else {
            return MappingResult::Direct { src, matches };
        }
    }
    let pair = get_pair_simp(partial_decomps, stores, &src_tr, &dst_tr);

    if flags.some() {
        dbg!();

        let mapped = {
            let mappings_cache = mappings_alone;
            let hyperast = stores;
            let src = &src_tr;
            let dst = &dst_tr;
            let (src_arena, dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
            matching::top_down(hyperast, src_arena, dst_arena)
        };
        let (mapper_src_arena, mapper_dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
        let mapper_mappings = &mapped;
        let mut curr = mapper_src_arena.root();
        let mut path = &no_spaces_path_to_target[..];
        let flags: EnumSet<_> = flags.into();
        loop {
            dbg!(path);
            let dsts = mapper_mappings.get_dsts(&curr);
            let curr_flags = FlagsE::Upd | FlagsE::Child | FlagsE::SimChild; //  | FlagsE::ExactChild
            let parent_flags = curr_flags | FlagsE::Parent | FlagsE::SimParent; //  | FlagsE::ExactParent
            if dsts.is_empty() {
                // continue through path_to_target
                dbg!(curr);
            } else if path.len() == 0 {
                // need to check curr node flags
                if flags.is_subset(curr_flags) {
                    // only trigger on curr and children changed

                    let src_size = stores.node_store.resolve(src_tr).size();
                    let dst_size = stores.node_store.resolve(dst_tr).size();
                    let nodes = src_size + dst_size;
                    let nodes = 10000;

                    return MappingResult::Skipped {
                        nodes,
                        src: {
                            let (pos, path_ids) = compute_position_and_nodes(
                                src_tr,
                                &mut path_to_target.iter().copied(),
                                with_spaces_stores,
                            );

                            LocalPieceOfCode {
                                file: pos.file().to_string_lossy().to_string(),
                                start,
                                end,
                                path: path_to_target.iter().map(|x| *x as usize).collect(),
                                path_ids,
                            }
                        },
                        next: dsts
                            .iter()
                            .map(|x| {
                                let path_dst = mapper_dst_arena.path(&mapper_dst_arena.root(), x);
                                let (path_dst,) = path_with_spaces(
                                    dst_tr,
                                    &mut path_dst.iter().copied(),
                                    &repositories.processor.main_stores,
                                );
                                let (pos, path_ids) = compute_position_and_nodes(
                                    dst_tr,
                                    &mut path_dst.iter().copied(),
                                    with_spaces_stores,
                                );
                                let range = pos.range();
                                PieceOfCode {
                                    user: repo_handle.spec().user.clone(),
                                    name: repo_handle.spec().name.clone(),
                                    commit: dst_oid.to_string(),
                                    file: pos.file().to_str().unwrap().to_string(),
                                    start: range.start,
                                    end: range.end,
                                    path: path_dst.iter().map(|x| *x as usize).collect(),
                                    path_ids,
                                }
                            })
                            .collect(),
                    };
                }
                // also the type of src and dsts
                // also check it file path changed
                // can we test if parent changed ? at least we can ckeck some attributes
            } else if path.len() == 1 {
                // need to check parent node flags
                if flags.is_subset(parent_flags) {
                    // only trigger on parent, curr and children changed
                    let src_size = stores.node_store.resolve(src_tr).size();
                    let dst_size = stores.node_store.resolve(dst_tr).size();
                    let nodes = src_size + dst_size;
                    let nodes = 10000;
                    return MappingResult::Skipped {
                        nodes,
                        src: {
                            let (pos, path_ids) = compute_position_and_nodes(
                                src_tr,
                                &mut path_to_target.iter().copied(),
                                with_spaces_stores,
                            );

                            LocalPieceOfCode {
                                file: pos.file().to_string_lossy().to_string(),
                                start,
                                end,
                                path: path_to_target.iter().map(|x| *x as usize).collect(),
                                path_ids,
                            }
                        },
                        next: dsts
                            .iter()
                            .map(|x| {
                                let mut path_dst =
                                    mapper_dst_arena.path(&mapper_dst_arena.root(), x);
                                path_dst.extend(path); // WARN with similarity it might not be possible to simply concat path...
                                let (path_dst,) = path_with_spaces(
                                    dst_tr,
                                    &mut path_dst.iter().copied(),
                                    &repositories.processor.main_stores,
                                );
                                let (pos, path_ids) = compute_position_and_nodes(
                                    dst_tr,
                                    &mut path_dst.iter().copied(),
                                    with_spaces_stores,
                                );
                                let range = pos.range();
                                PieceOfCode {
                                    user: repo_handle.spec().user.clone(),
                                    name: repo_handle.spec().name.clone(),
                                    commit: dst_oid.to_string(),
                                    file: pos.file().to_str().unwrap().to_string(),
                                    start: range.start,
                                    end: range.end,
                                    path: path_dst.iter().map(|x| *x as usize).collect(),
                                    path_ids,
                                }
                            })
                            .collect(),
                    };
                }
                // also the type of src and dsts
                // also check if file path changed
                // can we test if parent changed ? at least we can ckeck some attributes
            } else {
                // need to check flags, the type of src and dsts
                if flags.is_subset(parent_flags) {
                    // only trigger on parent, curr and children changed
                    let src_size = stores.node_store.resolve(src_tr).size();
                    let dst_size = stores.node_store.resolve(dst_tr).size();
                    let nodes = src_size + dst_size;
                    let nodes = 10000;
                    return MappingResult::Skipped {
                        nodes,
                        src: {
                            let (pos, path_ids) = compute_position_and_nodes(
                                src_tr,
                                &mut path_to_target.iter().copied(),
                                with_spaces_stores,
                            );

                            LocalPieceOfCode {
                                file: pos.file().to_string_lossy().to_string(),
                                start,
                                end,
                                path: path_to_target.iter().map(|x| *x as usize).collect(),
                                path_ids,
                            }
                        },
                        next: dsts
                            .iter()
                            .map(|x| {
                                let mut path_dst =
                                    mapper_dst_arena.path(&mapper_dst_arena.root(), x);
                                path_dst.extend(path); // WARN with similarity it might not be possible to simply concat path...
                                let (path_dst,) = path_with_spaces(
                                    dst_tr,
                                    &mut path_dst.iter().copied(),
                                    &repositories.processor.main_stores,
                                );
                                let (pos, path_ids) = compute_position_and_nodes(
                                    dst_tr,
                                    &mut path_dst.iter().copied(),
                                    with_spaces_stores,
                                );
                                let range = pos.range();
                                PieceOfCode {
                                    user: repo_handle.spec().user.clone(),
                                    name: repo_handle.spec().name.clone(),
                                    commit: dst_oid.to_string(),
                                    file: pos.file().to_str().unwrap().to_string(),
                                    start: range.start,
                                    end: range.end,
                                    path: path_dst.iter().map(|x| *x as usize).collect(),
                                    path_ids,
                                }
                            })
                            .collect(),
                    };
                }
                // also check if file path changed
                // can we test if parent changed ? at least we can ckeck some attributes
            }

            let Some(i) = path.get(0) else {
                break;
            };
            path = &path[1..];
            let cs = mapper_src_arena.decompress_children(node_store, &curr);
            if cs.is_empty() {
                break;
            }
            curr = cs[*i as usize];
        }
    }

    let mapped = {
        let mappings_cache = mappings_alone;
        use hyper_diff::matchers::mapping_store::MappingStore;
        use hyper_diff::matchers::mapping_store::VecStore;
        let hyperast = stores;
        use hyper_diff::matchers::Mapping;

        dbg!();
        match mappings_cache.entry((src_tr, dst_tr)) {
            dashmap::mapref::entry::Entry::Occupied(entry) => {
                entry.into_ref().downgrade()
            }
            dashmap::mapref::entry::Entry::Vacant(entry) => {
                let mappings = VecStore::default();
                let (src_arena, dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
                dbg!(src_arena.len());
                dbg!(dst_arena.len());
                let src_size = stores.node_store.resolve(src_tr).size();
                let dst_size = stores.node_store.resolve(dst_tr).size();
                dbg!(src_size);
                dbg!(dst_size);
                let mut mapper = Mapper {
                    hyperast,
                    mapping: Mapping {
                        src_arena,
                        dst_arena,
                        mappings,
                    },
                };
                dbg!();
                dbg!(mapper.mapping.src_arena.len());
                dbg!(mapper.mapping.dst_arena.len());
                mapper.mapping.mappings.topit(
                    mapper.mapping.src_arena.len(),
                    mapper.mapping.dst_arena.len(),
                );
                dbg!();

                let vec_store = matching::full2(hyperast, mapper);

                dbg!();
                entry
                    .insert((crate::MappingStage::Bottomup, vec_store))
                    .downgrade()
            }
        }
    };
    let (mapper_src_arena, mapper_dst_arena) = (pair.0.get_mut(), pair.1.get_mut());
    let mapper_mappings = &mapped.1;
    let root = mapper_src_arena.root();
    let mapping_target =
        mapper_src_arena.child_decompressed(node_store, &root, &no_spaces_path_to_target);

    let mut matches = vec![];
    if let Some(mapped) = mapper_mappings.get_dst(&mapping_target) {
        let mapped = mapper_dst_arena.decompress_to(node_store, &mapped);
        let path = mapper_dst_arena.path(&mapper_dst_arena.root(), &mapped);
        let mut path_ids = vec![mapper_dst_arena.original(&mapped)];
        mapper_dst_arena
            .parents(mapped)
            .map(|i| mapper_dst_arena.original(&i))
            .collect_into(&mut path_ids);
        path_ids.pop();
        assert_eq!(path.len(), path_ids.len());
        let (path,) = path_with_spaces(
            dst_tr,
            &mut path.iter().copied(),
            &repositories.processor.main_stores,
        );
        let (pos, mapped_node) =
            compute_position(dst_tr, &mut path.iter().copied(), with_spaces_stores);
        dbg!(&pos);
        dbg!(&mapped_node);
        let mut flagged = false;
        let mut triggered = false;
        if flags.exact_child {
            flagged = true;
            dbg!();
            triggered |= target_node != mapped_node;
        }
        if flags.child || flags.sim_child {
            flagged = true;
            dbg!();

            let target_node = stores.node_store.resolve(target_node);
            let mapped_node = stores.node_store.resolve(mapped_node);
            if flags.sim_child {
                triggered |= target_node.hash(&types::HashKind::structural())
                    != mapped_node.hash(&types::HashKind::structural());
            } else {
                triggered |= target_node.hash(&types::HashKind::label())
                    != mapped_node.hash(&types::HashKind::label());
            }
        }
        if flags.upd {
            flagged = true;
            dbg!();
            // TODO need role name
            // let target_ident = child_by_type(stores, target_node, &Type::Identifier);
            // let mapped_ident = child_by_type(stores, mapped_node, &Type::Identifier);
            // if let (Some(target_ident), Some(mapped_ident)) = (target_ident, mapped_ident) {
            //     let target_node = stores.node_store.resolve(target_ident.0);
            //     let target_ident = target_node.try_get_label();
            //     let mapped_node = stores.node_store.resolve(mapped_ident.0);
            //     let mapped_ident = mapped_node.try_get_label();
            //     triggered |= target_ident != mapped_ident;
            // }
        }
        if flags.parent {
            flagged = true;
            dbg!();

            let target_parent = mapper_src_arena.parent(&mapping_target);
            let target_parent = target_parent.map(|x| mapper_src_arena.original(&x));
            let mapped_parent = mapper_dst_arena.parent(&mapped);
            let mapped_parent = mapped_parent.map(|x| mapper_dst_arena.original(&x));
            triggered |= target_parent != mapped_parent;
        }
        // if flags.meth {
        //     flagged = true;
        //     dbg!();
        // }
        // if flags.typ {
        //     flagged = true;
        //     dbg!();
        // }
        // if flags.top {
        //     flagged = true;
        //     dbg!();
        // }
        // if flags.file {
        //     flagged = true;
        //     dbg!();
        // }
        // if flags.pack {
        //     flagged = true;
        //     dbg!();
        // }
        // TODO add flags for artefacts (tests, prod code, build, lang, misc)
        // TODO add flags for similarity comps
        let range = pos.range();
        matches.push(PieceOfCode {
            user: repo_handle.spec().user.clone(),
            name: repo_handle.spec().name.clone(),
            commit: dst_oid.to_string(),
            file: pos.file().to_str().unwrap().to_string(),
            start: range.start,
            end: range.end,
            path: path.iter().map(|x| *x as usize).collect(),
            path_ids: path_ids.clone(),
        });
        if flagged && !triggered {
            use hyper_ast::types::WithStats;
            let src_size = stores.node_store.resolve(src_tr).size();
            let dst_size = stores.node_store.resolve(dst_tr).size();
            let nodes = src_size + dst_size;
            return MappingResult::Skipped {
                nodes,
                src: {
                    let (pos, path_ids) = compute_position_and_nodes(
                        src_tr,
                        &mut path_to_target.iter().copied(),
                        with_spaces_stores,
                    );

                    LocalPieceOfCode {
                        file: pos.file().to_string_lossy().to_string(),
                        start,
                        end,
                        path: path_to_target.iter().map(|x| *x as usize).collect(),
                        path_ids,
                    }
                },
                next: matches,
            };
        }
        let path = path_to_target.iter().map(|x| *x as usize).collect();
        let (target_pos, target_path_ids) = compute_position_and_nodes(
            src_tr,
            &mut path_to_target.iter().copied(),
            with_spaces_stores,
        );
        return MappingResult::Direct {
            src: LocalPieceOfCode {
                file: target_pos.file().to_string_lossy().to_string(),
                start,
                end,
                path,
                path_ids: target_path_ids,
            },
            matches,
        };
    }

    for parent_target in mapper_src_arena.parents(mapping_target) {
        if let Some(mapped_parent) = mapper_mappings.get_dst(&parent_target) {
            let mapped_parent = mapper_dst_arena.decompress_to(node_store, &mapped_parent);
            let path = mapper_dst_arena.path(&mapper_dst_arena.root(), &mapped_parent);
            let mut path_ids = vec![mapper_dst_arena.original(&mapped_parent)];
            mapper_dst_arena
                .parents(mapped_parent)
                .map(|i| mapper_dst_arena.original(&i))
                .collect_into(&mut path_ids);
            path_ids.pop();
            assert_eq!(path.len(), path_ids.len());
            let (path,) = path_with_spaces(
                dst_tr,
                &mut path.iter().copied(),
                &repositories.processor.main_stores,
            );
            let (pos, mapped_node) =
                compute_position(dst_tr, &mut path.iter().copied(), with_spaces_stores);
            dbg!(&pos);
            dbg!(&mapped_node);
            let range = pos.range();
            let fallback = PieceOfCode {
                user: repo_handle.spec().user.clone(),
                name: repo_handle.spec().name.clone(),
                commit: dst_oid.to_string(),
                file: pos.file().to_str().unwrap().to_string(),
                start: range.start,
                end: range.end,
                path: path.iter().map(|x| *x as usize).collect(),
                path_ids: path_ids.clone(),
            };

            let src = {
                let path = path_to_target.iter().map(|x| *x as usize).collect();
                let (target_pos, target_path_ids) = compute_position_and_nodes(
                    src_tr,
                    &mut path_to_target.iter().copied(),
                    with_spaces_stores,
                );
                LocalPieceOfCode {
                    file: target_pos.file().to_string_lossy().to_string(),
                    start,
                    end,
                    path,
                    path_ids: target_path_ids,
                }
            };
            return MappingResult::Missing { src, fallback };
        };
    }
    let path = path_to_target.iter().map(|x| *x as usize).collect();
    let (target_pos, target_path_ids) = compute_position_and_nodes(
        src_tr,
        &mut path_to_target.iter().copied(),
        with_spaces_stores,
    );
    // TODO what should be done if there is no match ?
    MappingResult::Direct {
        src: LocalPieceOfCode {
            file: target_pos.file().to_string_lossy().to_string(),
            start,
            end,
            path,
            path_ids: target_path_ids,
        },
        matches,
    }
}

// fn diff<'a>(
//     repositories: &'a multi_preprocessed::PreProcessedRepositories,
//     mappings: &'a mut crate::MappingCache,
//     src_tr: NodeIdentifier,
//     dst_tr: NodeIdentifier,
// ) -> &'a hyper_diff::matchers::Mapping<
//     hyper_diff::decompressed_tree_store::CompletePostOrder<
//         hyper_ast::store::nodes::legion::HashedNodeRef<'a>,
//         u32,
//     >,
//     hyper_diff::decompressed_tree_store::CompletePostOrder<
//         hyper_ast::store::nodes::legion::HashedNodeRef<'a>,
//         u32,
//     >,
//     hyper_diff::matchers::mapping_store::VecStore<u32>,
// > {
//     use hyper_diff::decompressed_tree_store::CompletePostOrder;
//     let mapped = mappings.entry((src_tr, dst_tr)).or_insert_with(|| {
//         hyper_diff::algorithms::gumtree_lazy::diff(
//             &repositories.processor.main_stores,
//             &src_tr,
//             &dst_tr,
//         )
//         .mapper
//         .persist()
//     });
//     unsafe { Mapper::<_,CompletePostOrder<_,_>,CompletePostOrder<_,_>,_>::unpersist(&repositories.processor.main_stores, &*mapped) }
// }

// WARN lazy subtrees are not complete
fn lazy_mapping<'a>(
    repositories: &'a multi_preprocessed::PreProcessedRepositories,
    mappings: &'a crate::MappingCache,
    src_tr: NodeIdentifier,
    dst_tr: NodeIdentifier,
) -> dashmap::mapref::one::RefMut<
    'a,
    (NodeIdentifier, NodeIdentifier),
    hyper_diff::matchers::Mapping<
        hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<
            hyper_ast::store::nodes::legion::HashedNodeRef<'a, NodeIdentifier>,
            u32,
        >,
        hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<
            hyper_ast::store::nodes::legion::HashedNodeRef<'a, NodeIdentifier>,
            u32,
        >,
        hyper_diff::matchers::mapping_store::VecStore<u32>,
    >,
> {
    use hyper_ast::types::HyperAST;
    use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    use hyper_diff::matchers::heuristic::gt::{
        lazy2_greedy_bottom_up_matcher::GreedyBottomUpMatcher,
        lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher,
    };
    use hyper_diff::matchers::mapping_store::DefaultMultiMappingStore;
    use hyper_diff::matchers::mapping_store::MappingStore;
    use hyper_diff::matchers::mapping_store::VecStore;
    let mapped = mappings.entry((src_tr, dst_tr)).or_insert_with(|| {
        let hyperast = &repositories.processor.main_stores;
        let src = &src_tr;
        let dst = &dst_tr;
        let now = Instant::now();
        let mapper: Mapper<_, LazyPostOrder<_, u32>, LazyPostOrder<_, u32>, VecStore<_>> =
            hyperast.decompress_pair(src, dst).into();
        let subtree_prepare_t = now.elapsed().as_secs_f64();
        let now = Instant::now();
        let mapper =
            LazyGreedySubtreeMatcher::<_, _, _, _>::match_it::<DefaultMultiMappingStore<_>>(mapper);
        let subtree_matcher_t = now.elapsed().as_secs_f64();
        let subtree_mappings_s = mapper.mappings().len();
        dbg!(&subtree_matcher_t, &subtree_mappings_s);
        let bottomup_prepare_t = 0.;
        let now = Instant::now();
        let mapper = GreedyBottomUpMatcher::<_, _, _, _, VecStore<_>>::match_it(mapper);
        dbg!(&now.elapsed().as_secs_f64());
        let bottomup_matcher_t = now.elapsed().as_secs_f64();
        let bottomup_mappings_s = mapper.mappings().len();
        dbg!(&bottomup_matcher_t, &bottomup_mappings_s);
        // let now = Instant::now();

        // NOTE could also have completed trees
        // let node_store = hyperast.node_store();
        // let mapper = mapper.map(
        //     |src_arena| CompletePostOrder::from(src_arena.complete(node_store)),
        //     |dst_arena| {
        //         let complete = CompletePostOrder::from(dst_arena.complete(node_store));
        //         SimpleBfsMapper::from(node_store, complete)
        //     },
        // );

        // NOTE we do not use edit scripts here
        // let prepare_gen_t = now.elapsed().as_secs_f64();
        // let now = Instant::now();
        // let actions = ScriptGenerator::compute_actions(mapper.hyperast, &mapper.mapping).ok();
        // let gen_t = now.elapsed().as_secs_f64();
        // dbg!(gen_t);
        // let mapper = mapper.map(|x| x, |dst_arena| dst_arena.back);
        Mapper::<_, LazyPostOrder<_, _>, LazyPostOrder<_, _>, _>::persist(mapper)
    });
    pub unsafe fn unpersist<'a>(
        _hyperast: &'a SimpleStores<TStore>,
        p: dashmap::mapref::one::RefMut<
            'a,
            (NodeIdentifier, NodeIdentifier),
            hyper_diff::matchers::Mapping<
                LazyPostOrder<
                    hyper_diff::decompressed_tree_store::PersistedNode<NodeIdentifier>,
                    u32,
                >,
                LazyPostOrder<
                    hyper_diff::decompressed_tree_store::PersistedNode<NodeIdentifier>,
                    u32,
                >,
                VecStore<u32>,
            >,
        >,
    ) -> dashmap::mapref::one::RefMut<
        'a,
        (NodeIdentifier, NodeIdentifier),
        hyper_diff::matchers::Mapping<
            LazyPostOrder<HashedNodeRef<'a, NodeIdentifier>, u32>,
            LazyPostOrder<HashedNodeRef<'a, NodeIdentifier>, u32>,
            VecStore<u32>,
        >,
    > {
        unsafe { std::mem::transmute(p) }
    }
    unsafe { unpersist(&repositories.processor.main_stores, mapped) }
}

struct RRR<'a>(
    dashmap::mapref::one::Ref<
        'a,
        (NodeIdentifier, NodeIdentifier),
        (
            crate::MappingStage,
            hyper_diff::matchers::mapping_store::VecStore<u32>,
        ),
    >,
);

mod my_dash {
    use std::{
        cell::UnsafeCell,
        collections::hash_map::RandomState,
        fmt::Debug,
        hash::{BuildHasher, Hash},
    };

    use dashmap::{DashMap, RwLockWriteGuard, SharedValue};
    use hashbrown::HashMap;

    // pub fn entries<'a, K: 'a + Eq + Hash, V: 'a, S: BuildHasher + Clone>(
    //     map: DashMap<K, V, S>,
    //     key1: K,
    //     key2: K,
    // ) -> Entry<'a, K, V, S> {
    //     let hash = map.hash_usize(&key1);

    //     let idx = map.determine_shard(hash);

    //     let shard: RwLockWriteGuard<HashMap<K, SharedValue<V>, S>> = unsafe {
    //         debug_assert!(idx < map.shards().len());

    //         map.shards().get_unchecked(idx).write()
    //     };

    //     #[repr(transparent)]
    //     struct MySharedValue<T> {
    //         value: UnsafeCell<T>,
    //     }

    //     impl<T> MySharedValue<T> {
    //         /// Get a mutable raw pointer to the underlying value
    //         fn as_ptr(&self) -> *mut T {
    //             self.value.get()
    //         }
    //     }
    //     // SAFETY: Sharded and UnsafeCell are transparent wrappers of V
    //     let shard: RwLockWriteGuard<HashMap<K, V, S>> = unsafe { std::mem::transmute(shard) };
    //     if let Some((kptr, vptr)) = shard.get_key_value(&key1) {
    //         unsafe {
    //             let kptr: *const K = kptr;
    //             // SAFETY: same memory layout because transparent and same fields
    //             let vptr: &MySharedValue<V> = std::mem::transmute(&vptr);
    //             let vptr: *mut V = vptr.as_ptr();
    //             Entry::Occupied(OccupiedEntry::new(shard, key1, (kptr, vptr)))
    //         }
    //     } else {
    //         unsafe {
    //             // SAFETY: same memory layout because transparent and same fields
    //             let shard: RwLockWriteGuard<HashMap<K, V, S>> = std::mem::transmute(shard);
    //             Entry::Vacant(VacantEntry::new(shard, key1))
    //         }
    //     }
    pub fn entries<'a, K: 'a + Eq + Hash, V: 'a, S: BuildHasher + Clone>(
        map: &'a DashMap<K, V, S>,
        key1: K,
        key2: K,
    ) -> Entry<'a, K, V, S> {
        assert!(key1 != key2, "keys should be different");
        let hash1 = map.hash_usize(&key1);
        let idx1 = map.determine_shard(hash1);
        let hash2 = map.hash_usize(&key2);
        let idx2 = map.determine_shard(hash2);

        if idx1 == idx2 {
            let shard = unsafe {
                debug_assert!(idx1 < map.shards().len());
                debug_assert!(idx2 < map.shards().len());
                map.shards().get_unchecked(idx1).write()
            };
            // SAFETY: Sharded and UnsafeCell are transparent wrappers of V
            let shard: RwLockWriteGuard<HashMap<K, V, S>> = unsafe { std::mem::transmute(shard) };
            let elem1 = shard
                .get_key_value(&key1)
                .map(|(kptr, vptr)| unsafe { as_ptr(kptr, vptr) });
            let elem2 = shard
                .get_key_value(&key2)
                .map(|(kptr, vptr)| unsafe { as_ptr(kptr, vptr) });
            Entry {
                shard1: shard,
                shard2: None,
                key1,
                key2,
                elem1,
                elem2,
            }
        } else {
            let (shard1, shard2) = unsafe {
                debug_assert!(idx1 < map.shards().len());
                debug_assert!(idx2 < map.shards().len());
                (
                    map.shards().get_unchecked(idx1).write(),
                    map.shards().get_unchecked(idx2).write(),
                )
            };
            let shard1: RwLockWriteGuard<HashMap<K, V, S>> = unsafe { std::mem::transmute(shard1) };
            let shard2: RwLockWriteGuard<HashMap<K, V, S>> = unsafe { std::mem::transmute(shard2) };
            let elem1 = shard1
                .get_key_value(&key1)
                .map(|(kptr, vptr)| unsafe { as_ptr(kptr, vptr) });
            let elem2 = shard2
                .get_key_value(&key2)
                .map(|(kptr, vptr)| unsafe { as_ptr(kptr, vptr) });
            Entry {
                shard1: shard1,
                shard2: Some(shard2),
                key1,
                key2,
                elem1,
                elem2,
            }
        }
    }

    unsafe fn as_ptr<'a, K: 'a + Eq + Hash, V: 'a>(kptr1: &K, vptr1: &V) -> (*const K, *mut V) {
        let kptr1: *const K = kptr1;
        // SAFETY: same memory layout because transparent and same fields
        let vptr1: &MySharedValue<V> = std::mem::transmute(&vptr1);
        let vptr1: *mut V = vptr1.as_ptr();
        (kptr1, vptr1)
    }

    pub(super) unsafe fn shard_as_ptr<'a, V: 'a>(vptr1: &SharedValue<V>) -> *mut V {
        // SAFETY: same memory layout because transparent and same fields
        let vptr1: &MySharedValue<V> = std::mem::transmute(&vptr1);
        let vptr1: *mut V = vptr1.as_ptr();
        vptr1
    }

    pub(super) unsafe fn shard_as_ptr2<'a, V: 'a>(vptr1: &V) -> *mut V {
        // SAFETY: same memory layout because transparent and same fields
        let vptr1: &MySharedValue<V> = std::mem::transmute(&vptr1);
        let vptr1: *mut V = vptr1.as_ptr();
        vptr1
    }

    #[repr(transparent)]
    struct MySharedValue<T> {
        value: UnsafeCell<T>,
    }

    impl<T> MySharedValue<T> {
        /// Get a mutable raw pointer to the underlying value
        fn as_ptr(&self) -> *mut T {
            self.value.get()
        }
    }
    pub struct Entry<'a, K, V, S = RandomState> {
        shard1: RwLockWriteGuard<'a, HashMap<K, V, S>>,
        shard2: Option<RwLockWriteGuard<'a, HashMap<K, V, S>>>,
        elem1: Option<(*const K, *mut V)>,
        elem2: Option<(*const K, *mut V)>,
        key1: K,
        key2: K,
    }
    unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Send for Entry<'a, K, V, S> {}
    unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Sync for Entry<'a, K, V, S> {}

    impl<'a, K: Clone + Eq + Hash + Debug, V: Debug, S: BuildHasher> Entry<'a, K, V, S> {
        pub fn or_insert_with(
            self,
            value: impl FnOnce((Option<()>, Option<()>)) -> (Option<V>, Option<V>),
        ) -> RefMut<'a, K, V, S> {
            match self {
                Entry {
                    shard1,
                    shard2,
                    elem1: Some((k1, v1)),
                    elem2: Some((k2, v2)),
                    ..
                } => {
                    dbg!(v1);
                    dbg!(v2);
                    RefMut {
                        guard1: shard1,
                        guard2: shard2,
                        k1,
                        k2,
                        v1,
                        v2,
                    }
                }
                Entry {
                    mut shard1,
                    shard2: None,
                    elem1,
                    elem2,
                    key1,
                    key2,
                } => {
                    let (r1, r2) = value((elem1.as_ref().map(|_| ()), elem2.as_ref().map(|_| ())));
                    let k1 = key1.clone();
                    let k2 = key2.clone();
                    if elem1.is_none() {
                        let value = r1.expect("some value");
                        let key = key1;
                        let shard = &mut shard1;
                        insert2_p1(key, shard, value)
                    }
                    if elem2.is_none() {
                        let value = r2.expect("some value");
                        let key = key2;
                        let shard = &mut shard1;
                        insert2_p1(key, shard, value)
                    }
                    let (k1, v1) = elem1.unwrap_or_else(|| {
                        let shard = &mut shard1;
                        insert2_p2(&k1, shard)
                    });
                    let (k2, v2) = elem2.unwrap_or_else(|| {
                        let shard = &mut shard1;
                        insert2_p2(&k2, shard)
                    });
                    dbg!(v1);
                    dbg!(v2);
                    RefMut {
                        guard1: shard1,
                        guard2: None,
                        k1,
                        k2,
                        v1,
                        v2,
                    }
                }
                Entry {
                    mut shard1,
                    shard2: Some(mut shard2),
                    elem1,
                    elem2,
                    key1,
                    key2,
                } => {
                    let (r1, r2) = value((elem1.as_ref().map(|_| ()), elem2.as_ref().map(|_| ())));
                    // let (k1, v1) = elem1.unwrap_or_else(|| {
                    //     let value = r1.expect("some value");
                    //     let key = key1;
                    //     let shard = &mut shard1;
                    //     println!("{:p}", shard);
                    //     println!("{:p}", &key);
                    //     println!("{}", shard.hasher().hash_one(&key));
                    //     insert2(key, shard, value)
                    // });
                    // let (k2, v2) = elem2.unwrap_or_else(|| {
                    //     let value = r2.expect("some value");
                    //     let key = key2;
                    //     let shard = &mut shard2;
                    //     insert2(key, shard, value)
                    // });
                    let k1 = key1.clone();
                    let k2 = key2.clone();
                    dbg!(&k1);
                    dbg!(&k2);
                    println!("{:p}", &k1);
                    println!("{:p}", &k2);
                    println!("{:p}", &r1);
                    println!("{:p}", &r2);
                    if elem1.is_none() {
                        let value = r1.expect("some value");
                        dbg!(&value);
                        println!("{:p}", &value);
                        let key = key1;
                        let shard = &mut shard1;
                        insert2_p1_shard(key, shard, value)
                    }
                    if elem2.is_none() {
                        let value = r2.expect("some value");
                        dbg!(&value);
                        println!("{:p}", &value);
                        let key = key2;
                        let shard = &mut shard2;
                        insert2_p1(key, shard, value)
                    }
                    let (k1, v1) = elem1.unwrap_or_else(|| {
                        let shard = &mut shard1;
                        dbg!(shard.hasher().hash_one(&k1));
                        let shard: &mut RwLockWriteGuard<HashMap<K, SharedValue<V>>> =
                            unsafe { std::mem::transmute(shard) };
                        insert2_p2_shard(&k1, shard)
                    });
                    let (k2, v2) = elem2.unwrap_or_else(|| {
                        let shard = &mut shard2;
                        insert2_p2(&k2, shard)
                    });
                    println!("{:p}", &shard1);
                    dbg!(shard1.len());
                    println!("{:p}", &shard2);
                    dbg!(shard2.len());
                    dbg!(v1);
                    dbg!(v2);
                    RefMut {
                        guard1: shard1,
                        guard2: Some(shard2),
                        k1,
                        k2,
                        v1,
                        v2,
                    }
                }
            }
        }
    }

    fn insert2<'a, K: Eq + Hash, V, S: BuildHasher>(
        key: K,
        shard: &mut RwLockWriteGuard<'a, HashMap<K, V, S>>,
        value: V,
    ) -> (*const K, *mut V) {
        let c = unsafe { std::ptr::read(&key) };
        shard.insert(key, value);
        // let shard: &mut RwLockWriteGuard<HashMap<K, SharedValue<V>>> =
        //     unsafe { std::mem::transmute(shard) };
        {
            // let shard: &'a mut RwLockWriteGuard<'a, HashMap<K, SharedValue<V>>> = shard;
            unsafe {
                use std::mem;
                dbg!();
                let (k, v) = shard.get_key_value(&c).unwrap();
                dbg!();
                let k = change_lifetime_const(k);
                dbg!();
                let v = &mut *shard_as_ptr2(v);
                dbg!();
                mem::forget(c);
                dbg!();
                (k, v)
            }
        }
    }

    fn insert2_p1<'a, K: Eq + Hash, V, S: BuildHasher>(
        key: K,
        shard: &mut RwLockWriteGuard<'a, HashMap<K, V, S>>,
        value: V,
    ) {
        println!("{:p}", &key);
        println!("{}", shard.hasher().hash_one(&key));
        println!("{:p}", &value);
        shard.insert(key, value);
    }

    fn insert2_p1_shard<'a, K: Eq + Hash, V, S: BuildHasher>(
        key: K,
        shard: &mut RwLockWriteGuard<'a, HashMap<K, V, S>>,
        value: V,
    ) {
        println!("{:p}", &key);
        println!("{}", shard.hasher().hash_one(&key));
        println!("{:p}", &value);
        // let shard: &mut RwLockWriteGuard<HashMap<K, SharedValue<V>>> =
        //             unsafe { std::mem::transmute(shard) };
        // let value: SharedValue<V> = SharedValue::new(value);
        println!("{:p}", &key);
        println!("{}", shard.hasher().hash_one(&key));
        println!("{:p}", &value);
        // todo!()
        shard.insert(key, value);
    }

    fn insert2_p2<'a, K: Eq + Hash, V, S: BuildHasher>(
        key: &K,
        shard: &mut RwLockWriteGuard<'a, HashMap<K, V, S>>,
    ) -> (*const K, *mut V) {
        unsafe {
            use std::mem;
            dbg!();
            println!("{:p}", &key);
            println!("{}", shard.hasher().hash_one(&key));
            let (k, v) = shard.get_key_value(key).unwrap();
            dbg!();
            let k = change_lifetime_const(k);
            dbg!();
            let v = &mut *shard_as_ptr2(v);
            dbg!();
            (k, v)
        }
    }

    fn insert2_p2_shard<'a, K: Eq + Hash, V, S: BuildHasher>(
        key: &K,
        shard: &mut RwLockWriteGuard<'a, HashMap<K, SharedValue<V>, S>>,
    ) -> (*const K, *mut V) {
        unsafe {
            dbg!();
            println!("{:p}", &key);
            println!("{}", shard.hasher().hash_one(&key));
            todo!();

            let (k, v) = shard.get_key_value(key).unwrap();
            dbg!();
            let k = change_lifetime_const(k);
            dbg!();
            let v = &mut *shard_as_ptr(v);
            dbg!();
            (k, v)
        }
    }

    fn insert<'a, K: Eq + Hash, V>(
        key: K,
        shard: &'a mut RwLockWriteGuard<'a, HashMap<K, SharedValue<V>>>,
        value: SharedValue<V>,
    ) -> (*const K, *mut V) {
        unsafe {
            use std::mem;
            use std::ptr;
            let c: K = ptr::read(&key);
            dbg!();
            println!("{:p}", &key);
            println!("{}", shard.hasher().hash_one(&key));
            println!("{:p}", &value);
            {
                // let shard: &mut RwLockWriteGuard<HashMap<K, V>> =
                //     unsafe { std::mem::transmute(shard) };
                // let value: V =
                //     unsafe { std::mem::transmute(value) };
                shard.insert(key, value);
            }
            dbg!();
            let (k, v) = shard.get_key_value(&c).unwrap();
            dbg!();
            let k = change_lifetime_const(k);
            dbg!();
            let v = &mut *shard_as_ptr(v);
            dbg!();
            mem::forget(c);
            dbg!();
            (k, v)
        }
    }

    /// # Safety
    ///
    /// Requires that you ensure the reference does not become invalid.
    /// The object has to outlive the reference.
    unsafe fn change_lifetime_const<'a, 'b, T>(x: &'a T) -> &'b T {
        &*(x as *const T)
    }

    pub struct RefMut<'a, K, V, S = RandomState> {
        guard1: RwLockWriteGuard<'a, HashMap<K, V, S>>,
        guard2: Option<RwLockWriteGuard<'a, HashMap<K, V, S>>>,
        k1: *const K,
        k2: *const K,
        v1: *mut V,
        v2: *mut V,
    }

    unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Send for RefMut<'a, K, V, S> {}
    unsafe impl<'a, K: Eq + Hash + Sync, V: Sync, S: BuildHasher> Sync for RefMut<'a, K, V, S> {}

    impl<'a, K: Eq + Hash, V, S: BuildHasher> RefMut<'a, K, V, S> {
        pub fn value_mut(&mut self) -> (&mut V, &mut V) {
            unsafe { (&mut *self.v1, &mut *self.v2) }
        }
    }
}

// WARN lazy subtrees are not complete
fn lazy_subtree_mapping<'a, 'b>(
    repositories: &'a multi_preprocessed::PreProcessedRepositories,
    partial_comp_cache: &'a crate::PartialDecompCache,
    src_tr: NodeIdentifier,
    dst_tr: NodeIdentifier,
) -> hyper_diff::matchers::Mapping<
    dashmap::mapref::one::RefMut<
        'a,
        NodeIdentifier,
        hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<
            hyper_ast::store::nodes::legion::HashedNodeRef<'a, NodeIdentifier>,
            u32,
        >,
    >,
    dashmap::mapref::one::RefMut<
        'a,
        NodeIdentifier,
        hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<
            hyper_ast::store::nodes::legion::HashedNodeRef<'a, NodeIdentifier>,
            u32,
        >,
    >,
    mapping_store::MultiVecStore<u32>,
> {
    use hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder;
    use hyper_diff::matchers::heuristic::gt::lazy2_greedy_subtree_matcher::LazyGreedySubtreeMatcher;
    use hyper_diff::matchers::mapping_store::DefaultMultiMappingStore;
    use hyper_diff::matchers::mapping_store::MappingStore;
    use hyper_diff::matchers::mapping_store::VecStore;
    use hyper_diff::matchers::Mapping;

    let hyperast = &repositories.processor.main_stores;
    let src = &src_tr;
    let dst = &dst_tr;
    let now = Instant::now();
    assert_ne!(src, dst);
    let (mut decompress_src, mut decompress_dst) = {
        use hyper_ast::types::DecompressedSubtree;
        let mut cached_decomp = |id: &NodeIdentifier| -> Option<
            dashmap::mapref::one::RefMut<NodeIdentifier, LazyPostOrder<HashedNodeRef<'a>, u32>>,
        > {
            let decompress = partial_comp_cache
                .try_entry(*id)?
                .or_insert_with(|| unsafe {
                    std::mem::transmute(LazyPostOrder::<_, u32>::decompress(
                        hyperast.node_store(),
                        id,
                    ))
                });
            Some(unsafe { std::mem::transmute(decompress) })
        };
        loop {
            match (cached_decomp(src), cached_decomp(dst)) {
                (Some(decompress_src), Some(decompress_dst)) => {
                    break (decompress_src, decompress_dst)
                }
                (None, None) => {
                    dbg!();
                }
                _ => {
                    dbg!(
                        partial_comp_cache.hash_usize(src),
                        partial_comp_cache.hash_usize(dst)
                    );
                    dbg!(
                        partial_comp_cache.determine_shard(partial_comp_cache.hash_usize(src)),
                        partial_comp_cache.determine_shard(partial_comp_cache.hash_usize(dst))
                    );
                }
            }
            sleep(Duration::from_secs(2));
        }
    };
    hyperast
        .node_store
        .resolve(decompress_src.original(&decompress_src.root()));
    hyperast
        .node_store
        .resolve(decompress_dst.original(&decompress_dst.root()));

    let mappings = VecStore::default();
    let mut mapper = Mapper {
        hyperast,
        mapping: Mapping {
            src_arena: decompress_src.value_mut(),
            dst_arena: decompress_dst.value_mut(),
            mappings,
        },
    };
    mapper.mapping.mappings.topit(
        mapper.mapping.src_arena.len(),
        mapper.mapping.dst_arena.len(),
    );
    dbg!();
    let mm = LazyGreedySubtreeMatcher::<
        'a,
        SimpleStores<TStore>,
        &mut LazyPostOrder<HashedNodeRef<'a>, u32>,
        &mut LazyPostOrder<HashedNodeRef<'a>, u32>,
        VecStore<_>,
    >::compute_multi_mapping::<DefaultMultiMappingStore<_>>(&mut mapper);
    dbg!();

    hyper_diff::matchers::Mapping {
        src_arena: decompress_src,
        dst_arena: decompress_dst,
        mappings: mm,
    }
}

pub fn child_by_type<'store, HAST: HyperAST<'store, IdN = NodeIdentifier>>(
    stores: &'store HAST,
    d: NodeIdentifier,
    t: &<HAST::T as types::Typed>::Type,
) -> Option<(NodeIdentifier, usize)> {
    let n = stores.node_store().resolve(&d);
    let s = n
        .children()
        .unwrap()
        .iter_children()
        .enumerate()
        .find(|(_, x)| {
            let n = stores.node_store().resolve(*x);
            n.get_type().eq(t)
        })
        .map(|(i, x)| (*x, i));
    s
}
