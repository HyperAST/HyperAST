use std::{
    ops::DerefMut,
    sync::{Arc, Mutex, RwLock},
};

use automerge::sync::SyncDoc;
use futures_util::SinkExt;
use poll_promise::Promise;

use crate::app::{crdt_over_ws, utils, API_URL};

use self::example_scripts::EXAMPLES;

use egui_addon::{
    code_editor::EditorInfo,
    egui_utils::{radio_collapsing, show_wip},
    interactive_split::interactive_splitter::InteractiveSplitter,
};

use super::{
    code_editor_automerge,
    crdt_over_ws::{DocSharingState, SharedDocView},
    show_repo_menu,
    types::{CodeEditors, Commit, Resource, SelectedConfig},
};
mod example_scripts;

const INFO_INIT: EditorInfo<&'static str> = EditorInfo {
    title: "Init",
    short: "initializes the accumulator on the root node",
    long: concat!("It will recieve the finally results of the entire computation."),
};
const INFO_FILTER:EditorInfo<&'static str> = EditorInfo {
    title: "Filter",
    short: "filters nodes of the HyperAST that should be processed",
    long: concat!("It goes through nodes in pre-order, returning the list of node that should be processed next and initializing their own states.\n","`s` is the current node accumulator")
    ,
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

// TODO allow to change user name and generate a random default
#[cfg(target_arch = "wasm32")]
pub(crate) const USER: &str = "web";
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const USER: &str = "native";

