use std::{
    ops::DerefMut,
    sync::{Arc, Mutex},
};

use poll_promise::Promise;

use crate::app::{
    types::EditorHolder,
    utils_edition::{
        show_available_remote_docs, show_locals_and_interact, show_shared_code_edition,
    },
    utils_results_batched::ComputeResults,
};

use self::example_scripts::EXAMPLES;

use egui_addon::{
    code_editor::EditorInfo, interactive_split::interactive_splitter::InteractiveSplitter,
};

use super::{
    Sharing, code_editor_automerge, show_repo_menu,
    types::{CodeEditors, Commit, Config, Resource, SelectedConfig, WithDesc},
    utils_edition::{self, EditStatus, EditingContext, show_interactions, update_shared_editors},
    utils_results_batched,
};
mod example_scripts;

const INFO_INIT: EditorInfo<&'static str> = EditorInfo {
    title: "Init",
    short: "initializes the accumulator on the root node",
    long: concat!("It will recieve the finally results of the entire computation."),
};
const INFO_FILTER: EditorInfo<&'static str> = EditorInfo {
    title: "Filter",
    short: "filters nodes of the HyperAST that should be processed",
    long: concat!(
        "It goes through nodes in pre-order, returning the list of node that should be processed next and initializing their own states.\n",
        "`s` is the current node accumulator"
    ),
};
const INFO_ACCUMULATE: EditorInfo<&'static str> = EditorInfo {
    title: "Accumulate",
    short: "accumulates values to produce the wanted metrics",
    long: concat!(
        "It goes through nodes in post-order, accumulating values from `s` into `p`.\n",
        "`s` is the accumulator of the current node.\n",
        "`p` the accumulator of the parent node."
    ),
};
const INFO_DESCRIPTION: EditorInfo<&'static str> = EditorInfo {
    title: "Desc",
    short: "describes what this script does",
    long: concat!(
        "TODO syntax is similar to markdown.\n",
        "WIP rendering the markdown, there is already an egui helper for that."
    ),
};

impl<C> From<&example_scripts::Scripts> for CodeEditors<C>
where
    C: From<(EditorInfo<String>, String)> + egui_addon::code_editor::CodeHolder,
{
    fn from(value: &example_scripts::Scripts) -> Self {
        let mut description: C = (INFO_DESCRIPTION.copied(), value.description.into()).into();
        description.set_lang("md");
        Self {
            description, // TODO config with markdown, not js
            init: (INFO_INIT.copied(), value.init.into()).into(),
            filter: (INFO_FILTER.copied(), value.filter.into()).into(),
            accumulate: (INFO_ACCUMULATE.copied(), value.accumulate.into()).into(),
        }
    }
}

impl<C> Default for CodeEditors<C>
where
    C: From<(EditorInfo<String>, String)> + egui_addon::code_editor::CodeHolder,
{
    fn default() -> Self {
        (&example_scripts::EXAMPLES[0].scripts).into()
    }
}

impl<T> super::types::EditorHolder for CodeEditors<T> {
    type Item = T;

    fn iter_editors_mut(&mut self) -> impl Iterator<Item = &mut Self::Item> {
        [
            &mut self.description,
            &mut self.init,
            &mut self.filter,
            &mut self.accumulate,
        ]
        .into_iter()
    }
}

impl<T> WithDesc<T> for CodeEditors<T> {
    fn desc(&self) -> &T {
        &self.description
    }
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

impl Into<CodeEditors<super::code_editor_automerge::CodeEditor>> for CodeEditors {
    fn into(self) -> CodeEditors<super::code_editor_automerge::CodeEditor> {
        self.to_shared()
    }
}

pub(super) type ScriptingContext =
    utils_edition::EditingContext<CodeEditors, CodeEditors<code_editor_automerge::CodeEditor>>;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(super) struct ComputeConfigSingle {
    commit: Commit,
    config: Config,
    len: usize,
}

impl Default for ComputeConfigSingle {
    fn default() -> Self {
        // let rt = Default::default();
        // // let quote = Default::default();
        // let ws = None;
        // let doc_db = None;
        Self {
            commit: From::from(&example_scripts::EXAMPLES[0].commit),
            config: example_scripts::EXAMPLES[0].config,
            // commit: "4acedc53a13a727be3640fe234f7e261d2609d58".into(),
            len: example_scripts::EXAMPLES[0].commits,
            // rt,
            // ws,
            // doc_db,
        }
    }
}

// pub(super) type RemoteResult =
//     Promise<ehttp::Result<Resource<Result<ComputeResults, ScriptingError>>>>;

pub(super) fn remote_compute_single(
    ctx: &egui::Context,
    api_addr: &str,
    single: &mut ComputeConfigSingle,
    code_editors: &mut ScriptingContext,
) -> Promise<Result<Resource<Result<ComputeResults, ScriptingError>>, String>> {
    // TODO multi requests from client
    // if single.len > 1 {
    //     let parents = fetch_commit_parents(&ctx, &single.commit, single.len);
    // }
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/script-depth/github/{}/{}/{}",
        api_addr, &single.commit.repo.user, &single.commit.repo.name, &single.commit.id,
    );
    #[derive(serde::Serialize)]
    struct ScriptContent {
        init: String,
        filter: String,
        accumulate: String,
        commits: usize,
    }
    impl ScriptContent {
        fn new(
            code_editors: &mut super::types::CodeEditors<impl AsRef<str>>,
            single: &ComputeConfigSingle,
        ) -> Self {
            ScriptContent {
                init: code_editors.init.as_ref().to_string(),
                filter: code_editors.filter.as_ref().to_string(),
                accumulate: code_editors.accumulate.as_ref().to_string(),
                commits: single.len,
            }
        }
    }
    let script = code_editors.map(
        |code_editors| ScriptContent::new(code_editors, single),
        |code_editors| ScriptContent::new(&mut code_editors.lock().unwrap(), single),
    );

