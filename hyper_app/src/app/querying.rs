use std::{
    ops::DerefMut,
    sync::{Arc, Mutex},
};

use poll_promise::Promise;

use crate::app::{
    types::EditorHolder,
    utils_edition::{self, show_available_remote_docs, show_locals_and_interact},
};

use self::example_queries::EXAMPLES;

use egui_addon::{
    code_editor::EditorInfo, egui_utils::radio_collapsing,
    interactive_split::interactive_splitter::InteractiveSplitter,
};

use super::{
    code_editor_automerge, show_repo_menu,
    types::{Commit, Config, QueryEditor, Resource, SelectedConfig, WithDesc},
    utils_edition::{show_interactions, update_shared_editors},
    utils_results_batched::{self, show_long_result, ComputeResults},
    Sharing,
};
pub(crate) mod example_queries;

const INFO_QUERY: EditorInfo<&'static str> = EditorInfo {
    title: "Query",
    short: "the query",
    long: concat!("follows the tree sitter query syntax"),
};

const INFO_DESCRIPTION: EditorInfo<&'static str> = EditorInfo {
    title: "Desc",
    short: "describes what this query should match",
    long: concat!(
        "TODO syntax is similar to markdown.\n",
        "WIP rendering the markdown, there is already an egui helper for that."
    ),
};

impl<C> From<&example_queries::Query> for QueryEditor<C>
where
    C: From<(EditorInfo<String>, String)> + egui_addon::code_editor::CodeHolder,
{
    fn from(value: &example_queries::Query) -> Self {
        let mut description: C = (INFO_DESCRIPTION.copied(), value.description.into()).into();
        description.set_lang("md");
        Self {
            description, // TODO config with markdown, not js
            query: (INFO_QUERY.copied(), value.query.into()).into(),
        }
    }
}

impl<C> Default for QueryEditor<C>
where
    C: From<(EditorInfo<String>, String)> + egui_addon::code_editor::CodeHolder,
{
    fn default() -> Self {
        (&example_queries::EXAMPLES[0].query).into()
    }
}

impl<T> WithDesc<T> for QueryEditor<T> {
    fn desc(&self) -> &T {
        &self.description
    }
}

impl<T> super::types::EditorHolder for QueryEditor<T> {
    type Item = T;

    fn iter_editors_mut(&mut self) -> impl Iterator<Item = &mut Self::Item> {
        [&mut self.description, &mut self.query].into_iter()
    }
}

impl<T> QueryEditor<T> {
    pub(crate) fn to_shared<U>(self) -> QueryEditor<U>
    where
        T: Into<U>,
    {
        QueryEditor {
            description: self.description.into(),
            query: self.query.into(),
        }
    }
}

