use self::{
    single_repo::ComputeConfigSingle,
    tree_view::store::FetchedHyperAST,
    types::{Commit, Repo},
};
use crate::{
    command::{CommandReceiver, CommandSender, UICommand},
    command_palette::CommandPalette,
};
use code_aspects::remote_fetch_node;
use commit::{CommitSlice, SelectedProjects, fetch_commit};
use core::f32;
use egui::util::hash;
use egui_addon::{
    Lang,
    code_editor::{self, generic_text_buffer::TextBuffer},
    egui_utils::radio_collapsing,
    syntax_highlighting,
};
use querying::DetailsResults;
use re_ui::{UiExt as _, notifications::NotificationUi};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use strum::IntoEnumIterator;
use tree_view::make_pp_code;
use types::{QueriedLang, SelectedConfig};
use utils_poll::{Buffered3, MultiBuffered2};

mod app_components;
mod code_aspects;
mod code_editor_automerge;
mod code_tracking;
mod commit;
pub(crate) mod crdt_over_ws;
#[allow(unused)]
mod long_tracking;
mod querying;
mod re_ui_collapse;
mod single_repo;
mod smells;
mod tree_view;
mod tsg;
pub(crate) mod types;
mod utils;
mod utils_commit;
mod utils_edition;
mod utils_egui;
mod utils_poll;
mod utils_results_batched;
pub use self::types::Languages;
pub(crate) use app_components::show_repo_menu;
pub(crate) use utils_commit::show_commit_menu;
mod commit_graph;

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(Deserialize, Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct HyperApp {
    /// more like the layout of the app
    /// TODO make it dynamically extensible
    selected: types::SelectedConfig,

    persistance: bool,
    save_interval: std::time::Duration,

    data: AppData,

    layouts: HashMap<String, (Vec<Tab>, egui_tiles::Tree<TabId>)>,

    tree: egui_tiles::Tree<TabId>,
    tabs: Vec<Tab>,
    maximized: Option<TabId>,

    #[serde(skip)]
    cmd_palette: CommandPalette,

    #[serde(skip)]
    /// regular modal
    modal_handler: re_ui::modal::ModalHandler,

    #[serde(skip)]
    /// modal with full span mode
    full_span_modal_handler: re_ui::modal::ModalHandler,

    #[serde(skip)]
    /// modal for project selection
    modal_handler_projects: re_ui::modal::ModalHandler,

    show_left_panel: bool,
    show_right_panel: bool,
    show_bottom_panel: bool,
    bottom_view: BottomPanelConfig,

    dummy_bool: bool,

    latest_cmd: String,

    capture_clip_into_repos: bool,

    // #[serde(skip)]
    // toasts: toasts::Toasts,
    #[serde(skip)]
    notifs: NotificationUi,

    selected_commit: Option<(ProjectId, String)>,
    selected_baseline: Option<String>,
}

#[derive(Deserialize, Serialize, Default, strum_macros::AsRefStr, PartialEq, Eq)]
enum BottomPanelConfig {
    Commits,
    #[default]
    CommitsTime,
    Temporal,
    CommitMetadata,
}

/// See [`querying::QueryContent`]
#[derive(Deserialize, Serialize)]
struct QueryData {
    name: String,
    lang: String,
    query: code_editor::CodeEditor<Languages>,
    results: Vec<QResId>,
    commits: u16,
    max_matches: u64,
    timeout: u64,
    precomp: Option<QueryId>,
}

impl Default for QueryData {
    fn default() -> Self {
        QueryData {
            name: "new".to_string(),
            lang: "Java".to_string(),
            query: code_editor::CodeEditor::new(
                code_editor::EditorInfo::default().copied(),
                r#""#.to_string(),
            ),
            results: vec![],
            commits: 2,
            max_matches: 3000,
            timeout: 1000,
            precomp: None,
        }
    }
}

// TODO plit by repo and query, maybe including config variations... but not commit limit for example
#[derive(Default, Debug)]
struct ResultsPerCommit {
    // prop: cols.len() == floats.len() + ints.len()
    cols: Vec<String>,
    // comp_time and offsets into second level of floats/ints, first level of texts
    map: HashMap<[u8; 8], (f32, u32)>,
    floats: Vec<Vec<f32>>,
    ints: Vec<Vec<i32>>,
    // does not include comp_time as it vary too much
    texts: Vec<Arc<egui::Galley>>,
}

impl ResultsPerCommit {
    fn text(&self, commit: &str) -> Option<&Arc<egui::Galley>> {
        let mut c: [u8; 8] = [0; 8];
        c.copy_from_slice(&commit.as_bytes()[..8]);
        Some(&self.texts[self.map.get(&c)?.1 as usize])
    }

    fn text_with_variation(
        &self,
        commit: &str,
        before: Option<&str>,
        after: Option<&str>,
    ) -> Option<&Arc<egui::Galley>> {
        let offset = self._get_offset(commit)?;
        match (before, after) {
            (Some(before), Some(after)) => {
                let before = self._get_offset(before);
                let after = self._get_offset(after);
                match (before, after) {
                    (Some(before), Some(after)) if offset == after && before == offset => None,
                    _ => Some(&self.texts[offset as usize]),
                }
            }
            _ => Some(&self.texts[offset as usize]),
        }
    }

    fn offset(&self, commit: &str) -> Option<u32> {
        let mut c: [u8; 8] = [0; 8];
        c.copy_from_slice(&commit.as_bytes()[..8]);
        Some(self.map.get(&c)?.1)
    }

    fn offset_with_variation(
        &self,
        commit: &str,
        before: Option<&str>,
        after: Option<&str>,
    ) -> Option<u32> {
        let offset = self._get_offset(commit)?;
        match (before, after) {
            (Some(before), Some(after)) => {
                let before = self._get_offset(before);
                let after = self._get_offset(after);
                match (before, after) {
                    (Some(before), Some(after)) if offset == after && before == offset => None,
                    _ => Some(offset),
                }
            }
            _ => Some(offset),
        }
    }
    fn diff_when_variation(&self, commit: &str, other: &str) -> Option<(u32, u32)> {
        let offset = self._get_offset(commit)?;
        let other = self._get_offset(commit)?;
        if offset != other {
            Some((offset, other))
        } else {
            None
        }
    }

    fn vals_to_string(&self, offset: u32) -> String {
        crate::app::utils::join(self.ints.iter().map(|v| v[offset as usize]), "\n").to_string()
    }

    fn offset_diff_to_string(&self, offset1: u32, offset2: u32) -> String {
        let vals = self
            .ints
            .iter()
            .map(|v| v[offset1 as usize] - v[offset2 as usize]);
        crate::app::utils::join(vals, "\n").to_string()
    }

    fn try_diff_as_string(&self, c1: &str, c2: &str) -> Option<String> {
        let c1 = self._get_offset(c1)?;
        let c2 = self._get_offset(c2)?;
        if c1 == c2 {
            return None;
        }
        let vals = self.ints.iter().map(|v| v[c1 as usize] - v[c2 as usize]);
        let vals = vals.map(|v| format!("{:+}", v));
        let s = crate::app::utils::join(vals, "\n").to_string();
        Some(s)
    }

    fn _get_offset(&self, commit: &str) -> Option<u32> {
        let mut c: [u8; 8] = [0; 8];
        c.copy_from_slice(&commit.as_bytes()[..8]);
        Some(self.map.get(&c)?.1)
    }

    fn get_vals<'a>(&'a self, commit: &str) -> Option<impl ToString + 'a> {
        self.text(commit).map(|x| &x.job.text)
    }

    /// true if columns did not change
    fn set_cols(&mut self, h: &[String]) -> bool {
        if self.cols.is_empty() {
            self.ints = vec![vec![]; h.len()];
            self.cols = h.to_vec();
            // TODO init also for floats
            return false;
        }
        if self.cols != h {
            // for now reset all data, and replace cols
            log::warn!("{:?} {:?}", self.cols, h);
            self.ints = vec![vec![]; h.len()];
            self.texts = vec![];
            self.cols = h.to_vec();
        }
        true
    }

    fn insert(
        &mut self,
        commit: &str,
        // galley: impl Fn() -> Arc<egui::Galley>,
        comp_time: f32,
        floats: &[f32],
        ints: &[i32],
    ) {
        let mut c: [u8; 8] = [0; 8];
        c.copy_from_slice(&commit.as_bytes()[..8]);
        match self.map.entry(c) {
            std::collections::hash_map::Entry::Occupied(mut occ) => {
                let (t, v) = occ.get_mut();
                *t = comp_time;
                let i = *v as usize;
                let mut ident = true;
                for j in 0..ints.len() {
                    if !ident {
                        break;
                    }
                    ident &= self.ints[j][i] == ints[j];
                }
                for j in 0..floats.len() {
                    if !ident {
                        break;
                    }
                    // pretty dangerous but necessary due to prerender
                    // anyway should be deterministic
                    // and it depends on data, just do not put it in,
                    // then setting opt out of ser for unstable values could be useful
                    ident &= self.floats[j][i] == floats[j];
                }
                if !ident {
                    // TODO gc unused ints and floats
                    match Self::find_vals(&mut self.ints, &mut self.floats, ints, floats) {
                        Ok(i) => {
                            *v = i as u32;
                        }
                        Err(len) => {
                            *v = len as u32;
                            for j in 0..ints.len() {
                                self.ints[j].push(ints[j]);
                            }
                            for j in 0..floats.len() {
                                self.floats[j].push(floats[j]);
                            }
                        }
                    }
                }
            }
            std::collections::hash_map::Entry::Vacant(vac) => {
                match Self::find_vals(&mut self.ints, &mut self.floats, ints, floats) {
                    Ok(i) => {
                        vac.insert((comp_time, i as u32));
                    }
                    Err(i) => {
                        vac.insert((comp_time, i as u32));
                        for j in 0..ints.len() {
                            self.ints[j].push(ints[j]);
                        }
                        for j in 0..floats.len() {
                            self.floats[j].push(floats[j]);
                        }
                    }
                }
            }
        }
    }

    fn find_vals(
        s_ints: &mut Vec<Vec<i32>>,
        s_floats: &mut Vec<Vec<f32>>,
        ints: &[i32],
        floats: &[f32],
    ) -> Result<usize, usize> {
        let len = s_ints[0].len();
        // TODO impl the complete logic
        for i in 0..len {
            let mut ident = true;
            for j in 0..ints.len() {
                if !ident {
                    break;
                }
                ident &= s_ints[j][i] == ints[j];
            }
            for j in 0..floats.len() {
                if !ident {
                    break;
                }
                // pretty dangerous but necessary due to prerendering of text.
                // anyway should be deterministic
                // and it depends on data, just do not put it in,
                // then setting opt out of ser for unstable values could be useful
                ident &= s_floats[j][i] == floats[j];
            }
            if ident {
                return Ok(i);
            }
        }
        Err(len)
    }
}

