use std::{fmt::Display, rc::Rc};

use axum::{body::HttpBody, Json};
use hyper_ast::types::LabelStore;
use hyper_ast_cvs_git::{git::{fetch_github_repository, retrieve_commit}, preprocessed::child_at_path};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::SharedState;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Param {
    user: String,
    name: String,
    /// either a commit id or a tag
    version: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Metadata {
    /// commit message
    message: Option<String>,
    /// parents commits
    /// if multiple parents, the first one should be where the merge happends
    parents: Vec<String>,
    /// tree corresponding to version
    tree: Option<String>,
    /// offset in minutes
    timezone: i32,
    /// seconds
    time: i64,
}

pub fn commit_metadata(_state: SharedState, path: Param) -> Result<Json<Metadata>, String> {
    let Param {
        user,
        name,
        version,
    } = path.clone();
    let repo = fetch_github_repository(&format!("{}/{}", user, name));
    log::warn!("done cloning {user}/{name}");
    let commit = retrieve_commit(&repo, &version);
    let commit = commit.map_err(|err|err.to_string())?;
    log::warn!("done retrieving version {version}");

    let time = commit.time();
    let timezone = time.offset_minutes();
    let time = time.seconds();
    let tree = commit.tree().ok().map(|x|x.id().to_string());
    let parents = commit.parent_ids().map(|x|x.to_string()).collect();
    let message = commit.message().map(|s|s.to_string());
    
    Ok(Json(Metadata {
        message,
        parents,
        tree,
        timezone,
        time,
    }))
}

#[derive(Default)]
struct BuffOut {
    buff: String,
}

impl std::fmt::Write for BuffOut {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        Ok(self.buff.extend(s.chars()))
    }
}

impl From<BuffOut> for String {
    fn from(value: BuffOut) -> Self {
        value.buff
    }
}