impl<C> From<&example_scripts::Scripts> for CodeEditors<C>
where
    C: From<(EditorInfo<String>, String)> + egui_addon::code_editor::CodeHolder,
{
    fn from(value: &example_scripts::Scripts) -> Self {
        let mut description: C = (INFO_DESCRIPTION.copied(), value.description.into()).into();
        description.set_lang("md".to_string());
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

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(super) struct ComputeConfigSingle {
    commit: Commit,
    config: example_scripts::Config,
    len: usize,
    #[serde(skip)]
    rt: crdt_over_ws::Rt,
    #[serde(skip)]
    ws: Option<crdt_over_ws::WsDoc>,
    #[serde(skip)]
    doc_db: Option<crdt_over_ws::WsDocsDb>,
}

impl Default for ComputeConfigSingle {
    fn default() -> Self {
        let rt = Default::default();
        // let quote = Default::default();
        let ws = None;
        let doc_db = None;
        Self {
            commit: From::from(&example_scripts::EXAMPLES[0].commit),
            config: example_scripts::EXAMPLES[0].config,
            // commit: "4acedc53a13a727be3640fe234f7e261d2609d58".into(),
            len: example_scripts::EXAMPLES[0].commits,
            rt,
            ws,
            doc_db,
        }
    }
}

pub(super) type RemoteResult =
    Promise<ehttp::Result<Resource<Result<ComputeResults, ScriptingError>>>>;

type SharedCodeEditors = std::sync::Arc<Mutex<CodeEditors<code_editor_automerge::CodeEditor>>>;

type ScriptingContext = super::ScriptingContext<
    super::types::CodeEditors,
    super::types::CodeEditors<code_editor_automerge::CodeEditor>,
>;

pub(super) fn remote_compute_single(
    ctx: &egui::Context,
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
        "{}/script-depth/github/{}/{}/{}",
        API_URL, &single.commit.repo.user, &single.commit.repo.name, &single.commit.id,
    );
    #[derive(serde::Serialize)]
    struct ScriptContent {
        init: String,
        filter: String,
        accumulate: String,
        commits: usize,
    }
    let script = match &mut code_editors.current {
        super::EditStatus::Shared(_, shared_script) | super::EditStatus::Sharing(shared_script) => {
            let code_editors = shared_script.lock().unwrap();
            ScriptContent {
                init: code_editors.init.code().to_string(),
                filter: code_editors.filter.code().to_string(),
                accumulate: code_editors.accumulate.code().to_string(),
                commits: single.len,
            }
        }
        super::EditStatus::Local { name: _, content }
        | super::EditStatus::Example { i: _, content } => ScriptContent {
            init: content.init.code().to_string(),
            filter: content.filter.code().to_string(),
            accumulate: content.accumulate.code().to_string(),
            commits: single.len,
        },
    };
    drop(code_editors);

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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ComputeResults {
    pub prepare_time: f64,
    pub results: Vec<Result<ComputeResultIdentified, String>>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ComputeResultIdentified {
    pub commit: super::types::CommitId,
    #[serde(flatten)]
    pub inner: ComputeResult,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ComputeResult {
    pub compute_time: f64,
    pub result: serde_json::Value,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum ScriptingError {
    AtCompilation(String),
    AtEvaluation(String),
    Other(String),
}

pub(super) fn show_single_repo(
    ui: &mut egui::Ui,
    single: &mut ComputeConfigSingle,
    code_editors: &mut ScriptingContext,
    trigger_compute: &mut bool,
    compute_single_result: &mut Option<
        poll_promise::Promise<
            Result<super::types::Resource<Result<ComputeResults, ScriptingError>>, String>,
        >,
    >,
) {
    if let Some(doc_db) = &mut single.doc_db {
        let ctx = ui.ctx().clone();
        let rt = single.rt.clone();
        let owner = USER.to_string();
        let data = doc_db.data.clone();
        if let Err(err) = doc_db.setup_atempt(
            move |sender, receiver| rt.spawn(db_update_handler(sender, receiver, owner, ctx, data)),
            &single.rt,
        ) {
            log::warn!("{}", err);
            if ui.button("try restarting sharing connection").clicked() {
                let url = format!("ws://{}/shared-scripts-db", &API_URL[7..]);
                *doc_db = crdt_over_ws::WsDocsDb::new(
                    &single.rt,
                    USER.to_string(),
                    ui.ctx().clone(),
                    url,
                );
            }
        }
        match &mut code_editors.current {
            super::EditStatus::Sharing(shared_script) => {
                let ctx = ui.ctx().clone();
                let rt = single.rt.clone();
                let db = doc_db;
                let guard = &mut db.data.write().unwrap();
                let (waiting, vec) = guard.deref_mut();
                if let Some(i) = waiting {
                    wasm_rs_dbg::dbg!();
                    let i = *i;
                    if let Some(Some(view)) = vec.get(i) {
                        assert_eq!(view.id, i);
                        let url = format!("ws://{}/shared-script/{}", &API_URL[7..], i);
                        single.ws = Some(crdt_over_ws::WsDoc::new(&rt, USER.to_string(), ctx, url));
                    }
                    code_editors.current = super::EditStatus::Shared(i, shared_script.clone());
                }
            }
            super::EditStatus::Shared(_, shared_script) => {
                if let Some(ws) = &mut single.ws {
                    wasm_rs_dbg::dbg!();
                    let ctx = ui.ctx().clone();
                    let doc = ws.data.clone();
                    let rt = single.rt.clone();
                    let code_editors = shared_script.clone();
                    if let Err(e) = ws.setup_atempt(
                        |sender, receiver| {
                            single.rt.spawn(update_handler(
                                receiver,
                                sender,
                                doc,
                                ctx,
                                rt,
                                code_editors,
                            ))
                        },
                        &single.rt,
                    ) {
                        log::error!("{}", e);
                    }
                }
            }
            _ => (),
        }
    } else {
        let url = format!("ws://{}/shared-scripts-db", &API_URL[7..]);
        single.doc_db = Some(crdt_over_ws::WsDocsDb::new(
            &single.rt,
            USER.to_string(),
            ui.ctx().clone(),
            url,
        ));
    }
    let is_portrait = ui.available_rect_before_wrap().aspect_ratio() < 1.0;
    if is_portrait {
        egui::ScrollArea::vertical().show(ui, |ui| {
            show_scripts_edition(ui, code_editors, single);
            handle_interactions(
                ui,
                code_editors,
                compute_single_result,
                single,
                trigger_compute,
            );
            show_long_result(&*compute_single_result, ui);
        });
    } else {
        InteractiveSplitter::vertical()
            .ratio(0.7)
            .show(ui, |ui1, ui2| {
                ui1.push_id(ui1.id().with("input"), |ui| {
                    show_scripts_edition(ui, code_editors, single);
                });
                let ui = ui2;
                handle_interactions(
                    ui,
                    code_editors,
                    compute_single_result,
                    single,
                    trigger_compute,
                );
                show_long_result(&*compute_single_result, ui);
            });
    }
}

fn handle_interactions(
    ui: &mut egui::Ui,
    code_editors: &mut super::ScriptingContext<
        CodeEditors,
        CodeEditors<code_editor_automerge::CodeEditor>,
    >,
    compute_single_result: &mut Option<
        Promise<Result<Resource<Result<ComputeResults, ScriptingError>>, String>>,
    >,
    single: &mut ComputeConfigSingle,
    trigger_compute: &mut bool,
) {
    let interaction = show_interactions(ui, code_editors, &single.doc_db, compute_single_result);
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
        log::warn!("saving script: {:#?}", content.clone());
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

fn show_interactions<'a>(
    ui: &mut egui::Ui,
    code_editors: &'a mut ScriptingContext,
    docs_db: &Option<crdt_over_ws::WsDocsDb>,
    compute_single_result: &mut Option<
        poll_promise::Promise<
            Result<super::types::Resource<Result<ComputeResults, ScriptingError>>, String>,
        >,
    >,
) -> InteractionResp<'a> {
    let mut save_button = None;
    let mut share_button = None;
    let mut editor: Option<(&str, &CodeEditors)> = None;
    ui.horizontal(|ui| match &mut code_editors.current {
        super::EditStatus::Example { i, content } => {
            save_button = Some(ui.add(egui::Button::new("Save Script")));
            let name = &EXAMPLES[*i].name;
            editor = Some((name, &*content));
        }
        super::EditStatus::Local { name, content } => {
            if let Some(doc_db) = docs_db {
                if doc_db.is_connected() {
                    share_button = Some(ui.add(egui::Button::new("Share Script")));
                }
            }
            save_button = Some(ui.add(egui::Button::new("Save Script")));
            ui.text_edit_singleline(name);
            editor = Some((name, &*content));
        }
        _ => (),
    });
    let compute_button = ui
        .horizontal(|ui| {
            let compute_button = ui.add(egui::Button::new("Compute"));
            show_short_result(&*compute_single_result, ui);
            compute_button
        })
        .inner;

    InteractionResp {
        compute_button,
        editor,
        save_button,
        share_button,
    }
}

struct InteractionResp<'a> {
    compute_button: egui::Response,
    save_button: Option<egui::Response>,
    share_button: Option<egui::Response>,
    editor: Option<(&'a str, &'a CodeEditors)>,
}

fn show_scripts_edition(
    ui: &mut egui::Ui,
    scripting_context: &mut ScriptingContext,
    single: &mut ComputeConfigSingle,
) {
    show_available_stuff(ui, single, scripting_context);
    match &mut scripting_context.current {
        super::EditStatus::Example {
            i: _,
            content: code_editors,
        } => {
            show_local_code_edition(ui, code_editors, single);
        }
        super::EditStatus::Local {
            name: _,
            content: code_editors,
        } => {
            show_local_code_edition(ui, code_editors, single);
        }
        super::EditStatus::Sharing(code_editors) => {
            show_shared_code_edition(ui, code_editors, single);
        }
        super::EditStatus::Shared(_, code_editors) => {
            show_shared_code_edition(ui, code_editors, single);
        }
    }
}

fn show_shared_code_edition(
    ui: &mut egui::Ui,
    code_editors: &mut SharedCodeEditors,
    single: &mut ComputeConfigSingle,
) {
    let resps = {
        let mut ce = code_editors.lock().unwrap();
        [
            ce.description.ui(ui),
            ce.init.ui(ui),
            ce.filter.ui(ui),
            ce.accumulate.ui(ui),
        ]
    };

    let Some(ws) = &mut single.ws else {
        return;
    };
    if resps.iter().filter_map(|x| x.as_ref()).any(|x| x.changed()) {
        let timer = if ws.timer != 0.0 {
            let dt = ui.input(|mem| mem.unstable_dt);
            ws.timer + dt
        } else {
            0.01
        };
        let rt = &single.rt;
        timed_updater(timer, ws, ui, code_editors, rt);
    } else if ws.timer != 0.0 {
        let dt = ui.input(|mem| mem.unstable_dt);
        let timer = ws.timer + dt;
        let rt = &single.rt;
        timed_updater(timer, ws, ui, code_editors, rt);
    }
}

fn show_local_code_edition(
    ui: &mut egui::Ui,
    code_editors: &mut CodeEditors,
    _single: &mut ComputeConfigSingle,
) {
    let _resps = {
        let ce = code_editors;
        [
            ce.description.ui(ui),
            ce.init.ui(ui),
            ce.filter.ui(ui),
            ce.accumulate.ui(ui),
        ]
    };
}

fn show_available_stuff(
    ui: &mut egui::Ui,
    single: &mut ComputeConfigSingle,
    scripting_context: &mut ScriptingContext,
) {
    egui::warn_if_debug_build(ui);
    egui::CollapsingHeader::new("Examples")
        .default_open(true)
        .show(ui, |ui| show_examples(ui, single, scripting_context));
    if !scripting_context.local_scripts.is_empty() {
        egui::CollapsingHeader::new("Local Scripts")
            .default_open(true)
            .show(ui, |ui| show_locals(ui, single, scripting_context));
    }
    if let Some(doc_db) = &single.doc_db {
        let names: Vec<_> = doc_db
            .data
            .read()
            .unwrap()
            .1
            .iter()
            .filter_map(|d| d.as_ref())
            .map(|x| (format!("{}/{}", x.owner, x.name), x.id))
            .collect();
        if !names.is_empty() {
            egui::CollapsingHeader::new("Shared Scripts")
                .default_open(true)
                .show(ui, |ui| show_shared(ui, single, scripting_context, names));
        }
    }
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
            if let super::EditStatus::Example { i, .. } = &scripting_context.current {
                if &j == i {
                    text = text.strong();
                }
            }
            let button = &ui.button(text);
            if button.clicked() {
                single.commit = (&ex.commit).into();
                single.config = ex.config;
                single.len = ex.commits;
                scripting_context.current = super::EditStatus::Example {
                    i: j,
                    content: (&ex.scripts).into(),
                };
            }
            if button.hovered() {
                egui::show_tooltip(ui.ctx(), button.id.with("tooltip"), |ui| {
                    let desc = ex.scripts.description;
                    egui_demo_lib::easy_mark::easy_mark(ui, desc);
                });
            }
            j += 1;
        }
    });
}