#[derive(Deserialize, Serialize)]
struct QueryResults {
    project: ProjectId,
    query: QueryId,
    content: utils_poll::Buffered2<
        Result<querying::StreamedComputeResults, querying::QueryingError>,
        Result<querying::StreamedComputeResults, querying::QueryingError>,
    >,
    // Buffered5<,
    // utils_results_batched::ComputeResults, querying::QueryingError
    // >,
    tab: TabId,
}
type CommitMdPayload = (commit::CommitMetadata, Option<(Commit, ProjectId)>);

type CommitMdStore = MultiBuffered2<
    types::CommitId,
    Result<CommitMdPayload, String>,
    Result<commit::CommitMetadata, String>,
>;

#[derive(Deserialize, Serialize)]
#[serde(default)]
pub(crate) struct AppData {
    api_addr: String,

    max_fetch: i64,
    offset_fetch: i64,

    scripting_context: single_repo::ScriptingContext,
    querying_context: querying::QueryingContext,
    tsg_context: tsg::TsgContext,
    smells_context: smells::Context,

    code_views: Vec<CodeView>,
    queries: Vec<QueryData>,
    queries_results: Vec<QueryResults>,
    queries_differential_results: Option<(
        ProjectId,
        QueryId,
        Buffered3<Result<DetailsResults, querying::QueryingError>>,
        TabId,
        u64, // hash of query and selected_baseline
    )>,
    // #[serde(skip)]
    // results_per_commit: ResultsPerCommit,
    #[serde(skip)]
    languages: Languages,
    single: Sharing<ComputeConfigSingle>,
    query: Sharing<querying::ComputeConfigQuery>,
    tsg: Sharing<tsg::ComputeConfigQuery>,
    smells: smells::Config,
    multi: types::ComputeConfigMulti,
    diff: types::ComputeConfigDiff,
    tracking: types::ComputeConfigTracking,
    aspects: types::ComputeConfigAspectViews,

    #[serde(skip)]
    compute_single_result: Option<utils_results_batched::RemoteResult<single_repo::ScriptingError>>,
    #[serde(skip)]
    querying_result: Option<utils_results_batched::RemoteResult<querying::QueryingError>>,
    #[serde(skip)]
    tsg_result: Option<utils_results_batched::RemoteResult<tsg::QueryingError>>,
    #[serde(skip)]
    smells_result: Option<smells::RemoteResult>,
    #[serde(skip)]
    smells_diffs_result: Option<smells::RemoteResultDiffs>,

    #[serde(skip)]
    fetched_files: HashMap<types::FileIdentifier, code_tracking::RemoteFile>,
    #[serde(skip)]
    fetched_files2: HashMap<
        types::FileIdentifier,
        poll_promise::Promise<Result<hyperast::store::nodes::fetched::NodeIdentifier, String>>,
    >,
    #[serde(skip)]
    fetched_commit: HashMap<Commit, code_aspects::RemoteView>,

    // TODO just use the oid as key...
    fetched_commit_metadata: CommitMdStore,
    #[serde(skip)]
    tracking_result: utils_poll::Buffered<code_tracking::RemoteResult>,
    #[serde(skip)]
    aspects_result: Option<code_aspects::RemoteView>,
    #[serde(skip)]
    store: Arc<FetchedHyperAST>,

    long_tracking: long_tracking::LongTacking,

    selected_code_data: SelectedProjects,

    /// Commands to run at the end of the frame.
    #[serde(skip)]
    pub command_sender: CommandSender,
    #[serde(skip)]
    command_receiver: CommandReceiver,

    /// cache for stuff that are still too WIP
    /// avoid churn in app data and reduce uses of locking structures.
    /// Avoid storing common/pub types and transfer data between modules, prefer to add a new attribute otherwise.
    #[serde(skip)]
    misc_cache: HashMap<(std::any::TypeId, u64), Box<dyn std::any::Any>>,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(untagged)]
pub(super) enum LocalOrRemote<R: std::marker::Send + 'static> {
    #[serde(skip)]
    Remote(utils_results_batched::Remote<R>),
    Local(R),
    #[default]
    None,
}

