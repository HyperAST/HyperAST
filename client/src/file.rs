use hyper_ast::types::LabelStore;
use hyper_ast_cvs_git::{git::fetch_github_repository, preprocessed::child_at_path};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::{RepoConfig, SharedState};

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
    let repo_spec = hyper_ast_cvs_git::git::Forge::Github.repo(user, name);
    let configs = state.configs.read().unwrap();
    let config = configs
        .get(&repo_spec)
        .ok_or_else(|| "missing config for repository".to_string())?;
    let mut repo = repo_spec.fetch();
    log::warn!("done cloning {}/{}", repo_spec.user, repo_spec.name);
    let commits = state
        .repositories
        .write()
        .unwrap()
        .pre_process_with_config(&mut repo, "", &commit, config.into())
        .map_err(|e| e.to_string())?;
    log::warn!(
        "done construction of {commits:?} in {}/{}",
        repo_spec.user,
        repo_spec.name
    );
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories.commits.get_key_value(&commits[0]).unwrap();
    let src_tr = commit_src.1.ast_root;
    let node_store = &repositories.processor.main_stores.node_store;

    // let size = node_store.resolve(src_tr).size();
    log::error!("searching for {file}");
    let file = child_at_path(&repositories.processor.main_stores, src_tr, file.split("/"));

    let Some(file) = file else {
        return Err("not found".to_string());
    };

    let file = hyper_ast::nodes::TextSerializer::new(&repositories.processor.main_stores, file);

    Ok(file.to_string())
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