fn show_locals(
    ui: &mut egui::Ui,
    single: &mut ComputeConfigSingle,
    scripting_context: &mut ScriptingContext,
) {
    ui.horizontal_wrapped(|ui| {
        // let mut n = None;
        for (name, s) in &scripting_context.local_scripts {
            let mut text = egui::RichText::new(name);
            if let super::EditStatus::Local {
                name: n,
                content: _,
            } = &scripting_context.current
            {
                if name == n {
                    text = text.strong();
                }
            }
            let button = ui.button(text);
            if button.clicked() {
                // res = Some(ex);
                scripting_context.current = super::EditStatus::Local {
                    name: name.clone(),
                    content: s.clone(),
                };
            }
            if button.hovered() {
                egui::show_tooltip(ui.ctx(), button.id.with("tooltip"), |ui| {
                    let desc = s.description.code();
                    egui_demo_lib::easy_mark::easy_mark(ui, desc);
                });
            }
            button.context_menu(|ui| {
                if ui.button("share").clicked() {
                    let content = s.clone().to_shared();
                    let content = Arc::new(Mutex::new(content));
                    scripting_context.current =
                        super::EditStatus::Shared(usize::MAX, content.clone());
                    let mut content = content.lock().unwrap();
                    single.doc_db.as_mut().unwrap().create_doc_atempt(
                        &single.rt,
                        name.to_string(),
                        content.deref_mut(),
                    );
                }
                // let rename_button = &ui.button("rename");
                // if rename_button.clicked() {
                //     let popup_id = ui.make_persistent_id("rename popup");
                //     ui.memory_mut(|mem| mem.open_popup(popup_id));
                //     let below = egui::AboveOrBelow::Below;
                //     egui::popup::popup_above_or_below_widget(ui, popup_id, &rename_button, below, |ui| {
                //         let mut new = name.clone();
                //         if ui.text_edit_singleline(&mut new).lost_focus() && name != &new {
                //             n = Some((name.clone(),new));
                //         }
                //         if ui.button("abort rename").clicked() {
                //             ui.memory_mut(|mem| mem.close_popup());
                //         }
                //     });
                // }
                // if ui.button("fork").clicked() {
                // }
                if ui.button("close menu").clicked() {
                    ui.close_menu()
                }
            });
        }
        // if let Some((old,new)) = n {
        //     let value = scripting_context.local_scripts.remove(&old).unwrap();
        //     scripting_context.local_scripts.insert(new, value);
        // };
    });
}

