use std::{fmt::Debug, thread::sleep, time::Duration};

use axum::{Json, response::IntoResponse};
use enumset::{EnumSet, EnumSetType};
use hyper_diff::{
    decompressed_tree_store::{
        DecompressedWithParent, LazyDecompressedTreeStore, ShallowDecompressedTreeStore,
    },
    matchers::{
        Mapper,
        mapping_store::{self, MonoMappingStore, MultiMappingStore},
    },
};
use hyperast::{
    PrimInt,
    position::{
        compute_position, compute_position_and_nodes, compute_position_with_no_spaces,
        compute_range, path_with_spaces,
        position_accessors::{self, WithOffsets, WithPreOrderPath},
        resolve_range,
    },
    store::{SimpleStores, defaults::NodeIdentifier, nodes::legion::HashedNodeRef},
    types::{self, Childrn, HyperAST, NodeStore, WithChildren, WithHashs, WithStats},
};
use hyperast_vcs_git::{
    TStore, git::Repo, multi_preprocessed, preprocessed::child_at_path_tracked,
    processing::ConfiguredRepoTrait,
};
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_bool_from_anything;
use tokio::time::Instant;

use crate::{
    MappingAloneCache, PartialDecompCache, SharedState,
    changes::{self, DstChanges, SrcChanges},
    matching, no_space,
};

#[derive(Deserialize, Clone, Debug)]
pub struct TrackingParam {
    pub user: String,
    pub name: String,
    pub commit: String,
    pub file: String,
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
    pub start: Option<usize>,
    pub end: Option<usize>,
    pub before: Option<String>,
    #[serde(flatten)]
    pub flags: Flags,
}