impl Default for AppData {
    fn default() -> Self {
        // 20 days
        const MAX_FETCH: i64 = 60 * 60 * 24 * 20;
        let (command_sender, command_receiver) = crate::command::command_channel();
        Self {
            api_addr: "".to_string(),
            max_fetch: MAX_FETCH,
            offset_fetch: 0,
            selected_code_data: Default::default(),
            scripting_context: Default::default(),
            querying_context: Default::default(),
            tsg_context: Default::default(),
            smells_context: Default::default(),
            languages: Default::default(),
            single: Default::default(),
            query: Default::default(),
            tsg: Default::default(),
            smells: Default::default(),
            diff: Default::default(),
            multi: Default::default(),
            tracking: Default::default(),
            code_views: vec![
                CodeView {
                    commit: Default::default(),
                    file_path: Default::default(),
                    root: Default::default(),
                    path: Default::default(),
                    prefill_cache: Default::default(),
                    generation: Default::default(),
                },
                CodeView {
                    commit: Default::default(),
                    file_path: Some("src/types.h".to_string()),
                    root: Default::default(),
                    path: Default::default(),
                    prefill_cache: Default::default(),
                    generation: Default::default(),
                },
            ],
            queries: vec![QueryData {
                name: "simple".to_string(),
                lang: "Java".to_string(),
                query: code_editor::CodeEditor::new(
                    code_editor::EditorInfo::default().copied(),
                    r#"(try_statement
  (block
    (expression_statement
      (method_invocation
        (identifier) (#EQ? "fail")
      )
    )
  )
  (catch_clause)
)
(class_declaration
  body: (_
    (method_declaration
      (modifiers
        (marker_annotation
          name: (_) (#EQ? "Test")
        )
      )
    )
  )
)
"#
                    .to_string(),
                ),
                ..Default::default()
            }],
            queries_results: vec![],
            queries_differential_results: None,
            compute_single_result: Default::default(),
            querying_result: Default::default(),
            // results_per_commit: Default::default(),
            tsg_result: Default::default(),
            smells_result: Default::default(),
            smells_diffs_result: Default::default(),
            fetched_files: Default::default(),
            fetched_files2: Default::default(),
            fetched_commit: Default::default(),
            fetched_commit_metadata: Default::default(),
            tracking_result: Default::default(),
            aspects: Default::default(),
            aspects_result: Default::default(),
            long_tracking: Default::default(),
            store: Default::default(),
            command_sender,
            command_receiver,
            misc_cache: Default::default(),
        }
    }
}

pub(crate) use commit::ProjectId;
type LocalQueryId = u16;
type QueryId = u16;
type DiffId = usize;
type RemCodeId = usize;
type RemTreeId = usize;
type QResId = u16;

#[derive(Deserialize, Serialize, PartialEq, Eq)]
enum Tab {
    RemoteQuery(QueryId),
    LocalQuery(LocalQueryId),
    Diff(DiffId),
    CodeTree(RemTreeId),
    CodeFile(RemCodeId),
    MarkdownEdit(String),
    MarkdownStatic(usize),
    ProjectSelection(),
    LongTracking,
    Smells,
    TSG,
    TreeAspect,
    CodeAspect,
    Empty,
    Commits,
    Querying,
    QueryResults { id: QResId, format: ResultFormat },
}

#[derive(Deserialize, Serialize, strum_macros::AsRefStr, PartialEq, Eq)]
enum ResultFormat {
    Table,
    List,
    Json,
    Hunks,
}

impl Tab {
    fn title(&self, data: &AppData) -> egui::WidgetText {
        match self {
            Tab::RemoteQuery(_) => "Query".into(),
            Tab::LocalQuery(id) => {
                let name = &data.queries[*id as usize].name;
                format!("Local Query: {name}").into()
            }
            Tab::Diff(_) => "Diff".into(),
            Tab::CodeTree(_) => "Remote Tree".into(),
            Tab::CodeFile(_) => "Remote Code".into(),
            Tab::QueryResults { id, .. } => format!("Query Results {id}").into(),
            Tab::ProjectSelection() => "Projects Selection".into(),
            Tab::Commits => "Commits".into(),
            Tab::MarkdownStatic(_) => "Markdown View".into(),
            Tab::MarkdownEdit(_) => "Markdown Edit".into(),
            Tab::Smells => "Smells".into(),
            Tab::LongTracking => "Tracking".into(),
            Tab::TSG => "TSG".into(),
            Tab::Querying => "Querying".into(),
            Tab::TreeAspect => "Tree Aspect".into(),
            Tab::CodeAspect => "Code Aspect".into(),
            Tab::Empty => unreachable!(),
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)]
pub(crate) struct Sharing<T> {
    pub(crate) content: T,
    #[serde(skip)]
    rt: crdt_over_ws::Rt, // TODO do not init
    #[serde(skip)]
    ws: Option<crdt_over_ws::WsDoc>,
    #[serde(skip)]
    doc_db: Option<crdt_over_ws::WsDocsDb>,
}

impl SelectedConfig {
    fn default_layout(&self) -> Vec<Tab> {
        match self {
            SelectedConfig::Single => vec![Tab::Commits],
            SelectedConfig::Querying => vec![
                // Tab::MarkdownEdit(DEFAULT_EXPLAINATIONS_MDS[0].to_string()),
                Tab::ProjectSelection(),
                // Tab::MarkdownStatic(0),
                // Tab::LongTracking,
                // // Tab::Smells,
                // Tab::Diff(0),
                // Tab::TreeAspect,
                // Tab::CodeTree(0),
                // Tab::CodeFile(1),
                // Tab::Commits,
                Tab::LocalQuery(0),
                // Tab::QueryResults {
                //     id: 0,
                //     format: ResultFormat::Json,
                // },
            ],
            SelectedConfig::Tsg => vec![Tab::TSG],
            SelectedConfig::Smells => vec![Tab::Smells],
            SelectedConfig::Multi => vec![Tab::Commits],
            SelectedConfig::Diff => vec![Tab::Diff(0)],
            SelectedConfig::Tracking => vec![Tab::CodeTree(0)],
            SelectedConfig::LongTracking => vec![Tab::LongTracking],
            SelectedConfig::Aspects => vec![Tab::TreeAspect],
        }
    }
}

impl Default for HyperApp {
    fn default() -> Self {
        let selected = SelectedConfig::default();
        let tabs = selected.default_layout();
        Self {
            // Example stuff:
            selected,
            persistance: false,
            save_interval: std::time::Duration::from_secs(20),
            data: Default::default(),
            layouts: HashMap::default(),
            tree: egui_tiles::Tree::new_grid("my_tree", (0..tabs.len() as u16).collect()),
            tabs,
            maximized: Default::default(),
            cmd_palette: CommandPalette::default(),
            modal_handler: Default::default(),
            full_span_modal_handler: Default::default(),
            modal_handler_projects: Default::default(),
            show_left_panel: true,
            show_right_panel: true,
            show_bottom_panel: true,
            bottom_view: BottomPanelConfig::default(),
            dummy_bool: true,
            latest_cmd: Default::default(),
            capture_clip_into_repos: false,
            notifs: Default::default(),
            selected_commit: None,
            selected_baseline: None,
        }
    }
}

const DEFAULT_EXPLAINATIONS_MDS: &[&str] = &[r#"# Graphical Interface of the HyperAST

You are using the GUI of the HyperAST.
The HyperAST enables developpers and researchers alike to explore and investigate
temporal code evolutions in the repositories of their choice.

Readily supports projects using Java with Maven, and simple C/C++ (Makefile in root and an src/ dir).
Other codebase structures, languages and build systems could be added, but time is lacking for now.

## Default Layouts

To kickstart you HyperAST journey,
we provide provide a few layouts and their associated examples.

"#];

impl HyperApp {
    /// Called once before the first frame.
    #[cfg(target_arch = "wasm32")]
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        languages: Languages,
        api_addr: Option<String>,
        default_api_addr: &str,
    ) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        dbg!();

        crate::platform::init_nat_menu();
        egui_extras::install_image_loaders(&cc.egui_ctx);

        use wasm_bindgen::prelude::*;
        #[wasm_bindgen]
        extern "C" {
            fn prompt(text: &str, default: &str) -> String;
        }

        let mut r: HyperApp;
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            if let Some(s) = storage.get_string(eframe::APP_KEY) {
                match serde_json::from_str(&s) {
                    Ok(_r) => {
                        r = _r;
                    }
                    Err(err) => {
                        wasm_rs_dbg::dbg!(storage.get_string(eframe::APP_KEY));
                        log::debug!("Failed to decode RON: {err}");
                        r = HyperApp::default();
                    }
                }
            } else {
                r = HyperApp::default()
            }
            // r = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            // if r.persistance == false {
            //     panic!();
            // }
            // if !r.persistance {
            //     r = HyperApp::default()
            // }
            if r.data.api_addr.is_empty() {
                r.data.api_addr = unsafe { prompt("API address", default_api_addr) };
            }
        } else {
            r = HyperApp::default();
            if let Some(api_addr) = api_addr {
                r.data.api_addr = api_addr;
            } else {
                r.data.api_addr = unsafe { prompt("API addresss", default_api_addr) };
            }
        }
        r.data.languages = languages;
        r
    }

    /// Called once before the first frame.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn new(cc: &eframe::CreationContext<'_>, languages: Languages, api_addr: String) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        dbg!();