fn show_shared(
    ui: &mut egui::Ui,
    single: &mut ComputeConfigSingle,
    scripting_context: &mut ScriptingContext,
    names: Vec<(String, usize)>,
) {
    let mut r = None;
    ui.horizontal_wrapped(|ui| {
        for (name, i) in names.iter() {
            let mut text = egui::RichText::new(name);
            if let super::EditStatus::Shared(j, _) = &scripting_context.current {
                if j == i {
                    text = text.strong();
                }
            }
            if ui.button(text).clicked() {
                r = Some(i);
            }
        }
    });
    if let Some(i) = r {
        let code_editors: Arc<Mutex<CodeEditors<code_editor_automerge::CodeEditor>>> =
            Default::default();
        scripting_context.current = super::EditStatus::Shared(*i, code_editors.clone());
        let doc_db = single.doc_db.as_ref().unwrap();
        let doc_views = doc_db.data.write().unwrap();
        let id = doc_views.1.get(*i).unwrap().as_ref().unwrap().id;
        if let Some(ws) = &mut single.ws {
            let ctx = ui.ctx().clone();
            let rt = single.rt.clone();
            let url = format!("ws://{}/shared-script/{}", &API_URL[7..], id);
            *ws = crdt_over_ws::WsDoc::new(&rt, USER.to_string(), ctx, url)
        }
    }
}
fn timed_updater(
    timer: f32,
    ws: &mut crdt_over_ws::WsDoc,
    ui: &mut egui::Ui,
    code_editors: &mut SharedCodeEditors,
    rt: &crdt_over_ws::Rt,
) {
    const TIMER: u64 = 1;
    if timer < std::time::Duration::from_secs(TIMER).as_secs_f32() {
        ws.timer = timer;
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f32(TIMER as f32));
    } else {
        ws.timer = 0.0;
        let quote: &mut CodeEditors<
            crate::app::code_editor_automerge::CodeEditor<crdt_over_ws::Quote>,
        > = &mut code_editors.lock().unwrap();
        ws.changed(rt, quote);
    }
}