    let mut request = ehttp::Request::post(&url, serde_json::to_vec(&script).unwrap());
    request.headers.insert(
        "Content-Type".to_string(),
        "application/json; charset=utf-8".to_string(),
    );

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        let resource = response.and_then(|response| {
            Resource::<Result<ComputeResults, ScriptingError>>::from_response(&ctx, response)
        });
        sender.send(resource);
    });
    promise
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum ScriptingError {
    AtCompilation(String),
    AtEvaluation(String),
    Other(String),
}

pub(super) fn show_single_repo(
    ui: &mut egui::Ui,
    api_addr: &str,
    single: &mut Sharing<ComputeConfigSingle>,
    code_editors: &mut ScriptingContext,
    trigger_compute: &mut bool,
    compute_single_result: &mut Option<
        poll_promise::Promise<
            Result<super::types::Resource<Result<ComputeResults, ScriptingError>>, String>,
        >,
    >,
) {
    let api_endpoint = &format!("{}/sharing-scripts", api_addr);
    update_shared_editors(ui, single, api_endpoint, code_editors);
    let is_portrait = ui.available_rect_before_wrap().aspect_ratio() < 1.0;
    if is_portrait {
        egui::ScrollArea::vertical().show(ui, |ui| {
            show_scripts_edition(ui, api_endpoint, code_editors, single);
            handle_interactions(
                ui,
                code_editors,
                compute_single_result,
                single,
                trigger_compute,
            );
            utils_results_batched::show_long_result(&*compute_single_result, ui);
        });
    } else {
        InteractiveSplitter::vertical()
            .ratio(0.7)
            .show(ui, |ui1, ui2| {
                ui1.push_id(ui1.id().with("input"), |ui| {
                    show_scripts_edition(ui, api_endpoint, code_editors, single);
                });
                let ui = ui2;
                handle_interactions(
                    ui,
                    code_editors,
                    compute_single_result,
                    single,
                    trigger_compute,
                );
                utils_results_batched::show_long_result(&*compute_single_result, ui);
            });
    }
}

fn handle_interactions(
    ui: &mut egui::Ui,
    code_editors: &mut EditingContext<CodeEditors, CodeEditors<code_editor_automerge::CodeEditor>>,
    compute_single_result: &mut Option<
        Promise<Result<Resource<Result<ComputeResults, ScriptingError>>, String>>,
    >,
    single: &mut Sharing<ComputeConfigSingle>,
    trigger_compute: &mut bool,
) {
    let interaction = show_interactions(
        ui,
        code_editors,
        &single.doc_db,
        compute_single_result,
        |i| EXAMPLES[i].name.to_string(),
    );
    if interaction.share_button.map_or(false, |x| x.clicked()) {
        let (name, content) = interaction.editor.unwrap();
        let content = content.clone().to_shared();
        let content = Arc::new(Mutex::new(content));
        let name = name.to_string();
        code_editors.current = EditStatus::Sharing(content.clone());
        let mut content = content.lock().unwrap();
        let db = &mut single.doc_db.as_mut().unwrap();
        db.create_doc_atempt(&single.rt, name, content.deref_mut());
    } else if interaction.save_button.map_or(false, |x| x.clicked()) {
        let (name, content) = interaction.editor.unwrap();
        log::warn!("saving script: {:#?}", content.clone());
        let name = name.to_string();
        let content = content.clone();
        code_editors
            .local_scripts
            .insert(name.to_string(), content.clone());
        code_editors.current = EditStatus::Local { name, content };
    } else if interaction.compute_button.clicked() {
        *trigger_compute |= true;
    }
}