        // _cc.egui_ctx.set_fonts(font_definitions);

        crate::platform::init_nat_menu();
        egui_extras::install_image_loaders(&cc.egui_ctx);

        let mut r: HyperApp;
        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            if let Some(s) = storage.get_string(eframe::APP_KEY) {
                match serde_json::from_str(&s) {
                    Ok(_r) => {
                        r = _r;
                    }
                    Err(err) => {
                        wasm_rs_dbg::dbg!(storage.get_string(eframe::APP_KEY));
                        log::debug!("Failed to decode RON: {err}");
                        r = HyperApp::default();
                    }
                }
            } else {
                r = HyperApp::default()
            }
            // r = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
            if !r.persistance {
                r = HyperApp::default()
            }
            if r.data.api_addr.is_empty() {
                r.data.api_addr = api_addr;
            }
        } else {
            r = HyperApp::default();
            r.data.api_addr = api_addr;
        }
        r.data.languages = languages;
        r
    }
}

#[derive(Deserialize, Serialize)]
struct CodeView {
    commit: Commit,
    file_path: Option<String>,
    root: Option<hyperast::store::nodes::fetched::NodeIdentifier>,
    path: Vec<u16>,
    #[serde(skip)]
    prefill_cache: Option<tree_view::PrefillCache>,
    // to gc weak fields
    generation: u64,
}

type TabId = u16;

struct MyTileTreeBehavior<'a> {
    data: &'a mut AppData,
    tabs: &'a mut Vec<Tab>,
    maximized: &'a mut Option<TabId>,
    edited: bool,
    selected_commit: &'a mut Option<(ProjectId, String)>,
    selected_baseline: &'a mut Option<String>,
    to_hide: Vec<egui_tiles::TileId>,
    tab_to_add: Option<TabId>,
}