async fn update_handler(
    mut receiver: futures_util::stream::SplitStream<tokio_tungstenite_wasm::WebSocketStream>,
    mut sender: futures::channel::mpsc::Sender<tokio_tungstenite_wasm::Message>,
    doc: std::sync::Arc<std::sync::RwLock<DocSharingState>>,
    ctx: egui::Context,
    rt: crdt_over_ws::Rt,
    code_editors: SharedCodeEditors,
) {
    use futures_util::StreamExt;
    #[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
    enum DbMsgToServer {
        Create { name: String },
        User { name: String },
    }
    let owner = USER.to_string();
    sender
        .send(tokio_tungstenite_wasm::Message::Text(
            serde_json::to_string(&DbMsgToServer::User { name: owner }).unwrap(),
        ))
        .await
        .unwrap();
    match receiver.next().await {
        Some(Ok(tokio_tungstenite_wasm::Message::Binary(bin))) => {
            let (doc, sync_state): &mut (_, _) = &mut doc.write().unwrap();
            let message = automerge::sync::Message::decode(&bin).unwrap();
            doc.sync()
                .receive_sync_message(sync_state, message)
                .unwrap();
            wasm_rs_dbg::dbg!(&doc);
            if let Ok(t) = autosurgeon::hydrate(&*doc) {
                let mut text = code_editors.lock().unwrap();
                *text = t;
            }
            ctx.request_repaint();
        }
        _ => (),
    }
    while let Some(Ok(msg)) = receiver.next().await {
        wasm_rs_dbg::dbg!();
        match msg {
            tokio_tungstenite_wasm::Message::Text(msg) => {
                wasm_rs_dbg::dbg!(&msg);
            }
            tokio_tungstenite_wasm::Message::Binary(bin) => {
                wasm_rs_dbg::dbg!();
                let (doc, sync_state): &mut (_, _) = &mut doc.write().unwrap();
                let message = automerge::sync::Message::decode(&bin).unwrap();
                // doc.merge(other)
                match doc.sync().receive_sync_message(sync_state, message) {
                    Ok(_) => (),
                    Err(e) => {
                        wasm_rs_dbg::dbg!(e);
                    }
                }
                match autosurgeon::hydrate(doc) {
                    Ok(t) => {
                        let mut text = code_editors.lock().unwrap();
                        *text = t;
                    }
                    Err(e) => {
                        wasm_rs_dbg::dbg!(e);
                    }
                }
                ctx.request_repaint();

                wasm_rs_dbg::dbg!();
                let mut sender = sender.clone();
                if let Some(message) = doc.sync().generate_sync_message(sync_state) {
                    wasm_rs_dbg::dbg!();
                    use futures_util::SinkExt;
                    let message =
                        tokio_tungstenite_wasm::Message::Binary(message.encode().to_vec());
                    rt.spawn(async move {
                        sender.send(message).await.unwrap();
                    });
                } else {
                    wasm_rs_dbg::dbg!();
                    use futures_util::SinkExt;
                    let message = tokio_tungstenite_wasm::Message::Binary(vec![]);
                    rt.spawn(async move {
                        sender.send(message).await.unwrap();
                    });
                };
            }
            tokio_tungstenite_wasm::Message::Close(_) => {
                wasm_rs_dbg::dbg!();
                break;
            }
        }
    }
}

