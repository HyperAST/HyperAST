use super::code_editor;

use std::{collections::HashMap, hash::Hash, ops::Range};

#[derive(Hash, PartialEq, Eq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Repo {
    pub(crate) user: String,
    pub(crate) name: String,
}

pub(crate) type CommitId = String;

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub(crate) struct ComputeConfigMulti {
    pub(crate) list: Vec<Commit>,
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub(crate) struct ComputeConfigDiff {
    pub(crate) repo: Repo,
    pub(crate) before: CommitId,
    pub(crate) after: CommitId,
}

#[derive(Hash, PartialEq, Eq, Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct CodeRange {
    #[serde(flatten)]
    pub(crate) file: FileIdentifier,
    #[serde(flatten)]
    pub(crate) range: Option<Range<usize>>,
}

impl Default for CodeRange {
    fn default() -> Self {
        Self {
            file: Default::default(),
            range: Default::default(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct ComputeConfigTracking {
    pub(crate) target: CodeRange,
}

impl Default for ComputeConfigTracking {
    fn default() -> Self {
        Self {
            target: Default::default(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct ComputeConfigAspectViews {
    pub(super) commit: Commit,
    pub(super) path: String,
    pub(super) cst: bool,
    pub(super) ast: bool,
    pub(super) type_decls: bool,
    pub(super) licence: bool,
    pub(super) doc: bool,
}

impl Default for ComputeConfigAspectViews {
    fn default() -> Self {
        Self {
            commit: Default::default(),
            path: "11/2/1".into(),
            cst: true,
            ast: false,
            type_decls: false,
            licence: false,
            doc: false,
            // commit: "4acedc53a13a727be3640fe234f7e261d2609d58".into(),
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct FileIdentifier {
    #[serde(flatten)]
    pub(crate) commit: Commit,
    #[serde(alias = "file")]
    pub(crate) file_path: String,
}

impl Default for FileIdentifier {
    fn default() -> Self {
        Self {
            commit: Default::default(),
            file_path: "src/main/java/spoon/Launcher.java".to_string(),
        }
    }
}

#[derive(Hash, PartialEq, Eq, Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct Commit {
    #[serde(flatten)]
    pub(crate) repo: Repo,
    #[serde(alias = "commit")]
    pub(crate) id: CommitId,
}

impl Default for Commit {
    fn default() -> Self {
        Self {
            repo: Default::default(),
            id: "4acedc53a13a727be3640fe234f7e261d2609d58".into(), //"61074989324d20e7d9cd387cee830a31a7e68aca".into(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, PartialEq, Eq, Clone, Copy)]
pub(crate) enum SelectedConfig {
    Single,
    Multi,
    Diff,
    Tracking,
    #[default]
    LongTracking,
    Aspects,
}

impl Default for Repo {
    fn default() -> Self {
        Self {
            user: "INRIA".to_string(),
            name: "spoon".to_string(),
        }
    }
}

impl From<&ComputeConfigAspectViews> for SelectedConfig {
    fn from(_: &ComputeConfigAspectViews) -> Self {
        Self::Aspects
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct CodeEditors {
    pub(crate) init: code_editor::CodeEditor,
    pub(crate) filter: code_editor::CodeEditor,
    pub(crate) accumulate: code_editor::CodeEditor,
}

#[derive(Debug)]
pub(crate) struct Resource<T> {
    /// HTTP response
    pub(crate) response: ehttp::Response,

    pub(crate) content: Option<T>,
    // /// If set, the response was an image.
    // image: Option<RetainedImage>,
}

pub type Languages = HashMap<String, Lang>;

#[derive(Debug, Clone)]
pub struct Lang {
    pub name: String,
    pub lang: tree_sitter::Language,
}

impl Hash for Lang {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}