impl<'a> egui_tiles::Behavior<TabId> for MyTileTreeBehavior<'a> {
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        pane: &mut TabId,
    ) -> egui_tiles::UiResponse {
        match &mut self.tabs[*pane as usize] {
            Tab::Commits => {
                egui::ScrollArea::both()
                    .auto_shrink([false; 2])
                    .stick_to_bottom(true)
                    .show(ui, |ui| {
                        egui::Frame {
                            inner_margin: egui::Margin::same(re_ui::DesignTokens::view_padding()),
                            ..Default::default()
                        }
                        .show(ui, |ui| self.show_commits(ui));
                    });
                Default::default()
            }
            Tab::LocalQuery(id) => {
                let query = &mut self.data.queries[*id as usize];
                ui.painter().rect_filled(
                    ui.available_rect_before_wrap(),
                    ui.visuals().window_corner_radius,
                    ui.visuals().extreme_bg_color,
                );

                let code = &mut query.query.code;
                let language = "rs";
                let theme =
                    egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

                const EDIT_AWARE: bool = false;
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        if EDIT_AWARE {
                            // some issues on cursor behavior, like lising focus on arrow key press
                            use egui_addon::code_editor::generic_text_edit::TextEdit;
                            ui.add_sized(
                                ui.available_size(),
                                TextEdit::multiline(code)
                                    .code_editor()
                                    .frame(false)
                                    .desired_width(f32::INFINITY)
                                    .layouter(&mut |ui, string, _wrap_width| {
                                        let layout_job =
                                            egui_extras::syntax_highlighting::highlight(
                                                ui.ctx(),
                                                ui.style(),
                                                &theme,
                                                string.as_str(),
                                                language,
                                            );
                                        ui.fonts(|f| f.layout_job(layout_job))
                                    }),
                            );
                        } else {
                            ui.add_sized(
                                ui.available_size(),
                                egui::TextEdit::multiline(&mut code.string)
                                    .code_editor()
                                    .frame(false)
                                    .desired_width(f32::INFINITY)
                                    .layouter(&mut |ui, string, _wrap_width| {
                                        let layout_job =
                                            egui_extras::syntax_highlighting::highlight(
                                                ui.ctx(),
                                                ui.style(),
                                                &theme,
                                                string.as_ref(),
                                                language,
                                            );
                                        ui.fonts(|f| f.layout_job(layout_job))
                                    }),
                            );
                        }
                    });

                // egui_addon::code_editor::show_edit_syntect(ui, &mut query.query.code);

                // let api_endpoint = &querying::end_point(&self.data.api_addr);
                // update_shared_editors(ui, query, api_endpoint, query_editors);
                // querying::show_scripts_edition(
                //     ui,
                //     api_endpoint,
                //     &mut self.data.querying_context,
                //     &mut self.data.query,
                // );

                Default::default()
            }
            Tab::RemoteQuery(id) => {
                // update_shared_editors(ui, query, api_endpoint, query_editors);
                if let Some(promise) = &mut self.data.smells_result {
                    match promise.ready() {
                        Some(Ok(res)) => match &res.content {
                            Some(Ok(res)) => {
                                if let Some(query_bad) = &res.bad.get(*id as usize) {
                                    smells::show_query(query_bad, ui);
                                }
                            }
                            Some(Err(err)) => {
                                ui.label(serde_json::to_string_pretty(err).unwrap());
                            }
                            None => {
                                ui.label("no content");
                            }
                        },
                        Some(Err(err)) => {
                            ui.label(err);
                        }
                        None => {
                            ui.label("prending smells result promise");
                        }
                    }
                }
                Default::default()
            }
            Tab::Diff(i) => {
                if let Some(examples) = &mut self.data.smells.diffs {
                    if let Some(example) = examples.examples.get_mut(*i) {
                        smells::show_diff(
                            ui,
                            &mut self.data.api_addr,
                            example,
                            &mut self.data.fetched_files,
                        );
                    } else {
                        ui.label("no diffs available");
                    }
                } else {
                    ui.label("no diffs available");
                }
                Default::default()
            }
            Tab::CodeFile(id) => {
                let code_view = &mut self.data.code_views[*id];
                code_view.generation = ui.ctx().cumulative_pass_nr();
                let commit = &code_view.commit;
                let Some(file_path) = &code_view.file_path else {
                    ui.error_label("no file path");
                    return Default::default();
                };
                let file = types::FileIdentifier {
                    commit: commit.clone(),
                    file_path: file_path.clone(),
                };

                let nid = if let Some(root) = &mut code_view.root {
                    root
                } else if let Some(prom) = self.data.fetched_files2.get_mut(&file) {
                    let Some(Ok(res)) = prom.ready_mut() else {
                        return Default::default();
                    };
                    code_view.root = Some(*res);
                    code_view.root.as_mut().unwrap()
                } else {
                    assert!(!file_path.contains(":"));
                    let v = remote_fetch_node(
                        ui.ctx(),
                        &self.data.api_addr,
                        self.data.store.clone(),
                        commit,
                        &format!("{}:", file_path),
                    );
                    self.data.fetched_files2.insert(file, v);
                    return Default::default();
                };

                // ui.label(nid.to_string());

                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show_viewport(ui, |ui, _viewport| {
                        let theme = egui_addon::syntax_highlighting::simple::CodeTheme::from_memory(
                            ui.ctx(),
                        );
                        let layout_job =
                            make_pp_code(self.data.store.clone(), ui.ctx(), *nid, theme);
                        let galley = ui.fonts(|f| f.layout_job(layout_job));
                        let size = galley.size();
                        let (rect, resp) = ui.allocate_exact_size(size, egui::Sense::click());
                        ui.painter_at(rect.expand(1.0)).galley(
                            rect.min,
                            galley,
                            egui::Color32::RED,
                        );
                        resp
                    });
                Default::default()
            }
            Tab::CodeTree(id) => {
                let code_view = &mut self.data.code_views[*id];
                code_view.generation = ui.ctx().cumulative_pass_nr();
                let commit = &code_view.commit;
                let path = &code_view.path;

                let root = if let Some(aspects_result) = &mut code_view.root {
                    aspects_result
                } else if let Some(prom) = self.data.fetched_commit.get_mut(commit) {
                    let Some(Ok(aspects_result)) = prom.ready_mut() else {
                        return Default::default();
                    };
                    let root = aspects_result.content.as_ref().unwrap().root;
                    code_view.root = Some(root);
                    code_view.root.as_mut().unwrap()
                } else {
                    self.data.fetched_commit.insert(
                        commit.clone(),
                        code_aspects::remote_fetch_node_old(
                            ui.ctx(),
                            &self.data.api_addr,
                            self.data.store.clone(),
                            commit,
                            "", //&utils::join(path.iter(), "/").to_string(),
                        ),
                    );
                    return Default::default();
                };
                egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show_viewport(ui, |ui, _viewport| {
                        ui.set_height(3_000.0);
                        {
                            let mut imp = tree_view::FetchedViewImpl::new(
                                self.data.store.clone(),
                                &self.data.aspects,
                                code_view.prefill_cache.take(),
                                vec![],
                                None,
                                path.iter().map(|x| *x as usize).collect(),
                                ui.id(),
                                None,
                                None,
                            );
                            let r = imp.show(ui, &self.data.api_addr, &root);
                            // wasm_rs_dbg::dbg!(&imp);
                            code_view.prefill_cache = imp.prefill_cache;
                            r
                        };
                    });
                Default::default()
            }
            Tab::QueryResults { id, format } => {
                let Some(QueryResults {
                    project: proj_id,
                    query: _,
                    content: res,
                    tab: _,
                }) = self.data.queries_results.get_mut(*id as usize)
                else {
                    ui.error_label(&format!("{} is not in the list of queries", id));
                    return Default::default();
                };
                let Some(res) = res.get_mut() else {
                    if res.try_poll_with(|x| {
                        todo!()
                        // x.expect("TODO").content.expect("TODO").map(|x| x.into())
                    }) {
                        // TODO is there something to do ?
                    } else {
                        ui.spinner();
                    }
                    // let take = std::mem::take(res);
                    // match take {
                    //     LocalOrRemote::Remote(prom) => {
                    //         match prom.try_take() {
                    //             Ok(Ok(r)) => {
                    //                 if let Some(r) = r.content {
                    //                     *res = LocalOrRemote::Local(r);
                    //                 }
                    //             }
                    //             Ok(Err(err)) => {
                    //                 ui.error_label(&format!("error: {}", err));
                    //             }
                    //             Err(prom) => {
                    //                 *res = LocalOrRemote::Remote(prom);
                    //                 ui.spinner();
                    //             }
                    //         };
                    //     }
                    //     LocalOrRemote::None => {
                    //         ui.error_label(&format!("no results for {}", id));
                    //     }
                    //     _ => (),
                    // }
                    return Default::default();
                };

                let res = match res {
                    Ok(res) => res,
                    Err(err) => {
                        ui.error_label(&format!("error {:?}", err));
                        return Default::default();
                    }
                };
                match format {
                    ResultFormat::Table => {
                        let mut selected_commit = None;
                        if let Some(selected) = &self.selected_commit {
                            if selected.0 == *proj_id {
                                let id = egui::Id::new(proj_id);
                                let i = ui.data_mut(|w| {
                                    let m: &mut (Option<(ProjectId, types::CommitId)>, usize) =
                                        w.get_temp_mut_or_default(id);
                                    if m.0.as_ref() != Some(selected) {
                                        m.0 = Some(selected.clone());
                                        m.1 = res
                                            // .results
                                            .rows
                                            .lock()
                                            .unwrap()
                                            .1
                                            .iter()
                                            .position(|x| {
                                                x.as_ref().map_or(false, |r| r.commit == selected.1)
                                            })
                                            .unwrap_or(usize::MAX);
                                        log::debug!("{:?}", m);
                                        Some(m.1.clone())
                                    } else {
                                        None
                                    }
                                });
                                selected_commit = i;
                            }
                        }
                        ui.push_id("table", |ui| {
                            utils_results_batched::show_long_result_table(
                                ui,
                                (&res.head, None, res.rows.lock().unwrap().1.as_slice()),
                                &mut selected_commit,
                                |cid| {
                                    self.data
                                        .fetched_commit_metadata
                                        .get(cid)?
                                        .as_ref()
                                        .ok()?
                                        .message
                                        .clone()
                                },
                            )
                        });
                    }
                    ResultFormat::Hunks => {
                        let oid = &self.selected_commit.as_ref().unwrap().1;
                        ui.label(oid);
                        let Some(selected_baseline) = &self.selected_baseline else {
                            *format = ResultFormat::Table;
                            return Default::default();
                        };
                        ui.label(format!("baseline: {}", selected_baseline));
                        if let Some(msg) = self
                            .data
                            .fetched_commit_metadata
                            .get(oid)
                            .and_then(|x| x.as_ref().ok())
                            .and_then(|x| x.message.as_ref())
                        {
                            ui.label("message: ");
                            egui::Frame::group(ui.style()).show(ui, |ui| {
                                let mut msg_lines = msg.lines();
                                let mut i = 0;
                                while let Some(t) = msg_lines.next() {
                                    ui.label(t);
                                    i += 1;
                                    if i == 3 {
                                        let rem = msg_lines.count();
                                        if rem > 0 {
                                            ui.weak(format!("... ({rem} rem. lines)"));
                                        }
                                        break;
                                    }
                                }
                            });
                        }

                        let qid = 0; //q_res.1 as usize;
                        if let Some(differential) = &mut self.data.queries_differential_results {
                            if differential.2.is_waiting() {
                                ui.spinner();
                            }
                            differential.2.try_poll_with(|x| {
                                x.map_err(|e| querying::QueryingError::NetworkError(e))
                                    .and_then(|x| x.content.unwrap())
                            });
                            if !differential.2.is_present() && !differential.2.is_waiting() {
                                self.data.queries_differential_results = None;
                            } else if let Some(Err(_)) = differential.2.get() {
                                self.data.queries_differential_results = None;
                            } else if hash((
                                self.data.queries[qid].query.as_ref(),
                                selected_baseline.clone(),
                            )) != differential.4
                            {
                                self.data.queries_differential_results = None;
                            } else if let Some(Ok(x)) = differential.2.get() {
                                const B: f32 = 15.;
                                const H: f32 = 800.;
                                let id = ui.id();
                                let len = x.results.len();
                                egui::ScrollArea::vertical()
                                    .scroll_bar_visibility(
                                        egui::scroll_area::ScrollBarVisibility::AlwaysVisible,
                                    )
                                    .show_rows(ui, H, len, |ui, cols| {
                                        let (mut rect, _) = ui.allocate_exact_size(
                                            egui::Vec2::new(
                                                ui.available_width(),
                                                H * (cols.end - cols.start) as f32,
                                            ),
                                            egui::Sense::hover(),
                                        );
                                        let top = rect.top();
                                        for i in cols.clone() {
                                            let mut rect = {
                                                let (t, b) = rect.split_top_bottom_at_y(
                                                    top + H * (i - cols.start + 1) as f32,
                                                );
                                                rect = b;
                                                t
                                            };
                                            use std::ops::SubAssign;
                                            rect.bottom_mut().sub_assign(B);
                                            let line_pos_1 =
                                                egui::emath::GuiRounding::round_to_pixels(
                                                    rect.left_bottom(),
                                                    ui.painter().pixels_per_point(),
                                                );
                                            let line_pos_2 =
                                                egui::emath::GuiRounding::round_to_pixels(
                                                    rect.right_bottom(),
                                                    ui.painter().pixels_per_point(),
                                                );
                                            ui.painter().line_segment(
                                                [line_pos_1, line_pos_2],
                                                ui.visuals().window_stroke(),
                                            );
                                            rect.bottom_mut().sub_assign(B);
                                            let mut ui = ui.new_child(
                                                egui::UiBuilder::new().max_rect(rect).layout(
                                                    egui::Layout::top_down(egui::Align::Min),
                                                ),
                                            );
                                            ui.set_clip_rect(rect.intersect(ui.clip_rect()));
                                            ui.label(format!(
                                                "{}:{}..{}",
                                                x.results[i].0.file.file_path,
                                                x.results[i].0.range.as_ref().unwrap().start,
                                                x.results[i].0.range.as_ref().unwrap().end
                                            ));
                                            ui.push_id(id.with(i).with(&x.results[i]), |ui| {
                                                let after = x.results[i].1.clone();
                                                assert_eq!(
                                                    after.file.commit.id,
                                                    self.selected_commit.as_ref().unwrap().1
                                                );
                                                // after.file.commit.id = self
                                                //     .selected_commit
                                                //     .as_ref()
                                                //     .unwrap()
                                                //     .1
                                                //     .clone();
                                                smells::show_diff(
                                                    ui,
                                                    &self.data.api_addr,
                                                    &smells::ExamplesValue {
                                                        before: x.results[i].0.clone(),
                                                        after,
                                                        inserts: Default::default(),
                                                        deletes: Default::default(),
                                                        moves: Default::default(),
                                                    },
                                                    &mut self.data.fetched_files,
                                                );
                                            });
                                        }
                                    });
                            }
                        } else {
                            let pid: ProjectId = self.selected_commit.as_ref().unwrap().0;
                            if pid != *proj_id {
                                return Default::default();
                            }
                            let (repo, mut c) = self.data.selected_code_data.get_mut(pid).unwrap();
                            let query = self.data.queries[qid].query.as_ref().to_string();
                            wasm_rs_dbg::dbg!(&query);
                            let config = types::Config::MakeCpp;
                            let language = config.language().to_string();
                            let commits = 2;
                            let baseline = Commit {
                                repo: repo.clone(),
                                id: self.selected_baseline.as_ref().unwrap().clone(),
                            };
                            let commit = Commit {
                                repo: repo.clone(),
                                id: self.selected_commit.as_ref().unwrap().1.clone(),
                            };
                            let max_matches = self.data.queries[qid].max_matches;
                            let timeout = self.data.queries[qid].timeout;
                            let precomp = self.data.queries[qid]
                                .precomp
                                .clone()
                                .map(|id| &self.data.queries[id as usize]);
                            let precomp = precomp.map(|p| p.query.as_ref().to_string());
                            let hash =
                                hash((&query, self.selected_baseline.as_ref().unwrap().clone()));
                            let prom = querying::remote_compute_query_differential(
                                ui.ctx(),
                                &self.data.api_addr,
                                &querying::ComputeConfigQueryDifferential {
                                    commit,
                                    config,
                                    baseline,
                                },
                                querying::QueryContent {
                                    language,
                                    query,
                                    precomp,
                                    commits,
                                    max_matches,
                                    timeout,
                                },
                            );
                            self.data.queries_differential_results =
                                Some((pid, 0, Default::default(), *pane, hash));
                            self.data
                                .queries_differential_results
                                .as_mut()
                                .unwrap()
                                .2
                                .buffer(prom);
                        }
                        // todo!("horizontal commits ; vertial hunks (per commit)");
                        // TODO integrate with tracking
                    }
                    ResultFormat::List => {
                        todo!()
                        // utils_results_batched::show_long_result_list(ui, res);
                    }
                    ResultFormat::Json => todo!(),
                }
                Default::default()
            }
            Tab::ProjectSelection() => {
                egui::Frame::NONE
                    .outer_margin(egui::Margin::same(5))
                    .inner_margin(egui::Margin::same(15))
                    .show(ui, |ui| {
                        show_project_selection(ui, &mut self.data);

                        let data = &mut self.data;
                        ui.center("project_top_left_actions", |ui| {
                            show_projects_actions(ui, data)
                        })
                    });
                Default::default()
            }
            Tab::TreeAspect => {
                if let Some(aspects_result) = &mut self.data.aspects_result {
                    code_aspects::show(
                        aspects_result,
                        ui,
                        &mut self.data.api_addr,
                        &mut self.data.aspects,
                    );
                } else {
                    self.data.aspects_result = Some(code_aspects::remote_fetch_node_old(
                        ui.ctx(),
                        &self.data.api_addr,
                        self.data.store.clone(),
                        &self.data.aspects.commit,
                        &self.data.aspects.path,
                    ));
                }
                Default::default()
            }
            Tab::CodeAspect => {
                ui.error_label("TODO");
                Default::default()
            }
            Tab::Smells => {
                ui.set_clip_rect(ui.available_rect_before_wrap());
                smells::show_central_panel(
                    ui,
                    &mut self.data.api_addr,
                    &mut self.data.smells,
                    &mut (),
                    &mut false,
                    &mut self.data.smells_result,
                    &mut self.data.smells_diffs_result,
                    &mut self.data.fetched_files,
                );
                Default::default()
            }
            Tab::TSG => {
                ui.set_clip_rect(ui.available_rect_before_wrap());
                let mut trigger = false;
                tsg::show_querying(
                    ui,
                    &self.data.api_addr,
                    &mut self.data.tsg,
                    &mut self.data.tsg_context,
                    &mut trigger,
                    &mut self.data.tsg_result,
                );
                if trigger {
                    self.data.tsg_result = Some(tsg::remote_compute_query(
                        ui.ctx(),
                        &self.data.api_addr,
                        &mut self.data.tsg,
                        &mut self.data.tsg_context,
                    ));
                }
                Default::default()
            }
            Tab::LongTracking => {
                long_tracking::show_results(
                    ui,
                    &self.data.api_addr,
                    &mut self.data.aspects,
                    self.data.store.clone(),
                    &mut self.data.long_tracking,
                    &mut self.data.fetched_files,
                );
                Default::default()
            }
            Tab::Querying => {
                let mut trigger_compute = false;
                querying::show_querying(
                    ui,
                    &self.data.api_addr,
                    &mut self.data.query,
                    &mut self.data.querying_context,
                    &mut trigger_compute,
                    &mut self.data.querying_result,
                );
                if trigger_compute {
                    self.data.querying_result = Some(querying::remote_compute_query(
                        ui.ctx(),
                        &self.data.api_addr,
                        &self.data.query,
                        &mut self.data.querying_context,
                    ));
                }
                Default::default()
            }
            Tab::MarkdownStatic(md) => {
                use epaint::mutex::Mutex;

                let ui = (&mut *ui).ui_mut();
                let commonmark_cache = ui.data_mut(|data| {
                    data.get_temp_mut_or_default::<std::sync::Arc<Mutex<egui_commonmark::CommonMarkCache>>>(
                        egui::Id::new("global_egui_commonmark_cache"),
                    )
                    .clone()
                });

                egui_commonmark::CommonMarkViewer::new().show_scrollable(
                    tile_id,
                    ui,
                    &mut commonmark_cache.lock(),
                    &format!(
                        "{}\n{}",
                        DEFAULT_EXPLAINATIONS_MDS[*md],
                        SelectedConfig::iter()
                            .map(|x| format!(
                                "\n- **{}**: {}",
                                x.title().as_ref(),
                                x.descriptions()
                            ))
                            .collect::<String>()
                    ),
                );
                Default::default()
            }
            Tab::MarkdownEdit(md) => {
                {
                    // use parking_lot::Mutex;
                    use epaint::mutex::Mutex;
                    let commonmark_cache = ui.data_mut(|data| {
                        data.get_temp_mut_or_default::<std::sync::Arc<Mutex<egui_commonmark::CommonMarkCache>>>(
                            egui::Id::new("global_egui_commonmark_cache"),
                        )
                        .clone()
                    });

                    egui_commonmark::CommonMarkViewer::new().show_mut(
                        ui,
                        &mut commonmark_cache.lock(),
                        md,
                    );
                };
                Default::default()
            }
            Tab::Empty => unreachable!(),
        }
    }

    fn tab_title_for_pane(&mut self, pane: &TabId) -> egui::WidgetText {
        self.tabs[*pane as usize].title(&self.data).into()
    }

    fn top_bar_right_ui(
        &mut self,
        tiles: &egui_tiles::Tiles<TabId>,
        ui: &mut egui::Ui,
        tile_id: egui_tiles::TileId,
        tabs: &egui_tiles::Tabs,
        _scroll_offset: &mut f32,
    ) {
        let Some(active) = tabs.active.and_then(|active| tiles.get(active)) else {
            return;
        };
        let egui_tiles::Tile::Pane(space_view_id) = active else {
            return;
        };
        let space_view_id = *space_view_id;

        // let Some(space_view) = self.viewport_blueprint.space_views.get(&space_view_id) else {
        //     return;
        // };
        let Some(space_view) = self.tabs.get(space_view_id as usize) else {
            return;
        };
        let num_space_views = tiles.tiles().filter(|tile| tile.is_pane()).count();

        ui.add_space(8.0); // margin within the frame

        if *self.maximized == Some(space_view_id) {
            // Show minimize-button:
            if ui
                .small_icon_button(&re_ui::icons::MINIMIZE)
                .on_hover_text("Restore - show all spaces")
                .clicked()
            {
                *self.maximized = None;
            }
        } else if num_space_views > 1 {
            // Show maximize-button:
            if ui
                .small_icon_button(&re_ui::icons::MAXIMIZE)
                .on_hover_text("Maximize space view")
                .clicked()
            {
                *self.maximized = Some(space_view_id);
                // Just maximize - don't select. See https://github.com/rerun-io/rerun/issues/2861
            }
        }

        // let help_markdown = space_view
        //     .class(self.ctx.space_view_class_registry)
        //     .help_markdown(self.ctx.egui_ctx);
        let help_markdown = "TODO Help text";
        ui.help_hover_button().on_hover_ui(|ui| {
            ui.markdown_ui(&help_markdown);
        });

        if let Tab::QueryResults { id, .. } = space_view {
            let Some(QueryResults { content: res, .. }) =
                self.data.queries_results.get_mut(*id as usize)
            else {
                ui.error_label(&format!("{} is not in the list of queries", id));
                return Default::default();
            };
            if let Some(Ok(res)) = res.get() {
                if ui
                    .small_icon_button(&re_ui::icons::EXTERNAL_LINK)
                    .on_hover_text("Export data as json")
                    .clicked()
                {
                    if let Ok(text) = serde_json::to_string_pretty(res) {
                        utils::file_save("data", ".json", &text);
                    }
                }
            };
        }
    }

    fn is_tab_closable(
        &self,
        tiles: &egui_tiles::Tiles<TabId>,
        tile_id: egui_tiles::TileId,
    ) -> bool {
        let Some(tile) = tiles.get(tile_id) else {
            return false;
        };
        let egui_tiles::Tile::Pane(tid) = tile else {
            return false;
        };
        let Some(space_view) = self.tabs.get(*tid as usize) else {
            return false;
        };
        if let Tab::MarkdownStatic(0) = space_view {
            true
        } else {
            false
        }
    }

    // Styling:

    fn tab_outline_stroke(
        &self,
        _visuals: &egui::Visuals,
        _tiles: &egui_tiles::Tiles<TabId>,
        _tile_id: egui_tiles::TileId,
        _tab_state: &egui_tiles::TabState,
    ) -> egui::Stroke {
        _visuals.window_stroke
        // egui::Stroke::NONE
    }

    /// The height of the bar holding tab titles.
    fn tab_bar_height(&self, _style: &egui::Style) -> f32 {
        re_ui::DesignTokens::title_bar_height()
    }

    /// What are the rules for simplifying the tree?
    fn simplification_options(&self) -> egui_tiles::SimplificationOptions {
        egui_tiles::SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }

    fn on_edit(&mut self, edit_action: egui_tiles::EditAction) {
        use egui_tiles::EditAction;
        match edit_action {
            EditAction::TileDropped | EditAction::TabSelected | EditAction::TileResized => {
                self.edited = true;
            }
            _ => (),
        }
    }
}