async fn db_update_handler(
    mut sender: futures::channel::mpsc::Sender<tokio_tungstenite_wasm::Message>,
    mut receiver: futures_util::stream::SplitStream<tokio_tungstenite_wasm::WebSocketStream>,
    owner: String,
    ctx: egui::Context,
    data: Arc<RwLock<(Option<usize>, Vec<Option<SharedDocView>>)>>,
) {
    use futures_util::{Future, SinkExt, StreamExt};
    use serde::{Deserialize, Serialize};
    type User = String;

    #[derive(Deserialize, Serialize, Debug, Clone)]
    enum DbMsgToServer {
        Create { name: String },
        User { name: String },
    }
    {
        wasm_rs_dbg::dbg!();
        let name = owner.clone();
        let msg = DbMsgToServer::User { name };
        let msg = serde_json::to_string(&msg).unwrap();
        let msg = tokio_tungstenite_wasm::Message::Text(msg);
        sender.send(msg).await.unwrap();
        wasm_rs_dbg::dbg!();
    }
    while let Some(Ok(msg)) = receiver.next().await {
        wasm_rs_dbg::dbg!();
        match msg {
            tokio_tungstenite_wasm::Message::Text(msg) => {
                wasm_rs_dbg::dbg!(&msg);

                #[derive(Deserialize, Serialize, Debug, Clone)]
                enum DbMsgFromServer {
                    Add(SharedDocView),
                    AddWriter(usize, User),
                    RmWriter(usize, User),
                    // Rename(usize, String),
                    Reset { all: Vec<SharedDocView> },
                }
                let msg = serde_json::from_str(&msg).unwrap();

                match msg {
                    DbMsgFromServer::Add(x) => {
                        let b = x.owner == owner;
                        let guard = &mut data.write().unwrap();
                        let (waiting, vec) = guard.deref_mut();
                        let id = x.id;
                        vec.resize(id + 1, None);
                        vec[id] = Some(x);
                        if b {
                            *waiting = Some(id);
                        }
                        ctx.request_repaint();
                    }
                    DbMsgFromServer::AddWriter(_, _) => todo!(),
                    DbMsgFromServer::RmWriter(_, _) => todo!(),
                    DbMsgFromServer::Reset { all } => {
                        let guard = &mut data.write().unwrap();
                        let (_, vec) = guard.deref_mut();
                        *vec = vec![];
                        for x in all {
                            let id = x.id;
                            vec.resize(id + 1, None);
                            vec[id] = Some(x);
                        }
                        ctx.request_repaint();
                    }
                }
            }
            tokio_tungstenite_wasm::Message::Binary(bin) => {
                wasm_rs_dbg::dbg!();
            }
            tokio_tungstenite_wasm::Message::Close(_) => {
                wasm_rs_dbg::dbg!();
                break;
            }
        }
    }
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
                return Err("".to_string())
            };
            let Ok(json) = serde_json::from_str::<ScriptingError>(text) else {
                wasm_rs_dbg::dbg!();
                return Err("".to_string())
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

pub(super) fn show_single_repo_menu(
    ui: &mut egui::Ui,
    selected: &mut SelectedConfig,
    single: &mut ComputeConfigSingle,
) {
    let title = "Single Repository";
    let wanted = SelectedConfig::Single;
    let id = ui.make_persistent_id(title);
    let add_body = |ui: &mut egui::Ui| {
        show_repo_menu(ui, &mut single.commit.repo);
        ui.push_id(ui.id().with("commit"), |ui| {
            egui::TextEdit::singleline(&mut single.commit.id)
                .clip_text(true)
                .desired_width(150.0)
                .desired_rows(1)
                .hint_text("commit")
                .interactive(true)
                .show(ui)
        });

        ui.add_enabled_ui(true, |ui| {
            ui.add(
                egui::Slider::new(&mut single.len, 1..=200)
                    .text("commits")
                    .clamp_to_range(false)
                    .integer()
                    .logarithmic(true),
            );
            // show_wip(ui, Some("only process one commit"));
        });
        let mut selected = &mut single.config;
        egui::ComboBox::from_label("Repo Config")
            .selected_text(format!("{:?}", selected))
            .show_ui(ui, |ui| {
                ui.selectable_value(selected, example_scripts::Config::Any, "Any");
                ui.selectable_value(selected, example_scripts::Config::MavenJava, "Java");
                ui.selectable_value(selected, example_scripts::Config::MakeCpp, "Cpp");
            });
    };

    radio_collapsing(ui, id, title, selected, &wanted, add_body);
}

pub(super) fn show_short_result(promise: &Option<RemoteResult>, ui: &mut egui::Ui) {
    if let Some(promise) = &promise {
        if let Some(result) = promise.ready() {
            match result {
                Ok(resource) => {
                    // ui_resource(ui, resource);
                    dbg!(&resource.response);
                    if let Some(content) = &resource.content {
                        match content {
                            Ok(content) => {
                                if ui.add(egui::Button::new("Export")).clicked() {
                                    if let Ok(text) = serde_json::to_string_pretty(content) {
                                        utils::file_save(&text)
                                    }
                                };
                                if content.results.len() == 1 {
                                    if let Ok(res) = &content.results[0] {
                                        ui.label(format!(
                                            "compute time: {:.3}",
                                            SecFmt(content.prepare_time + res.inner.compute_time)
                                        ));
                                    }
                                } else {
                                    ui.label(format!(
                                        "compute time: {:.3} + {:.3}",
                                        SecFmt(content.prepare_time),
                                        SecFmt(
                                            content
                                                .results
                                                .iter()
                                                .filter_map(|x| x.as_ref().ok())
                                                .map(|x| x.inner.compute_time)
                                                .sum::<f64>()
                                        )
                                    ));
                                }
                            }
                            Err(_) => {
                                ui.label(format!("compute time: N/A"));
                            }
                        }
                    }
                }
                Err(_) => {
                    ui.label(format!("compute time: N/A"));
                }
            }
        } else {
            ui.label(format!("compute time: "));
            ui.spinner();
        }
    }
}

pub(super) fn show_long_result(promise: &Option<RemoteResult>, ui: &mut egui::Ui) {
    if let Some(promise) = &promise {
        if let Some(result) = promise.ready() {
            match result {
                Ok(resource) => {
                    // ui_resource(ui, resource);
                    dbg!(&resource.response);
                    if let Some(content) = &resource.content {
                        match content {
                            Ok(content) => {
                                show_long_result_success(ui, content);
                            }
                            Err(error) => {
                                show_long_result_compute_failure(error, ui);
                            }
                        }
                    }
                }
                Err(error) => {
                    // This should only happen if the fetch API isn't available or something similar.
                    ui.colored_label(
                        ui.visuals().error_fg_color,
                        if error.is_empty() { "Error" } else { error },
                    );
                }
            }
        } else {
            ui.spinner();
        }
    } else {
        ui.label("click on Compute");
    }
}

fn show_long_result_compute_failure(error: &ScriptingError, ui: &mut egui::Ui) {
    let (h, c) = match error {
        ScriptingError::AtCompilation(err) => ("Error at compilation:", err),
        ScriptingError::AtEvaluation(err) => ("Error at evaluation:", err),
        ScriptingError::Other(err) => ("Error somewhere else:", err),
    };
    ui.label(
        egui::RichText::new(h)
            .heading()
            .color(ui.visuals().error_fg_color),
    );
    ui.colored_label(ui.visuals().error_fg_color, c);
}

fn show_long_result_success(ui: &mut egui::Ui, content: &ComputeResults) {
    if content.results.len() > 5 {
        egui::ScrollArea::horizontal()
            .always_show_scroll(true)
            .auto_shrink([false, false])
            .show(ui, |ui| show_long_result_table(content, ui));
    } else {
        egui::CollapsingHeader::new("Results (JSON)")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .always_show_scroll(false)
                    .auto_shrink([false, false])
                    .show(ui, |ui| show_long_result_list(content, ui));
            });
    }
}