#[derive(serde::Deserialize, serde::Serialize, Default, Clone, Debug)]
#[serde(default)]
pub struct Flags {
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub upd: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub child: bool,
    #[serde(deserialize_with = "deserialize_bool_from_anything")]
    pub parent: bool,
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

#[derive(Deserialize, Serialize)]
pub struct TrackingResult<IdN, Idx> {
    pub compute_time: f64,
    commits_processed: usize,
    #[serde(bound(serialize = "IdN: Clone + Into<self::IdN>, Idx: Serialize"))]
    src: PieceOfCode<IdN, Idx>,
    #[serde(bound(serialize = "IdN: Clone + Into<self::IdN>, Idx: Serialize"))]
    intermediary: Option<PieceOfCode<IdN, Idx>>,
    #[serde(bound(serialize = "IdN: Clone + Into<self::IdN>, Idx: Serialize"))]
    fallback: Option<PieceOfCode<IdN, Idx>>,
    #[serde(bound(serialize = "IdN: Clone + Into<self::IdN>, Idx: Serialize"))]
    matched: Vec<PieceOfCode<IdN, Idx>>,
}

// set the type of offset used to index in children list
type Idx = u16;
type IdN = NodeIdentifier;

impl IntoResponse for TrackingResult<IdN, Idx> {
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

impl<IdN, Idx> TrackingResult<IdN, Idx> {
    pub(crate) fn with_changes(
        self,
        (src_changes, dst_changes): (SrcChanges, DstChanges),
    ) -> TrackingResultWithChanges<IdN, Idx> {
        TrackingResultWithChanges {
            track: self,
            src_changes,
            dst_changes,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct TrackingResultWithChanges<IdN, Idx> {
    #[serde(bound(serialize = "IdN: Clone + Into<self::IdN>, Idx: Serialize"))]
    pub track: TrackingResult<IdN, Idx>,
    src_changes: SrcChanges,
    dst_changes: DstChanges,
}

impl IntoResponse for TrackingResultWithChanges<IdN, Idx> {
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
pub struct PieceOfCode<IdN = self::IdN, Idx = usize> {
    user: String,
    name: String,
    commit: String,
    path: Vec<Idx>,
    #[serde(bound(serialize = "IdN: Clone + Into<self::IdN>"))]
    #[serde(serialize_with = "custom_ser")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    path_ids: Vec<IdN>, // WARN this is not fetched::NodeIdentifier
    file: String,
    start: usize,
    end: usize,
}

fn custom_ser<IdN: Clone + Into<self::IdN>, S>(
    x: &Vec<IdN>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    use serde::ser::SerializeSeq;
    let mut seq = serializer.serialize_seq(Some(x.len()))?;
    for element in x {
        let element: self::IdN = element.clone().into();
        let id: u64 = unsafe { std::mem::transmute(element) };
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
    pub message: String,
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
) -> Result<TrackingResult<IdN, Idx>, TrackingError> {
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
    let repo_specifier = hyperast_vcs_git::git::Forge::Github.repo(user, name);
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
    log::debug!("done cloning {}", repository.spec);
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
            .pre_process_with_limit(&mut repository, "", &commit, 2)
            .map_err(|e| TrackingError {
                compute_time: now.elapsed().as_secs_f64(),
                commits_processed: 0,
                node_processed: 0,
                message: e.to_string(),
            })?;
        log::debug!("done construction of {commits:?} in {}", repository.spec);
        let src_oid = commits[0];
        let dst_oid = commits[1];
        match track_aux(
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
                log::debug!("tracked {commits:?}");
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
                log::debug!("tracking miss {commits:?}");
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
                if before.as_ref() == Some(&dst_oid.to_string()) {
                    let aaa = src.globalize(repository.spec, commit);
                    let (src, intermediary) = if let Some(src) = source {
                        (src, Some(aaa))
                    } else {
                        (aaa, None)
                    };
                    log::debug!("tracking skip {commits:?}");
                    return Ok(TrackingResult {
                        compute_time: now.elapsed().as_secs_f64(),
                        commits_processed,
                        src,
                        intermediary,
                        fallback: None,
                        matched: next,
                    }
                    .into());
                }
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
) -> Result<TrackingResult<IdN, Idx>, TrackingError> {
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
    let repo_specifier = hyperast_vcs_git::git::Forge::Github.repo(user, name);
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
    log::debug!("done cloning {}", repository.spec);
    // let mut get_mut = state.write().unwrap();
    // let state = get_mut.deref_mut();
    let mut commit = commit.clone();
    let mut node_processed = 0;
    let mut commits_processed = 1;
    let mut path: Vec<Idx> = path.split("/").filter_map(|x| x.parse().ok()).collect();
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
        log::debug!("done construction of {commits:?} in {}", repository.spec);
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
        match track_aux2(state.clone(), &repository, src_oid, dst_oid, &path, &flags) {
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
) -> Result<TrackingResultWithChanges<IdN, Idx>, TrackingError> {
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
    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo(user, name);
    let repo_handle = state
        .repositories
        .write()
        .unwrap()
        .get_config(repo_spec)
        .ok_or_else(|| TrackingError {
            compute_time: now.elapsed().as_secs_f64(),
            commits_processed: 0,
            node_processed: 0,
            message: "missing config for repository".to_string(),
        })?;
    let mut repository = repo_handle.fetch();
    log::debug!("done cloning {}", repository.spec);
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
            .pre_process_with_limit(&mut repository, "", &commit, 4)
            .map_err(|e| TrackingError {
                compute_time: now.elapsed().as_secs_f64(),
                commits_processed: 0,
                node_processed: 0,
                message: e.to_string(),
            })?;
        log::warn!(
            "done construction of {commits:?} in {}",
            repository.spec.user()
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
        match track_aux2(state.clone(), &repository, src_oid, dst_oid, &path, &flags) {
            MappingResult::Direct { src: aaa, matches } => {
                let changes = changes::added_deleted(state, &repository, dst_oid, ori_oid.unwrap())
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
                dbg!();
                let changes = changes::added_deleted(state, &repository, dst_oid, ori_oid.unwrap())
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
                // TODO fix issue of not stoping when failling to match accurately,
                // most likely related to miss use of fallback value ?
                if commits.len() < 3 || !(node_processed < MAX_NODES) {
                    // no commit remaining (first + second < 3)
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
                    // dbg!(next);
                    path = next.path.clone();
                    commit = next.commit.clone();
                }
            }
        }
    }
    // TODO should not look as an irrecoverable fail...
    Err(TrackingError {
        compute_time: now.elapsed().as_secs_f64(),
        commits_processed,
        node_processed,
        message: format!("reached max number of diffed nodes: (ie. {})", MAX_NODES),
    })
}

enum MappingResult<IdN, Idx, T = PieceOfCode<IdN, Idx>> {
    Direct {
        src: LocalPieceOfCode<IdN, Idx>,
        matches: Vec<T>,
    },
    Missing {
        src: LocalPieceOfCode<IdN, Idx>,
        fallback: T,
    },
    Error(String),
    Skipped {
        nodes: usize,
        src: LocalPieceOfCode<IdN, Idx>,
        next: Vec<T>,
    },
}

#[derive(Clone, PartialEq, Debug)]
struct LocalPieceOfCode<IdN, Idx> {
    file: String,
    start: usize,
    end: usize,
    path: Vec<Idx>,
    path_ids: Vec<IdN>,
}

impl<'a, S, IdN: Clone, Idx: Clone> From<(&S, &'a LocalPieceOfCode<IdN, Idx>)>
    for LocalPieceOfCode<IdN, Idx>
{
    fn from((_, p): (&S, &'a LocalPieceOfCode<IdN, Idx>)) -> Self {
        p.clone()
    }
}

impl<IdN, Idx> LocalPieceOfCode<IdN, Idx> {
    pub(crate) fn from_position(
        pos: &hyperast::position::Position,
        path: Vec<Idx>,
        path_ids: Vec<IdN>,
    ) -> Self {
        let std::ops::Range { start, end } = pos.range();
        let file = pos.file().to_str().unwrap().to_string();
        Self {
            file,
            start,
            end,
            path,
            path_ids,
        }
    }
    pub(crate) fn from_file_and_range(
        file: &std::path::Path,
        range: std::ops::Range<usize>,
        path: Vec<Idx>,
        path_ids: Vec<IdN>,
    ) -> Self {
        let std::ops::Range { start, end } = range;
        let file = file.to_str().unwrap().to_string();
        Self {
            file,
            start,
            end,
            path,
            path_ids,
        }
    }
    pub(crate) fn from_pos<P>(pos: &P) -> Self
    where
        P: WithOffsets<Idx = Idx>
            + WithPreOrderPath<IdN>
            + hyperast::position::position_accessors::FileAndOffsetPostionT<IdN, IdO = usize>,
    {
        let mut path = vec![];
        let mut path_ids = vec![];
        for (o, i) in pos.iter_offsets_and_nodes() {
            path.push(o);
            path_ids.push(i);
        }
        Self {
            file: pos.file().to_str().unwrap().to_owned(),
            start: pos.start(),
            end: pos.end(),
            path,
            path_ids,
        }
    }
    pub(crate) fn globalize(self, spec: Repo, commit: impl ToString) -> PieceOfCode<IdN, Idx> {
        let LocalPieceOfCode {
            file,
            start,
            end,
            path,
            path_ids,
        } = self;
        let commit = commit.to_string();
        PieceOfCode {
            user: spec.user().to_string(),
            name: spec.name().to_string(),
            commit,
            path,
            path_ids,
            file,
            start,
            end,
        }
    }
    fn map_path<Idx2, F: Fn(Idx) -> Idx2>(self, f: F) -> LocalPieceOfCode<IdN, Idx2> {
        let LocalPieceOfCode {
            file,
            start,
            end,
            path,
            path_ids,
        } = self;
        let path = path.into_iter().map(f).collect();
        LocalPieceOfCode {
            file,
            start,
            end,
            path,
            path_ids,
        }
    }
}

#[derive(Clone)]
struct TargetCodeElement<IdN, Idx> {
    start: usize,
    end: usize,
    path: Vec<Idx>,
    path_no_spaces: Vec<Idx>,
    root: IdN,
    node: IdN,
}

impl<IdN: Clone, Idx> position_accessors::SolvedPosition<IdN> for TargetCodeElement<IdN, Idx> {
    fn node(&self) -> IdN {
        self.node.clone()
    }
}

impl<IdN: Clone, Idx> position_accessors::RootedPosition<IdN> for TargetCodeElement<IdN, Idx> {
    fn root(&self) -> IdN {
        self.root.clone()
    }
}

impl<IdN, Idx: PrimInt> position_accessors::WithPath<IdN> for TargetCodeElement<IdN, Idx> {}

impl<IdN, Idx: PrimInt> position_accessors::WithPreOrderOffsets for TargetCodeElement<IdN, Idx> {
    type It<'a>
        = std::iter::Copied<std::slice::Iter<'a, Idx>>
    where
        Idx: 'a,
        Self: 'a;

    fn iter_offsets(&self) -> Self::It<'_> {
        self.path.iter().copied()
    }
}

impl<IdN, Idx: PrimInt> position_accessors::WithPreOrderPath<IdN> for TargetCodeElement<IdN, Idx> {
    type ItPath = std::vec::IntoIter<(Idx, IdN)>;

    fn iter_offsets_and_nodes(&self) -> Self::ItPath {
        todo!()
    }
}

impl<IdN, Idx: PrimInt> position_accessors::WithOffsets for TargetCodeElement<IdN, Idx> {
    type Idx = Idx;
}

impl<IdN, Idx> position_accessors::OffsetPostionT<IdN> for TargetCodeElement<IdN, Idx> {
    type IdO = usize;

    fn offset(&self) -> Self::IdO {
        self.start
    }

    fn len(&self) -> Self::IdO {
        self.end - self.start
    }

    fn start(&self) -> Self::IdO {
        self.start
    }

    fn end(&self) -> Self::IdO {
        self.end
    }
}

impl<IdN, Idx: PrimInt> compute::WithPreOrderOffsetsNoSpaces for TargetCodeElement<IdN, Idx> {
    type It<'a>
        = std::slice::Iter<'a, Idx>
    where
        Idx: 'a,
        Self: 'a;

    fn iter_offsets_nospaces(&self) -> Self::It<'_> {
        self.path_no_spaces.iter()
    }
}

fn track_aux(
    state: std::sync::Arc<crate::AppState>,
    repo_handle: &impl ConfiguredRepoTrait<
        Config = hyperast_vcs_git::processing::ParametrizedCommitProcessorHandle,
    >,
    src_oid: hyperast_vcs_git::git::Oid,
    dst_oid: hyperast_vcs_git::git::Oid,
    file: &String,
    start: Option<usize>,
    end: Option<usize>,
    flags: &Flags,
) -> MappingResult<IdN, Idx> {
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories
        .get_commit(repo_handle.config(), &src_oid)
        .unwrap();
    let src_tr = commit_src.ast_root;
    let commit_dst = repositories
        .get_commit(&repo_handle.config(), &dst_oid)
        .unwrap();
    let dst_tr = commit_dst.ast_root;
    let stores = &repositories.processor.main_stores;

    // let size = node_store.resolve(src_tr).size();
    log::debug!("tracking {file}");
    let file_node =
        child_at_path_tracked(&repositories.processor.main_stores, src_tr, file.split("/"));

    let Some((file_node, offsets_to_file)) = file_node else {
        return MappingResult::Error("not found".into());
    };

    dbg!(&offsets_to_file);
    let mut path_to_target = vec![];
    let (node, offsets_in_file) = resolve_range(file_node, start.unwrap_or(0), end, stores);
    path_to_target.extend(offsets_to_file.iter().map(|x| *x as Idx));
    dbg!(&node);
    dbg!(&offsets_in_file);
    path_to_target.extend(offsets_in_file.iter().map(|x| *x as Idx));

    let computed_range = compute_range(file_node, &mut offsets_in_file.into_iter(), stores);
    dbg!(computed_range.0, computed_range.1);
    dbg!(&computed_range.2);
    let no_spaces_path_to_target = if false {
        // TODO use this version
        use hyperast::position;
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
    let target = TargetCodeElement::<IdN, Idx> {
        start: computed_range.0,
        end: computed_range.1,
        path: path_to_target.clone(),
        path_no_spaces: no_spaces_path_to_target,
        node: computed_range.2,
        root: src_tr,
    };
    let postprocess_matching = |p: LocalPieceOfCode<IdN, Idx>| {
        p.globalize(repo_handle.spec().clone(), dst_oid.to_string())
    };
    compute::do_tracking(
        &repositories,
        &state.partial_decomps,
        &state.mappings_alone,
        flags,
        &target,
        dst_tr,
        &postprocess_matching,
    )
}

fn track_aux2(
    state: std::sync::Arc<crate::AppState>,
    repo_handle: &impl ConfiguredRepoTrait<
        Config = hyperast_vcs_git::processing::ParametrizedCommitProcessorHandle,
    >,
    src_oid: hyperast_vcs_git::git::Oid,
    dst_oid: hyperast_vcs_git::git::Oid,
    path: &[Idx],
    flags: &Flags,
) -> MappingResult<IdN, Idx> {
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories
        .get_commit(repo_handle.config(), &src_oid)
        .unwrap();
    let src_tr = commit_src.ast_root;
    let commit_dst = repositories
        .get_commit(repo_handle.config(), &dst_oid)
        .unwrap();
    let dst_tr = commit_dst.ast_root;
    let stores = &repositories.processor.main_stores;

    let path_to_target: Vec<_> = path.iter().map(|x| *x as u16).collect();
    dbg!(&path_to_target);
    let (pos, target_node, no_spaces_path_to_target): _ = if false {
        // NOTE trying stuff
        // TODO use this version
        use hyperast::position;
        use position::offsets;
        use position::offsets_and_nodes;
        let src = offsets::OffsetsRef::from(path_to_target.as_slice());
        let src = src.with_root(src_tr);
        let src = src.with_store(stores);
        // let no_spaces_path_to_target: offsets::Offsets<_, position::tags::TopDownNoSpace> =
        //     src.compute_no_spaces::<_, offsets::Offsets<_, _>>();
        let position::CompoundPositionPreparer(pos, path) = src
            .compute_no_spaces::<_, position::CompoundPositionPreparer<
                position::Position,
                offsets_and_nodes::StructuralPosition<_, _, position::tags::TopDownNoSpace>,
            >>();
        // no_spaces_path_to_target.into()
        let (node, path) = path.into();
        (pos, node, path)
    } else {
        compute_position_with_no_spaces(src_tr, &mut path_to_target.iter().map(|x| *x), stores)
    };
    let range = pos.range();
    let target = TargetCodeElement::<IdN, Idx> {
        start: range.start,
        end: range.end,
        path: path_to_target.clone(),
        path_no_spaces: no_spaces_path_to_target,
        node: target_node,
        root: src_tr,
    };
    let postprocess_matching = |p: LocalPieceOfCode<IdN, Idx>| {
        p.globalize(repo_handle.spec().clone(), dst_oid.to_string())
    };
    compute::do_tracking(
        &repositories,
        &state.partial_decomps,
        &state.mappings_alone,
        flags,
        &target,
        dst_tr,
        &postprocess_matching,
    )
}

mod compute;
mod more;

#[cfg(feature = "experimental")]
mod my_dash;