impl<'a> MyTileTreeBehavior<'a> {
    fn show_commits(&mut self, ui: &mut egui::Ui) {
        for i in self.data.selected_code_data.project_ids() {
            let Some((r, mut c)) = self.data.selected_code_data.get_mut(i) else {
                continue;
            };
            ui.label(format!("{}/{}", r.user, r.name));
            for commit_oid in c.iter_mut() {
                fn f(
                    ui: &mut egui::Ui,
                    repo: &Repo,
                    id: &types::CommitId,
                    fetched_commit_metadata: &CommitMdStore,
                    to_fetch: &mut HashSet<String>,
                    d: usize,
                ) {
                    let limit = 20;
                    if let Some(res) = fetched_commit_metadata.get(id) {
                        match res {
                            Ok(md) => {
                                ui.label(format!("{}: {} {:?}", &id[..6], md.time, md.parents));
                                if d < limit {
                                    for id in &md.parents {
                                        f(ui, repo, id, fetched_commit_metadata, to_fetch, d + 1);
                                    }
                                    for (i, a) in md.ancestors.iter().enumerate() {
                                        let i = i + 2;
                                        if d + i * i >= limit {
                                            break;
                                        }
                                        if fetched_commit_metadata.is_absent(a.as_str()) {
                                            to_fetch.insert(a.to_string());
                                        }
                                    }
                                }
                            }
                            Err(err) => {
                                ui.error_label(err);
                            }
                        }
                    } else {
                        ui.horizontal(|ui| {
                            ui.label("fetching");
                            ui.add_space(2.0);
                            ui.label(id);
                            ui.add_space(2.0);
                            ui.spinner();
                            ui.label(to_fetch.len().to_string());
                        });
                        to_fetch.insert(id.to_string());
                    }
                }
                let mut to_fetch = HashSet::default();

                f(
                    ui,
                    r,
                    commit_oid,
                    &self.data.fetched_commit_metadata,
                    &mut to_fetch,
                    0,
                );

                for id in to_fetch {
                    let repo = r.clone();
                    let commit = Commit { repo, id };
                    let v = fetch_commit(ui.ctx(), &self.data.api_addr, &commit);
                    self.data.fetched_commit_metadata.insert(commit.id, v);
                }
            }
        }
        // if self.data.fetched_commit_metadata.try_poll_all_waiting(|x| {
        //     x.map(|x| {
        //         let selected_projects = &mut self.data.selected_code_data;
        //         poll_md_with_pr(x, selected_projects)
        //     })
        // }) {
        //     //
        // }
    }
}

