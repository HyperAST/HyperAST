use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::DerefMut,
    sync::{Arc, Mutex},
};

use poll_promise::Promise;

use crate::app::{
    types::EditorHolder,
    utils_edition::{self, show_available_remote_docs, show_locals_and_interact},
    utils_results_batched::ComputeResultIdentified,
};

use self::example_queries::EXAMPLES;

use egui_addon::{
    code_editor::EditorInfo, interactive_split::interactive_splitter::InteractiveSplitter,
};

use super::{
    Sharing, code_editor_automerge, show_repo_menu,
    types::{Commit, Config, QueryEditor, Resource, SelectedConfig, WithDesc},
    utils_edition::{EditStatus, show_interactions, update_shared_editors},
    utils_results_batched::{self, ComputeResults, show_long_result},
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
pub(super) struct ComputeConfigQuery {
    pub(super) commit: Commit,
    pub(super) config: Config,
    pub(super) len: usize,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub(super) struct ComputeConfigQueryDifferential {
    pub(super) commit: Commit,
    pub(super) config: Config,
    pub(super) baseline: Commit,
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

pub(crate) type QueryingContext = super::utils_edition::EditingContext<
    super::types::QueryEditor,
    super::types::QueryEditor<code_editor_automerge::CodeEditor>,
>;

pub(super) fn remote_compute_query(
    ctx: &egui::Context,
    api_addr: &str,
    single: &Sharing<ComputeConfigQuery>,
    query_editors: &mut QueryingContext,
) -> Promise<Result<Resource<Result<ComputeResults, QueryingError>>, String>> {
    let language = match single.content.config {
        Config::Any => "",
        Config::MavenJava => "Java",
        Config::MakeCpp => "Cpp",
    }
    .to_string();
    let script = match &mut query_editors.current {
        EditStatus::Shared(_, shared_script) | EditStatus::Sharing(shared_script) => {
            let code_editors = shared_script.lock().unwrap();
            QueryContent {
                language,
                query: code_editors.query.code().to_string(),
                precomp: None,
                commits: single.content.len,
                max_matches: u64::MAX,
                timeout: u64::MAX,
            }
        }
        EditStatus::Local { name: _, content } | EditStatus::Example { i: _, content } => {
            QueryContent {
                language,
                query: content.query.code().to_string(),
                precomp: None,
                commits: single.content.len,
                max_matches: u64::MAX,
                timeout: u64::MAX,
            }
        }
    };
    remote_compute_query_aux_old(ctx, api_addr, &single.content, script)
}

#[derive(serde::Serialize, Debug)]
pub(crate) struct QueryContent {
    pub(crate) language: String,
    pub(crate) query: String,
    pub(crate) precomp: Option<String>,
    pub(crate) commits: usize,
    /// checked per individual match
    /// if triggered on first search (ie. first commit searched) it return directly
    /// if triggered later, divide the numer of commits remaining to analyze by 2 each time (ie. `commits`` field)
    pub(crate) max_matches: u64,
    /// checked each match (in milli seconds)
    pub(crate) timeout: u64,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct StreamedDataTable<H, R> {
    pub head: H,
    pub commits: usize,
    pub rows: Arc<Mutex<Rows<R>>>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub(crate) struct Rows<R>(
    pub(crate) u64,
    pub(crate) Vec<R>,
    #[serde(skip)]
    #[serde(default)]
    pub(crate) bool,
);

impl<R> Default for Rows<R> {
    fn default() -> Self {
        Self(Default::default(), Default::default(), Default::default())
    }
}

pub(crate) type StreamedComputeResults =
    StreamedDataTable<Vec<String>, Result<ComputeResultIdentified, MatchingError>>;

pub(crate) fn remote_compute_query_aux(
    ctx: &egui::Context,
    api_addr: &str,
    single: &ComputeConfigQuery,
    script: QueryContent,
    additional_commits: impl Iterator<Item = super::types::CommitId>,
) -> Promise<Result<StreamedComputeResults, QueryingError>> {
    // TODO multi requests from client
    // if single.len > 1 {
    //     let parents = fetch_commit_parents(&ctx, &single.commit, single.len);
    // }
    let ctx = ctx.clone();
    let (header_sender, promise) = Promise::<Result<_, QueryingError>>::new();
    let tabl_body: Arc<Mutex<Rows<_>>> = Arc::default();

    let move_once = std::cell::Cell::new(Some((header_sender, tabl_body.clone())));
    let addi = crate::app::utils::join(additional_commits, "/").to_string();
    let url = format!(
        "http://{}/query-st/github/{}/{}/{}/{}",
        api_addr, &single.commit.repo.user, &single.commit.repo.name, &single.commit.id, addi,
    );
    wasm_rs_dbg::dbg!(&url, &script);

    let mut request = ehttp::Request::post(&url, serde_json::to_vec(&script).unwrap());
    request.headers.insert(
        "Content-Type".to_string(),
        "application/json; charset=utf-8".to_string(),
    );
    ehttp::streaming::fetch(request, move |response| {
        match response {
            Ok(ehttp::streaming::Part::Response(s)) => {
                log::debug!("{:?}", s);
                // TODO parse the headers
                let (header_sender, rows) = move_once.take().unwrap();
                rows.lock().unwrap().2 = true;
                if 200 <= s.status && s.status < 400 {
                    let commits = s.headers.get("commits").unwrap().parse().unwrap();
                    header_sender.send(Ok(StreamedDataTable {
                        head: s
                            .headers
                            .get("table_head")
                            .map_or(vec![], |x| serde_json::from_str(x).unwrap()),
                        rows,
                        commits,
                    }));
                    std::ops::ControlFlow::Continue(())
                } else {
                    log::debug!("{:?}", s.headers.get("error_query"));
                    log::debug!("{:?}", s.headers.get("error_parsing"));
                    if let Some(e) = s.headers.get("error_query") {
                        header_sender.send(Err(serde_json::from_str(e).unwrap()));
                    } else if let Some(e) = s.headers.get("error_parsing") {
                        header_sender.send(Err(serde_json::from_str(e)
                            .unwrap_or(QueryingError::NetworkError(e.to_string()))));
                    } else {
                        header_sender.send(Err(QueryingError::NetworkError("Unknown".into())));
                    }
                    std::ops::ControlFlow::Break(())
                }
            }
            Ok(ehttp::streaming::Part::Chunk(chunk)) if chunk.is_empty() => {
                log::debug!("{:?}", chunk);
                let mut l = tabl_body.lock().unwrap();
                log::debug!("{:?}", l);
                // l.2 = true;
                let mut hasher = DefaultHasher::new();
                l.0.hash(&mut hasher);
                0.hash(&mut hasher);
                l.0 = hasher.finish();
                std::ops::ControlFlow::Break(())
            }
            Ok(ehttp::streaming::Part::Chunk(chunk)) => {
                ctx.request_repaint(); // wake up UI thread
                match std::str::from_utf8(&chunk) {
                    Ok(a) => {
                        #[derive(Debug, serde::Deserialize, serde::Serialize)]
                        #[serde(untagged)]
                        enum R {
                            Resp(ComputeResultIdentified),
                            Err(MatchingError),
                        }
                        log::debug!("{}", a);

                        let s = serde_json::Deserializer::from_str(a);
                        let it = s.into_iter::<R>();
                        let mut l = tabl_body.lock().unwrap();
                        let mut hasher = DefaultHasher::new();
                        l.0.hash(&mut hasher);
                        let it = it
                            .filter_map(|x| x.inspect_err(|err| log::error!("{:?}", err)).ok())
                            .map(|x| match x {
                                R::Resp(r) => {
                                    r.hash(&mut hasher);
                                    // log::debug!("{:?}", r);
                                    Ok(r)
                                }
                                R::Err(err) => {
                                    log::debug!("{:?}", err);
                                    err.hash(&mut hasher);
                                    Err(err)
                                }
                            });
                        l.1.extend(it);
                        l.0 = hasher.finish();
                    }
                    _ => panic!(),
                }
                std::ops::ControlFlow::Continue(())
            }
            Err(err) => {
                log::error!("{}", err);
                // let (header_sender, _) = move_once.take().unwrap();
                // header_sender.send(Err(QueryingError::NetworkError(err)));
                std::ops::ControlFlow::Break(())
            }
        }
    });
    promise
}

pub(crate) fn remote_compute_query_aux_old(
    ctx: &egui::Context,
    api_addr: &str,
    single: &ComputeConfigQuery,
    script: QueryContent,
) -> Promise<Result<Resource<Result<ComputeResults, QueryingError>>, String>> {
    // TODO multi requests from client
    // if single.len > 1 {
    //     let parents = fetch_commit_parents(&ctx, &single.commit, single.len);
    // }
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/query/github/{}/{}/{}",
        api_addr, &single.commit.repo.user, &single.commit.repo.name, &single.commit.id,
    );
    wasm_rs_dbg::dbg!(&url, &script);

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

pub(crate) fn remote_compute_query_differential(
    ctx: &egui::Context,
    api_addr: &str,
    single: &ComputeConfigQueryDifferential,
    script: QueryContent,
) -> Promise<Result<Resource<Result<DetailsResults, QueryingError>>, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    assert_eq!(single.baseline.repo, single.commit.repo);
    let url = format!(
        "http://{}/query-differential/github/{}/{}/{}/{}",
        api_addr,
        &single.commit.repo.user,
        &single.commit.repo.name,
        &single.commit.id,
        &single.baseline.id,
    );
    wasm_rs_dbg::dbg!(&url, &script);

    let mut request = ehttp::Request::post(&url, serde_json::to_vec(&script).unwrap());
    request.headers.insert(
        "Content-Type".to_string(),
        "application/json; charset=utf-8".to_string(),
    );

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        let resource = response.and_then(|response| {
            Resource::<Result<DetailsResults, QueryingError>>::from_response(&ctx, response)
        });
        sender.send(resource);
    });
    promise
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum QueryingError {
    NetworkError(String),
    ProcessingError(String),
    MissingLanguage(String),
    ParsingError(String),
    MatchingErrOnFirst(MatchingError),
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Hash)]
pub enum MatchingError {
    TimeOut(utils_results_batched::ComputeResultIdentified),
    MaxMatches(utils_results_batched::ComputeResultIdentified),
}

impl utils_results_batched::PartialError<utils_results_batched::ComputeResultIdentified>
    for MatchingError
{
    fn error(&self) -> impl ToString {
        match self {
            MatchingError::TimeOut(_) => "time out",
            MatchingError::MaxMatches(_) => "max matches",
        }
    }

    fn try_partial(&self) -> Option<&utils_results_batched::ComputeResultIdentified> {
        match self {
            MatchingError::TimeOut(x) => Some(x),
            MatchingError::MaxMatches(x) => Some(x),
        }
    }
}

impl std::fmt::Display for MatchingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self) // TODO something better
    }
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
    let api_endpoint = &end_point(api_addr);
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

