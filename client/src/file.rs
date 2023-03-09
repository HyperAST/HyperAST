use std::{fmt::Display, rc::Rc};

use axum::{body::HttpBody, Json};
use hyper_ast::types::LabelStore;
use hyper_ast_cvs_git::{git::fetch_github_repository, preprocessed::child_at_path};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::SharedState;

#[derive(Deserialize, Clone, Debug)]
pub struct FetchFileParam {
    user: String,
    name: String,
    commit: String,
    file: String,
}

pub fn from_hyper_ast(state: SharedState, path: FetchFileParam) -> Result<String, String> {
    let now = Instant::now();
    let FetchFileParam {
        user,
        name,
        commit,
        file,
    } = path.clone();
    let mut repo = fetch_github_repository(&format!("{}/{}", user, name));
    log::warn!("done cloning {user}/{name}");
    let mut get_mut = state.write().unwrap();
    let commits = get_mut
        .repositories
        .pre_process_with_limit(&mut repo, "", &commit, "", 2)
        .map_err(|e| e.to_string())?;
    log::warn!("done construction of {commits:?} in {user}/{name}");
    let commit_src = get_mut
        .repositories
        .commits
        .get_key_value(&commits[0])
        .unwrap();
    let src_tr = commit_src.1.ast_root;
    let node_store = &get_mut.repositories.processor.main_stores.node_store;

    // let size = node_store.resolve(src_tr).size();
    log::error!("searching for {file}");
    let file = child_at_path(
        &get_mut.repositories.processor.main_stores,
        src_tr,
        file.split("/"),
    );

    let Some(file) = file else {
        return Err("not found".to_string());
    };

    let mut out = BuffOut::default();

    let file = hyper_ast::nodes::serialize(
        |id| -> _ {
            node_store
                .resolve(id.clone())
                .into_compressed_node()
                .unwrap()
        },
        |id| -> _ {
            get_mut
                .repositories
                .processor
                .main_stores
                .label_store
                .resolve(id)
                .to_owned()
        },
        &file,
        &mut out,
        "\n",
    );

    Ok(out.into())
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