fn show_scripts_edition(
    ui: &mut egui::Ui,
    api_endpoint: &str,
    scripting_context: &mut ScriptingContext,
    single: &mut Sharing<ComputeConfigSingle>,
) {
    egui::warn_if_debug_build(ui);
    egui::CollapsingHeader::new("Examples")
        .default_open(true)
        .show(ui, |ui| {
            show_examples(ui, &mut single.content, scripting_context)
        });
    if !scripting_context.local_scripts.is_empty() {
        egui::CollapsingHeader::new("Local Queries")
            .default_open(true)
            .show(ui, |ui| {
                show_locals_and_interact(ui, scripting_context, single);
            });
    }
    show_available_remote_docs(ui, api_endpoint, single, scripting_context);
    let local = scripting_context
        .when_local(|code_editors| code_editors.iter_editors_mut().for_each(|e| e.ui(ui)));
    let shared = scripting_context
        .when_shared(|code_editors| show_shared_code_edition(ui, code_editors, single));
    assert!(local.or(shared).is_some());
}

fn show_examples(
    ui: &mut egui::Ui,
    single: &mut ComputeConfigSingle,
    scripting_context: &mut ScriptingContext,
) {
    ui.horizontal_wrapped(|ui| {
        let mut j = 0;
        for ex in EXAMPLES {
            let mut text = egui::RichText::new(ex.name);
            if let EditStatus::Example { i, .. } = &scripting_context.current {
                if &j == i {
                    text = text.strong();
                }
            }
            let button = &ui.button(text);
            if button.clicked() {
                single.commit = (&ex.commit).into();
                single.config = ex.config;
                single.len = ex.commits;
                scripting_context.current = EditStatus::Example {
                    i: j,
                    content: (&ex.scripts).into(),
                };
            }
            if button.hovered() {
                egui::show_tooltip(ui.ctx(), ui.layer_id(), button.id.with("tooltip"), |ui| {
                    let desc = ex.scripts.description;
                    egui_demo_lib::easy_mark::easy_mark(ui, desc);
                });
            }
            j += 1;
        }
    });
}

impl Resource<Result<ComputeResults, ScriptingError>> {
    pub(super) fn from_response(
        _ctx: &egui::Context,
        response: ehttp::Response,
    ) -> Result<Self, String> {
        wasm_rs_dbg::dbg!(&response);
        let content_type = response.content_type().unwrap_or_default();
        if !content_type.starts_with("application/json") {
            return Err(format!("Wrong content type: {}", content_type));
        }
        // let image = if content_type.starts_with("image/") {
        //     RetainedImage::from_image_bytes(&response.url, &response.bytes).ok()
        // } else {
        //     None
        // };
        if response.status != 200 {
            let Some(text) = response.text() else {
                wasm_rs_dbg::dbg!();
                return Err("".to_string());
            };
            let Ok(json) = serde_json::from_str::<ScriptingError>(text) else {
                wasm_rs_dbg::dbg!();
                return Err("".to_string());
            };
            return Ok(Self {
                response,
                content: Some(Err(json)),
            });
        }

        let text = response.text();
        // let colored_text = text.and_then(|text| syntax_highlighting(ctx, &response, text));
        let text = text.and_then(|text| {
            serde_json::from_str(text)
                .inspect_err(|err| {
                    dbg!(&err);
                })
                .ok()
        });

        Ok(Self {
            response,
            content: text.map(|x| Ok(x)),
        })
    }
}

pub(crate) const WANTED: SelectedConfig = SelectedConfig::Single;

pub(crate) fn show_config(ui: &mut egui::Ui, single: &mut Sharing<ComputeConfigSingle>) {
    show_repo_menu(ui, &mut single.content.commit.repo);
    ui.push_id(ui.id().with("commit"), |ui| {
        egui::TextEdit::singleline(&mut single.content.commit.id)
            .clip_text(true)
            .desired_width(150.0)
            .desired_rows(1)
            .hint_text("commit")
            .interactive(true)
            .show(ui)
    });

    ui.add_enabled_ui(true, |ui| {
        ui.add(
            egui::Slider::new(&mut single.content.len, 1..=200)
                .text("commits")
                .clamping(egui::SliderClamping::Never)
                .integer()
                .logarithmic(true),
        );
        // show_wip(ui, Some("only process one commit"));
    });
    let selected = &mut single.content.config;
    selected.show_combo_box(ui, "Repo Config");
}

impl utils_results_batched::ComputeError for ScriptingError {
    fn head(&self) -> &str {
        match self {
            ScriptingError::AtCompilation(_) => "Error at compilation:",
            ScriptingError::AtEvaluation(_) => "Error at evaluation:",
            ScriptingError::Other(_) => "Error somewhere else:",
        }
    }

    fn content(&self) -> &str {
        match self {
            ScriptingError::AtCompilation(err) => err,
            ScriptingError::AtEvaluation(err) => err,
            ScriptingError::Other(err) => err,
        }
    }
}