fn show_long_result_list(content: &ComputeResults, ui: &mut egui::Ui) {
    for cont in &content.results {
        match cont {
            Ok(cont) => {
                let mut code: &str = &serde_json::to_string_pretty(&cont.inner.result).unwrap();
                let language = "json";
                let theme = egui_demo_lib::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
                let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                    let layout_job = egui_demo_lib::syntax_highlighting::highlight(
                        ui.ctx(),
                        &theme,
                        string,
                        language,
                    );
                    // layout_job.wrap.max_width = wrap_width; // no wrapping
                    ui.fonts(|f| f.layout_job(layout_job))
                };
                if content.results.len() > 1 {
                    ui.label(format!(
                        "compute time: {:.3}",
                        SecFmt(cont.inner.compute_time)
                    ));
                }
                ui.add(
                    egui::TextEdit::multiline(&mut code)
                        .font(egui::TextStyle::Monospace) // for cursor height
                        .code_editor()
                        .desired_rows(1)
                        .lock_focus(true)
                        .layouter(&mut layouter),
                );
            }
            Err(err) => {
                ui.colored_label(ui.visuals().error_fg_color, err);
            }
        }
    }
}

fn show_long_result_table(content: &ComputeResults, ui: &mut egui::Ui) {
    // header
    let header = content.results.iter().find(|x| x.is_ok());
    let Some(header) = header
        .as_ref() else {
            wasm_rs_dbg::dbg!("issue with header");
            return;
        };
    let header = header.as_ref().unwrap();
    use egui_extras::{Column, TableBuilder};
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .auto_shrink([true, true])
        .column(Column::auto().resizable(true).clip(false))
        // .column(Column::remainder())
        .columns(
            Column::auto().resizable(true),
            header.inner.result.as_object().unwrap().len(),
        )
        .column(Column::auto().resizable(true).clip(false))
        .header(20.0, |mut head| {
            let hf = |ui: &mut egui::Ui, name| {
                ui.label(
                    egui::RichText::new(name)
                        .size(15.0)
                        .text_style(egui::TextStyle::Monospace),
                )
            };
            head.col(|ui| {
                hf(ui, " commit");
            });
            for (name, _) in header.inner.result.as_object().unwrap().iter() {
                head.col(|ui| {
                    hf(ui, name);
                });
            }
            head.col(|ui| {
                hf(ui, "compute time");
            });
            // head.col(|ui| {
            //     ui.heading("First column");
            // });
            // head.col(|ui| {
            //     ui.heading("Second column");
            // });
        })
        .body(|mut body| {
            for cont in &content.results {
                match cont {
                    Ok(cont) => {
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.label(&cont.commit[..8]);
                            });
                            for (_, v) in cont.inner.result.as_object().unwrap() {
                                row.col(|ui| {
                                    // ui.button(v.to_string());
                                    ui.label(v.to_string());
                                });
                            }
                            row.col(|ui| {
                                ui.label(format!("{:.3}", SecFmt(cont.inner.compute_time)));
                            });
                        });
                    }
                    Err(err) => {
                        body.row(30.0, |mut row| {
                            row.col(|ui| {
                                ui.colored_label(ui.visuals().error_fg_color, err);
                            });
                        });
                    }
                }
            }
        });
}

