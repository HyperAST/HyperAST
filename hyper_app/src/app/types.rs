use egui_addon::code_editor;
use hyper_ast::store::nodes::fetched::NodeIdentifier;
use re_ui::UiExt;

use std::{collections::HashSet, hash::Hash, ops::Range};

#[derive(Hash, PartialEq, Eq, Clone, Debug, serde::Deserialize, serde::Serialize)]
pub(crate) struct Repo {
    pub(crate) user: String,
    pub(crate) name: String,
}
// TODO uuse [u8;20]
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
    #[serde(rename = "file")]
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
    #[serde(rename = "commit")]
    #[serde(alias = "id")]
    pub(crate) id: CommitId,
}

impl Repo {
    pub fn with(self, id: impl Into<String>) -> Commit {
        Commit {
            repo: self,
            id: id.into(),
        }
    }
}

impl Default for Commit {
    fn default() -> Self {
        Repo::default().with("7f2eb10e93879bc569c7ddf6fb51d6f812cc477c")
        // id: "cd339e2c5f0e5c1e42c66b890f02bc282c3a0ea1".into(), // 61074989324d20e7d9cd387cee830a31a7e68aca // 4acedc53a13a727be3640fe234f7e261d2609d58
        // id: "7f2eb10e93879bc569c7ddf6fb51d6f812cc477c".into(),
        // # stockfish
        // * long 7f2eb10e93879bc569c7ddf6fb51d6f812cc477c
        // * more in past "587bc647d7d14b53d8625c4446006e23a4acd82a".into()
        // * close to first b8e487ff9caffb5061f680b1919ab2fe442bc0a1
    }
}

#[derive(
    serde::Deserialize,
    serde::Serialize,
    Default,
    PartialEq,
    Eq,
    Clone,
    Copy,
    strum_macros::EnumIter,
)]
pub enum SelectedConfig {
    Single,
    #[default]
    Querying,
    Tsg,
    Smells,
    Multi,
    Diff,
    Tracking,
    LongTracking,
    Aspects,
}

impl SelectedConfig {
    pub const fn title(&self) -> impl Into<String> + AsRef<str> {
        match self {
            SelectedConfig::Single => "Single Repository",
            SelectedConfig::Querying => "Querying",
            SelectedConfig::Tsg => "TSG",
            SelectedConfig::Smells => "Interactive Finder", //ℹ //🗖
            SelectedConfig::Multi => "Multi Repo",
            SelectedConfig::Diff => "Tree Diff",
            SelectedConfig::Tracking => "Code Tracking",
            SelectedConfig::LongTracking => "Long Tracking",
            SelectedConfig::Aspects => "Aspects Views",
        }
    }

    pub(crate) const fn enabled(&self) -> bool {
        match self {
            SelectedConfig::Single => true,
            SelectedConfig::Querying => true,
            SelectedConfig::Tsg => true,
            SelectedConfig::Smells => true,
            SelectedConfig::Multi => false,
            SelectedConfig::Diff => true,
            SelectedConfig::Tracking => false,
            SelectedConfig::LongTracking => true,
            SelectedConfig::Aspects => true,
        }
    }

    pub(crate) fn on_hover_show(&self, ui: &mut egui::Ui) {
        ui.markdown_ui(ui.id().with(self.title().as_ref()),
        match self {
            SelectedConfig::Single =>"TODO",
            SelectedConfig::Querying => "TODO",
            SelectedConfig::Tsg => 
                r#"Compute a graph using the [tree-sitter-graph DSL](https://docs.rs/tree-sitter-graph/latest/tree_sitter_graph/reference/index.html)"#,
            SelectedConfig::Smells => 
                "Search for problematic code patterns",
            SelectedConfig::Multi => "TODO",
            SelectedConfig::Diff => "TODO",
            SelectedConfig::Tracking => "TODO",
            SelectedConfig::LongTracking => "TODO",
            SelectedConfig::Aspects => "TODO",
        })
    }
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

impl From<[&str;2]> for Repo {
    fn from(value: [&str;2]) -> Self {
        let [user, name] = value.map(|s|s.to_string());
        Self { user, name }
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

pub(crate) trait WithDesc<T> {
    fn desc(&self) -> &T;
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

#[derive(
    serde::Deserialize, serde::Serialize, autosurgeon::Hydrate, autosurgeon::Reconcile, Clone, Debug,
)]
#[serde(default)]
pub(crate) struct QueryEditor<T = code_editor::CodeEditor<Languages>> {
    pub(crate) description: T,
    pub(crate) query: T,
}

pub trait EditorHolder {
    type Item;
    fn iter_editors_mut(&mut self) -> impl Iterator<Item = &mut Self::Item>;
}

#[derive(
    serde::Deserialize, serde::Serialize, autosurgeon::Hydrate, autosurgeon::Reconcile, Clone, Debug,
)]
#[serde(default)]
pub(crate) struct TsgEditor<T = code_editor::CodeEditor<Languages>> {
    pub(crate) description: T,
    pub(crate) query: T,
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

#[derive(serde::Deserialize, serde::Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Config {
    Any,
    MavenJava,
    MakeCpp,
}

impl Config {
    pub fn language(&self) -> &'static str {
        match self {
            Config::Any => "",
            Config::MavenJava => "Java",
            Config::MakeCpp => "Cpp",
        }
    }
}
impl Config {
    pub(crate) fn show_combo_box(
        &mut self,
        ui: &mut egui::Ui,
        label: impl Into<egui::WidgetText>,
    ) -> egui::InnerResponse<std::option::Option<()>> {
        egui::ComboBox::from_label(label)
            .selected_text(format!("{:?}", self))
            .show_ui(ui, |ui| {
                ui.selectable_value(self, super::types::Config::Any, "Any");
                ui.selectable_value(self, super::types::Config::MavenJava, "Java");
                ui.selectable_value(self, super::types::Config::MakeCpp, "Cpp");
            })
    }
}
