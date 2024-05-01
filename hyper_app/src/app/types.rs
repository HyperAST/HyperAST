use egui_addon::code_editor;
use hyper_ast::store::nodes::fetched::NodeIdentifier;

use std::{collections::HashSet, hash::Hash, ops::Range};

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
    pub(crate) path: Vec<usize>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(crate) path_ids: Vec<NodeIdentifier>,
}

impl Default for CodeRange {
    fn default() -> Self {
        Self {
            file: Default::default(),
            range: Default::default(),
            path: Default::default(),
            path_ids: Default::default(),
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
    pub(super) hightlight: String,
    pub(super) cst: bool,
    pub(super) spacing: bool,
    pub(super) syntax: bool,
    pub(super) ast: bool,
    pub(super) type_decls: bool,
    pub(super) licence: bool,
    pub(super) doc: bool,
    // pub(super) ser_opt_cpp_text: String,
    // pub(super) ser_opt_java_text: String,
    // TODO use an enum set btw...
    #[serde(skip)]
    pub(super) ser_opt_cpp: HashSet<hyper_ast_gen_ts_cpp::types::Type>,
    #[serde(skip)]
    pub(super) ser_opt_java: HashSet<hyper_ast_gen_ts_java::types::Type>,
    #[serde(skip)]
    pub(super) hide_opt_cpp: HashSet<hyper_ast_gen_ts_cpp::types::Type>,
    #[serde(skip)]
    pub(super) hide_opt_java: HashSet<hyper_ast_gen_ts_java::types::Type>,
}

impl Default for ComputeConfigAspectViews {
    fn default() -> Self {
        let mut ser_opt_cpp: HashSet<hyper_ast_gen_ts_cpp::types::Type> = Default::default();
        ser_opt_cpp.insert(hyper_ast_gen_ts_cpp::types::Type::FunctionDeclarator);
        let mut ser_opt_java: HashSet<hyper_ast_gen_ts_java::types::Type> = Default::default();
        ser_opt_java.insert(hyper_ast_gen_ts_java::types::Type::MethodDeclaration);
        let hide_opt_cpp: HashSet<hyper_ast_gen_ts_cpp::types::Type> = Default::default();
        let hide_opt_java: HashSet<hyper_ast_gen_ts_java::types::Type> = Default::default();
        Self {
            commit: Default::default(),
            path: "".into(),
            hightlight: "0".into(),
            cst: true,
            spacing: false,
            syntax: false,
            ast: false,
            type_decls: false,
            licence: false,
            doc: false,
            ser_opt_cpp,
            ser_opt_java,
            hide_opt_cpp,
            hide_opt_java,
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
            // id: "cd339e2c5f0e5c1e42c66b890f02bc282c3a0ea1".into(), // 61074989324d20e7d9cd387cee830a31a7e68aca // 4acedc53a13a727be3640fe234f7e261d2609d58
            id: "7f2eb10e93879bc569c7ddf6fb51d6f812cc477c".into(),
            // # stockfish
            // * long 7f2eb10e93879bc569c7ddf6fb51d6f812cc477c
            // * more in past "587bc647d7d14b53d8625c4446006e23a4acd82a".into()
            // * close to first b8e487ff9caffb5061f680b1919ab2fe442bc0a1
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default, PartialEq, Eq, Clone, Copy)]
pub enum SelectedConfig {
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
            // user: "INRIA".to_string(),
            // name: "spoon".to_string(),
            user: "official-stockfish".to_string(),
            name: "Stockfish".to_string(),
        }
    }
}

impl From<&ComputeConfigAspectViews> for SelectedConfig {
    fn from(_: &ComputeConfigAspectViews) -> Self {
        Self::Aspects
    }
}

#[derive(Default, Clone)]
pub struct Languages;

impl egui_addon::Languages for Languages {
    fn get(&self, lang: &str) -> Option<egui_addon::Lang> {
        match lang {
            #[cfg(not(target_arch = "wasm32"))]
            "JavaScript" => Some(egui_addon::Lang {
                name: "JavaScript".into(),
                lang: tree_sitter_javascript::language().into(),
            }),
            _ => None,
        }
    }
}

#[derive(
    serde::Deserialize, serde::Serialize, autosurgeon::Hydrate, autosurgeon::Reconcile, Clone, Debug,
)]
#[serde(default)]
pub(crate) struct CodeEditors<T = code_editor::CodeEditor<Languages>> {
    pub(crate) description: T,
    pub(crate) init: T,
    pub(crate) filter: T,
    pub(crate) accumulate: T,
}

impl<T> CodeEditors<T> {
    pub(crate) fn to_shared<U>(self) -> CodeEditors<U>
    where
        T: Into<U>,
    {
        CodeEditors {
            description: self.description.into(),
            init: self.init.into(),
            filter: self.filter.into(),
            accumulate: self.accumulate.into(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Resource<T> {
    /// HTTP response
    pub(crate) response: ehttp::Response,

    pub(crate) content: Option<T>,
    // /// If set, the response was an image.
    // image: Option<RetainedImage>,
}

impl<T> Resource<T> {
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> Resource<U> {
        Resource {
            response: self.response,
            content: self.content.map(f),
        }
    }
}