pub(crate) fn end_point(api_addr: &str) -> String {
    format!("{}/sharing-queries", api_addr)
}

pub(crate) fn handle_interactions(
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
        code_editors.current = EditStatus::Sharing(content.clone());
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
        code_editors.current = EditStatus::Local { name, content };
    } else if interaction.compute_button.clicked() {
        *trigger_compute |= true;
    }
}

pub(crate) fn show_scripts_edition(
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
            if let EditStatus::Example { i, .. } = &querying_context.current {
                if &j == i {
                    text = text.strong();
                }
            }
            let button = &ui.button(text);
            if button.clicked() {
                single.commit = (&ex.commit).into();
                single.config = ex.config;
                single.len = ex.commits;
                querying_context.current = EditStatus::Example {
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
            let json = match serde_json::from_str::<QueryingError>(text) {
                Ok(json) => json,
                Err(err) => {
                    log::error!("error converting QueryError: {}", err);
                    return Err(text.to_string());
                }
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

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct DetailsResults {
    pub prepare_time: f64,
    pub results: Vec<(super::types::CodeRange, super::types::CodeRange)>,
}

impl std::hash::Hash for DetailsResults {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.results.hash(state);
    }
}

impl Resource<Result<DetailsResults, QueryingError>> {
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
            let json = match serde_json::from_str::<QueryingError>(text) {
                Ok(json) => json,
                Err(err) => {
                    log::error!("error converting QueryError: {}", err);
                    return Err(text.to_string());
                }
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

pub(crate) const WANTED: SelectedConfig = SelectedConfig::Querying;

pub(crate) fn show_config(ui: &mut egui::Ui, single: &mut Sharing<ComputeConfigQuery>) {
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

impl utils_results_batched::ComputeError for QueryingError {
    fn head(&self) -> &str {
        match self {
            QueryingError::NetworkError(_) => "Network Error:",
            QueryingError::MissingLanguage(_) => "Missing Language:",
            QueryingError::ProcessingError(_) => "Processing Error:",
            QueryingError::ParsingError(_) => "Parsing Error:",
            QueryingError::MatchingErrOnFirst(MatchingError::TimeOut(_)) => {
                "Timed out on first commit:"
            }
            QueryingError::MatchingErrOnFirst(MatchingError::MaxMatches(_)) => {
                "Too many matches on first commit:"
            }
        }
    }

    fn content(&self) -> &str {
        match self {
            QueryingError::NetworkError(err) => err,
            QueryingError::MissingLanguage(err) => err,
            QueryingError::ProcessingError(err) => err,
            QueryingError::ParsingError(err) => err,
            QueryingError::MatchingErrOnFirst(MatchingError::TimeOut(res)) => "",
            QueryingError::MatchingErrOnFirst(MatchingError::MaxMatches(res)) => "",
        }
    }
}
