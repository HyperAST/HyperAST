use hyperast_vcs_git::preprocessed::child_at_path;
use serde::Deserialize;
use tokio::time::Instant;

use crate::{utils, SharedState};

#[derive(Deserialize, Clone, Debug)]
pub struct FetchFileParam {
    user: String,
    name: String,
    commit: String,
    file: String,
}

pub fn from_hyperast(state: SharedState, path: FetchFileParam) -> Result<String, String> {
    let now = Instant::now();
    let FetchFileParam {
        user,
        name,
        commit,
        file,
    } = path.clone();
    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo(user, name);
    let repo = state
        .repositories
        .write()
        .unwrap()
        .get_config(repo_spec)
        .ok_or_else(|| "missing config for repository".to_string())?;
    let mut repo = repo.fetch();
    log::debug!("done cloning {}", repo.spec);
    let commits = utils::handle_pre_processing(&state, &mut repo, "", &commit, 2).map_err(|e| e.to_string())?;
    // let commits = state
    //     .repositories
    //     .write()
    //     .unwrap()
    //     .pre_process_with_limit(&mut repo, "", &commit, 2)
    //     .map_err(|e| e.to_string())?;
    log::debug!("done construction of {commits:?} in {}", repo.spec,);
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories.get_commit(&repo.config, &commits[0]).unwrap();
    let src_tr = commit_src.ast_root;

    // let size = node_store.resolve(src_tr).size();
    log::debug!("searching for {file}");
    let content = child_at_path(&repositories.processor.main_stores, src_tr, file.split("/"));

    let Some(content) = content else {
        return Err("not found".to_string());
    };

    let content = hyperast::nodes::TextSerializer::new(&repositories.processor.main_stores, content);
    log::debug!("sending file {file} from {}/{commits:?}", repo.spec);

    Ok(content.to_string())
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