fn poll_md_with_pr(
    (mut md, head_commit): (commit::CommitMetadata, Option<(Commit, ProjectId)>),
    selected_projects: &mut SelectedProjects,
) -> commit::CommitMetadata {
    if let Some((head_commit, i)) = head_commit {
        if !md.parents.contains(&head_commit.id) {
            if let Some((r, mut c)) = selected_projects.get_mut(i) {
                todo!()
            }
            md.parents.push(head_commit.id.clone());
        }
    }
    md
}

fn poll_md_with_pr2(
    (mut md, head_commit): (commit::CommitMetadata, Option<(Commit, ProjectId)>),
    rid: ProjectId,
    c: &mut CommitSlice<'_>,
) -> commit::CommitMetadata {
    if let Some((head_commit, i)) = head_commit {
        if !md.parents.contains(&head_commit.id) {
            if rid == i {
                c.push(head_commit.id.clone())
            } else {
                log::error!("{:?} {:?}", rid, i)
            }
            md.parents.push(head_commit.id.clone());
        }
    }
    md
}

pub(crate) fn show_projects_actions(ui: &mut egui::Ui, data: &mut AppData) {
    let multi = data.selected_code_data.len() > 1;

    let button = egui::Button::new("Update the current set of Repositories");
    if ui.add_enabled(false, button).clicked() {}
    let button = egui::Button::new("Show Commit Graph");
    if ui.add_enabled(true, button).clicked() {}
    let button = egui::Button::new("Find code patterns from examples");
    if ui.add_enabled(true, button).clicked() {}
    let button = egui::Button::new(if multi {
        "Query Repositories with tree-sitter-query"
    } else {
        "Query Repository with tree-sitter-query"
    });
    if ui.add_enabled(true, button).clicked() {}
}