impl Into<QueryEditor<super::code_editor_automerge::CodeEditor>> for QueryEditor {
    fn into(self) -> QueryEditor<super::code_editor_automerge::CodeEditor> {
        self.to_shared()
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(super) struct ComputeConfigQuery {
    commit: Commit,
    config: Config,
    len: usize,
}

impl Default for ComputeConfigQuery {
    fn default() -> Self {
        Self {
            commit: From::from(&example_queries::EXAMPLES[0].commit),
            config: example_queries::EXAMPLES[0].config,
            // commit: "4acedc53a13a727be3640fe234f7e261d2609d58".into(),
            len: example_queries::EXAMPLES[0].commits,
        }
    }
}

type QueryingContext = super::EditingContext<
    super::types::QueryEditor,
    super::types::QueryEditor<code_editor_automerge::CodeEditor>,
>;

pub(super) fn remote_compute_query(
    ctx: &egui::Context,
    api_addr: &str,
    single: &mut Sharing<ComputeConfigQuery>,
    query_editors: &mut QueryingContext,
) -> Promise<Result<Resource<Result<ComputeResults, QueryingError>>, String>> {
    // TODO multi requests from client
    // if single.len > 1 {
    //     let parents = fetch_commit_parents(&ctx, &single.commit, single.len);
    // }
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/query/github/{}/{}/{}",
        api_addr,
        &single.content.commit.repo.user,
        &single.content.commit.repo.name,
        &single.content.commit.id,
    );
    #[derive(serde::Serialize)]
    struct QueryContent {
        language: String,
        query: String,
        commits: usize,
    }
    let language = match single.content.config {
        Config::Any => "",
        Config::MavenJava => "Java",
        Config::MakeCpp => "Cpp",
    }
    .to_string();
    let script = match &mut query_editors.current {
        super::EditStatus::Shared(_, shared_script) | super::EditStatus::Sharing(shared_script) => {
            let code_editors = shared_script.lock().unwrap();
            QueryContent {
                language,
                query: code_editors.query.code().to_string(),
                commits: single.content.len,
            }
        }
        super::EditStatus::Local { name: _, content }
        | super::EditStatus::Example { i: _, content } => QueryContent {
            language,
            query: content.query.code().to_string(),
            commits: single.content.len,
        },
    };

    let mut request = ehttp::Request::post(&url, serde_json::to_vec(&script).unwrap());
    request.headers.insert(
        "Content-Type".to_string(),
        "application/json; charset=utf-8".to_string(),
    );

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        let resource = response.and_then(|response| {
            Resource::<Result<ComputeResults, QueryingError>>::from_response(&ctx, response)
        });
        sender.send(resource);
    });
    promise
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum QueryingError {
    MissingLanguage(String),
}

pub(super) fn show_querying(
    ui: &mut egui::Ui,
    api_addr: &str,
    query: &mut Sharing<ComputeConfigQuery>,
    query_editors: &mut QueryingContext,
    trigger_compute: &mut bool,
    querying_result: &mut Option<
        poll_promise::Promise<
            Result<super::types::Resource<Result<ComputeResults, QueryingError>>, String>,
        >,
    >,
) {
    let api_endpoint = &format!("{}/sharing-queries", api_addr);
    update_shared_editors(ui, query, api_endpoint, query_editors);
    let is_portrait = ui.available_rect_before_wrap().aspect_ratio() < 1.0;
    if is_portrait {
        egui::ScrollArea::vertical().show(ui, |ui| {
            show_scripts_edition(ui, api_endpoint, query_editors, query);
            handle_interactions(ui, query_editors, querying_result, query, trigger_compute);
            show_long_result(&*querying_result, ui);
        });
    } else {
        InteractiveSplitter::vertical()
            .ratio(0.7)
            .show(ui, |ui1, ui2| {
                ui1.push_id(ui1.id().with("input"), |ui| {
                    show_scripts_edition(ui, api_endpoint, query_editors, query);
                });
                let ui = ui2;
                handle_interactions(ui, query_editors, querying_result, query, trigger_compute);
                show_long_result(&*querying_result, ui);
            });
    }
}

fn handle_interactions(
    ui: &mut egui::Ui,
    code_editors: &mut QueryingContext,
    querying_result: &mut Option<
        Promise<Result<Resource<Result<ComputeResults, QueryingError>>, String>>,
    >,
    single: &mut Sharing<ComputeConfigQuery>,
    trigger_compute: &mut bool,
) {
    let interaction = show_interactions(ui, code_editors, &single.doc_db, querying_result, |i| {
        EXAMPLES[i].name.to_string()
    });
    if interaction.share_button.map_or(false, |x| x.clicked()) {
        let (name, content) = interaction.editor.unwrap();
        let content = content.clone().to_shared();
        let content = Arc::new(Mutex::new(content));
        let name = name.to_string();
        code_editors.current = super::EditStatus::Sharing(content.clone());
        let mut content = content.lock().unwrap();
        let db = &mut single.doc_db.as_mut().unwrap();
        db.create_doc_atempt(&single.rt, name, content.deref_mut());
    } else if interaction.save_button.map_or(false, |x| x.clicked()) {
        let (name, content) = interaction.editor.unwrap();
        log::warn!("saving query: {:#?}", content.clone());
        let name = name.to_string();
        let content = content.clone();
        code_editors
            .local_scripts
            .insert(name.to_string(), content.clone());
        code_editors.current = super::EditStatus::Local { name, content };
    } else if interaction.compute_button.clicked() {
        *trigger_compute |= true;
    }
}