struct SecFmt(f64);

impl From<f64> for SecFmt {
    fn from(value: f64) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for SecFmt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // f.precision()
        let x = self.0;
        let (t, n) = if x > 60.0 {
            let n = if f.alternate() { "minutes" } else { "m" };
            (x / 60.0, n)
        } else if x == 0.0 {
            let n = if f.alternate() { "seconds" } else { "s" };
            (x, n)
        } else if x < 0.00_000_000_001 {
            let n = if f.alternate() { "pico seconds" } else { "ps" };
            (x * 1_000_000_000_000., n)
        } else if x < 0.00_000_001 {
            let n = if f.alternate() { "nano seconds" } else { "ns" };
            (x * 1_000_000_000., n)
        } else if x < 0.00_001 {
            let n = if f.alternate() { "micro seconds" } else { "us" };
            (x * 1_000_000., n)
        } else if x < 1.0 {
            let n = if f.alternate() { "milli seconds" } else { "ms" };
            (x * 1_000., n)
        } else {
            let n = if f.alternate() { "seconds" } else { "s" };
            (x, n)
        };
        fn round_to_significant_digits3(number: f64, significant_digits: usize) -> String {
            if number == 0.0 {
                return format!("{:.*}", significant_digits, number);
            }
            let abs = number.abs();
            let d = if abs == 1.0 {
                1.0
            } else {
                (abs.log10().ceil()).max(0.0)
            };
            let power = significant_digits - d as usize;

            let magnitude = 10.0_f64.powi(power as i32);
            let shifted = number * magnitude;
            let rounded_number = shifted.round();
            let unshifted = rounded_number as f64 / magnitude;
            dbg!(
                number,
                (number.abs() + 0.000001).log10().ceil(),
                significant_digits,
                power,
                d
            );
            format!("{:.*}", power, unshifted)
        }
        if t == 0.0 {
            write!(f, "{:.1} {}", t, n)
        } else if let Some(prec) = f.precision() {
            write!(f, "{} {}", round_to_significant_digits3(t, prec), n)
        } else {
            write!(f, "{} {}", t, n)
        }
    }
}

#[test]
fn aaa() {
    assert_eq!(format!("{:.4}", SecFmt(0.0)), "0.0 s");
    assert_eq!(format!("{:.3}", SecFmt(1.0 / 1000.0)), "1.00 ms");
    assert_eq!(format!("{:.3}", SecFmt(1.0 / 1000.0 / 1000.0)), "1.00 us");
    assert_eq!(format!("{:.4}", SecFmt(0.00_000_000_1)), "1.000 ns");
    assert_eq!(format!("{:.4}", SecFmt(0.00_000_000_000_1)), "1.000 ps");
    assert_eq!(format!("{:.2}", SecFmt(0.0000000012)), "1.2 ns");
    assert_eq!(format!("{:.4}", SecFmt(10.43333)), "10.43 s");
    assert_eq!(format!("{:.3}", SecFmt(10.43333)), "10.4 s");
    assert_eq!(format!("{:.2}", SecFmt(10.43333)), "10 s");
    assert_eq!(format!("{:3e}", 10.43333), "1.043333e1");
}