pub(crate) fn show_project_selection(ui: &mut egui::Ui, data: &mut AppData) {
    let mut rm = None;
    for i in data.selected_code_data.project_ids() {
        let Some((r, _)) = data.selected_code_data.get_mut(i) else {
            continue;
        };
        ui.push_id(ui.id().with(i), |ui| {
            ui.horizontal(|ui| {
                ui.label("github.com");
                ui.label("/");
                ui.add(
                    egui::TextEdit::singleline(&mut r.user)
                        .id(ui.id().with("user"))
                        .clip_text(false)
                        .hint_text("user")
                        .desired_width(0.0)
                        .min_size((50.0, 0.0).into()),
                );
                ui.label("/");
                ui.add(
                    egui::TextEdit::singleline(&mut r.name)
                        .id(ui.id().with("name"))
                        .clip_text(false)
                        .hint_text("name")
                        .desired_width(0.0)
                        .min_size((50.0, 0.0).into()),
                );
                if (&ui.button("➖").on_hover_text("remove repository")).clicked() {
                    rm = Some(i);
                }
            });
        });

        ui.separator();
    }
    if let Some(i) = rm {
        data.selected_code_data.remove(i);
    }

    ui.add_space(20.0);
    if (&ui.button("➕").on_hover_text("add new repository")).clicked() {
        data.selected_code_data.add(
            Repo {
                user: Default::default(),
                name: Default::default(),
            },
            vec![],
        );
    }

    ui.add_space(40.0);
}

impl eframe::App for HyperApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        if !self.persistance {
            if let Some(s) = storage.get_string(eframe::APP_KEY) {
                match serde_json::from_str::<HyperApp>(&s) {
                    Ok(r) => {
                        if !r.persistance {
                            self.save_interval = std::time::Duration::from_secs(20);
                            return;
                        }
                    }
                    Err(err) => {
                        wasm_rs_dbg::dbg!(storage.get_string(eframe::APP_KEY));
                        log::debug!("Failed to decode RON: {err}");
                    }
                }
            }
            // let r: Option<HyperApp> = eframe::get_value(storage, eframe::APP_KEY);
            // if let Some(r) = r {
            //     if !r.persistance {
            //         return;
            //     }
            // } else {
            // }
        }
        // #[cfg(target_arch = "wasm32")]
        // use wasm_bindgen::prelude::*;
        // #[cfg(target_arch = "wasm32")]
        // #[wasm_bindgen]
        // extern "C" {
        //     fn prompt(text: &str, default: &str) -> String;
        // }
        // #[cfg(target_arch = "wasm32")]
        // unsafe {
        //     prompt("coucou", "cou")
        // };
        match serde_json::to_string(self) {
            Ok(s) => {
                storage.set_string(eframe::APP_KEY, s);
            }
            Err(err) => {
                log::debug!("Failed to decode RON: {err}");
                // storage.set_string(eframe::APP_KEY, s)
            }
        }
        self.save_interval = std::time::Duration::from_secs(20);
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }
    /// Time between automatic calls to [`Self::save`]
    fn auto_save_interval(&self) -> std::time::Duration {
        self.save_interval
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // hack to make sure the initial screen rect is correct...
        #[cfg(target_arch = "wasm32")]
        static mut AA: bool = true;
        #[cfg(target_arch = "wasm32")]
        static mut BB: Option<egui::Rect> = None;
        #[cfg(target_arch = "wasm32")]
        if unsafe { AA } {
            let rect = ctx.input(|i| i.raw.screen_rect);
            if rect.is_some() && unsafe { BB } == rect {
                unsafe { AA = false };
            } else {
                unsafe { BB = rect }
                return;
            }
        };
        crate::platform::show_nat_menu(ctx, _frame);

        self.show_text_logs_as_notifications();
        // self.toasts.ui(ctx);

        self.top_bar(ctx);

        self.bottom_panel(ctx);

        self.show_left_panel(ctx);

        egui::CentralPanel::default()
            .frame(egui::Frame {
                fill: ctx.style().visuals.panel_fill,
                ..Default::default()
            })
            .show(ctx, |ui| {
                if ui.available_size().min_elem() <= 0.0 {
                    return;
                }

                let mut edited = false;

                let mut maximized = self.maximized;

                if let Some(space_view_id) = self.maximized {
                    if let Tab::Empty = self.tabs[space_view_id as usize] {
                        maximized = None;
                    } else if let Some(tile_id) = self.tree.tiles.find_pane(&space_view_id) {
                        if !self.tree.tiles.is_visible(tile_id) {
                            maximized = None;
                        }
                    }
                }

                // if let Some(view_id) = focused {
                //     let found = self.tree.make_active(|_, tile| match tile {
                //         egui_tiles::Tile::Pane(this_view_id) => {
                //             *this_view_id == view_id
                //         }
                //         egui_tiles::Tile::Container(_) => false,
                //     });
                //     log::trace!(
                //         "Found tab to focus on for space view ID {view_id}: {found}"
                //     );
                //     edited = true;
                // }

                let mut maximized_tree;

                let tree = if let Some(view_id) = self.maximized {
                    let mut tiles = egui_tiles::Tiles::default();
                    let root = tiles.insert_pane(view_id);
                    maximized_tree = egui_tiles::Tree::new("viewport_tree", root, tiles);
                    &mut maximized_tree
                } else {
                    &mut self.tree
                };

                let mut tile_tree = MyTileTreeBehavior {
                    data: &mut self.data,
                    tabs: &mut self.tabs,
                    maximized: &mut maximized,
                    edited,
                    selected_commit: &mut self.selected_commit,
                    selected_baseline: &mut self.selected_baseline,
                    to_hide: Default::default(),
                    tab_to_add: None,
                };
                tree.ui(&mut tile_tree, ui);

                if tile_tree.edited {
                    self.save_interval = std::time::Duration::ZERO;
                }
                for tile_id in tile_tree.to_hide {
                    tree.set_visible(tile_id, false);
                }

                if let Some(tid) = tile_tree.tab_to_add {
                    let child = self.tree.tiles.insert_pane(tid);
                    match self.tree.tiles.get_mut(self.tree.root.unwrap()) {
                        Some(egui_tiles::Tile::Container(c)) => c.add_child(child),
                        _ => todo!(),
                    };
                }
                self.maximized = maximized;
            });

        //TODO use https://github.com/mlange-42/git-graph/blob/7b9bb72a310243cc53d906d1e7ec3c9aad1c75d2/src/graph.rs#L791 to display git history

        self.old_ui(ctx);

        use crate::command::UICommandSender;
        if let Some(cmd) = self.cmd_palette.show(ctx) {
            self.data.command_sender.send_ui(cmd);
        }
        if let Some(cmd) = UICommand::listen_for_kb_shortcut(ctx) {
            self.data.command_sender.send_ui(cmd);
        }

        while let Some(cmd) = self.data.command_receiver.recv() {
            self.latest_cmd = cmd.text().to_owned();
            self.save_interval = std::time::Duration::from_secs(0);

            match cmd {
                UICommand::ToggleCommandPalette => self.cmd_palette.toggle(),
                UICommand::PersistApp => {
                    self.save_interval = std::time::Duration::ZERO;
                    self.persistance = true;
                }
                UICommand::NewQuery => {
                    self.data.queries.push(crate::app::QueryData {
                        ..Default::default()
                    });
                    let qid = self.data.queries.len() as u16 - 1;
                    let tid = self.tabs.len() as u16;
                    self.tabs.push(crate::app::Tab::LocalQuery(qid));
                    let child = self.tree.tiles.insert_pane(tid);
                    match self.tree.tiles.get_mut(self.tree.root.unwrap()) {
                        Some(egui_tiles::Tile::Container(c)) => c.add_child(child),
                        _ => todo!(),
                    };
                }
                #[cfg(not(target_arch = "wasm32"))]
                UICommand::ZoomIn => {
                    let mut zoom_factor = ctx.zoom_factor();
                    zoom_factor += 0.1;
                    ctx.set_zoom_factor(zoom_factor);
                }
                #[cfg(not(target_arch = "wasm32"))]
                UICommand::ZoomOut => {
                    let mut zoom_factor = ctx.zoom_factor();
                    zoom_factor -= 0.1;
                    ctx.set_zoom_factor(zoom_factor);
                }
                #[cfg(not(target_arch = "wasm32"))]
                UICommand::ZoomReset => {
                    ctx.set_zoom_factor(1.0);
                }
                x => {
                    wasm_rs_dbg::dbg!(x);
                }
            }
        }
    }
}