fn show_scripts_edition(
    ui: &mut egui::Ui,
    api_endpoint: &str,
    querying_context: &mut QueryingContext,
    single: &mut Sharing<ComputeConfigQuery>,
) {
    egui::warn_if_debug_build(ui);
    egui::CollapsingHeader::new("Examples")
        .default_open(true)
        .show(ui, |ui| {
            show_examples(ui, &mut single.content, querying_context)
        });
    if !querying_context.local_scripts.is_empty() {
        egui::CollapsingHeader::new("Local Queries")
            .default_open(true)
            .show(ui, |ui| {
                show_locals_and_interact(ui, querying_context, single);
            });
    }
    show_available_remote_docs(ui, api_endpoint, single, querying_context);
    let local = querying_context
        .when_local(|code_editors| code_editors.iter_editors_mut().for_each(|c| c.ui(ui)));
    let shared = querying_context.when_shared(|query_editors| {
        utils_edition::show_shared_code_edition(ui, query_editors, single)
    });
    assert!(local.or(shared).is_some());
}

fn show_examples(
    ui: &mut egui::Ui,
    single: &mut ComputeConfigQuery,
    querying_context: &mut QueryingContext,
) {
    ui.horizontal_wrapped(|ui| {
        let mut j = 0;
        for ex in EXAMPLES {
            let mut text = egui::RichText::new(ex.name);
            if let super::EditStatus::Example { i, .. } = &querying_context.current {
                if &j == i {
                    text = text.strong();
                }
            }
            let button = &ui.button(text);
            if button.clicked() {
                single.commit = (&ex.commit).into();
                single.config = ex.config;
                single.len = ex.commits;
                querying_context.current = super::EditStatus::Example {
                    i: j,
                    content: (&ex.query).into(),
                };
            }
            if button.hovered() {
                egui::show_tooltip(ui.ctx(), ui.layer_id(), button.id.with("tooltip"), |ui| {
                    let desc = ex.query.description;
                    egui_demo_lib::easy_mark::easy_mark(ui, desc);
                });
            }
            j += 1;
        }
    });
}

impl Resource<Result<ComputeResults, QueryingError>> {
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
            let Ok(json) = serde_json::from_str::<QueryingError>(text) else {
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
                    wasm_rs_dbg::dbg!(&err);
                })
                .ok()
        });

        Ok(Self {
            response,
            content: text.map(|x| Ok(x)),
        })
    }
}

pub(super) fn show_querying_menu(
    ui: &mut egui::Ui,
    selected: &mut SelectedConfig,
    single: &mut Sharing<ComputeConfigQuery>,
) {
    let title = "Querying";
    let wanted = SelectedConfig::Querying;
    let id = ui.make_persistent_id(title);
    let add_body = |ui: &mut egui::Ui| {
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
                    .clamp_to_range(false)
                    .integer()
                    .logarithmic(true),
            );
            // show_wip(ui, Some("only process one commit"));
        });
        let selected = &mut single.content.config;
        selected.show_combo_box(ui, "Repo Config");
    };

    radio_collapsing(ui, id, title, selected, &wanted, add_body);
}

impl utils_results_batched::ComputeError for QueryingError {
    fn head(&self) -> &str {
        match self {
            QueryingError::MissingLanguage(_) => "Missing Language:",
        }
    }

    fn content(&self) -> &str {
        match self {
            QueryingError::MissingLanguage(err) => err,
        }
    }
}
