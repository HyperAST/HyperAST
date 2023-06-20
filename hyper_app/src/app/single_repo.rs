use automerge::sync::SyncDoc;
use poll_promise::Promise;

use crate::app::{utils, API_URL};

use self::example_scripts::EXAMPLES;

use egui_addon::{
    code_editor::EditorInfo,
    egui_utils::{radio_collapsing, show_wip},
    interactive_split::interactive_splitter::InteractiveSplitter,
};

// use super::code_editor_automerge::CodeEditor;
use super::{
    code_editor_automerge, show_repo_menu,
    types::{CodeEditors, Commit, Resource, SelectedConfig},
};
use egui_addon::code_editor::CodeEditor;

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

impl<C: From<(EditorInfo<String>, String)>> From<&example_scripts::Scripts> for CodeEditors<C> {
    fn from(value: &example_scripts::Scripts) -> Self {
        Self {
            init: (INFO_INIT.copied(), value.init.into()).into(),
            filter: (INFO_FILTER.copied(), value.filter.into()).into(),
            accumulate: (INFO_ACCUMULATE.copied(), value.accumulate.into()).into(),
            // auto: std::sync::Arc::new(std::sync::Mutex::new(
            //     crate::app::code_editor_automerge::CodeEditor {
            //         info: AUTOMERGE.copied(),
            //         ..value.accumulate.into()
            //     },
            // )),
        }
    }
}

impl<C: From<(EditorInfo<String>, String)>> Default for CodeEditors<C> {
    fn default() -> Self {
        (&example_scripts::EXAMPLES[0].scripts).into()
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(super) struct ComputeConfigSingle {
    commit: Commit,
    len: usize,
    #[serde(skip)]
    rt: exp::Rt,
    #[serde(skip)]
    ws: Option<exp::WsDoc>,
    // // ws: Option<exp::Ws>,
    // #[serde(skip)]
    // quote: exp::Quote,
}

impl Default for ComputeConfigSingle {
    fn default() -> Self {
        let rt = Default::default();
        // let quote = Default::default();
        let ws = None;
        Self {
            commit: From::from(&example_scripts::EXAMPLES[0].commit),
            // commit: "4acedc53a13a727be3640fe234f7e261d2609d58".into(),
            len: example_scripts::EXAMPLES[0].commits,
            rt,
            ws,
            // quote,
        }
    }
}

pub(super) type RemoteResult =
    Promise<ehttp::Result<Resource<Result<ComputeResults, ScriptingError>>>>;

pub(super) fn remote_compute_single(
    ctx: &egui::Context,
    single: &mut ComputeConfigSingle,
    code_editors: &mut std::sync::Arc<
        std::sync::Mutex<super::types::CodeEditors<code_editor_automerge::CodeEditor>>,
    >,
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
    let code_editors = code_editors.lock().unwrap();
    let script = ScriptContent {
        init: code_editors.init.code().to_string(),
        filter: code_editors.filter.code().to_string(),
        accumulate: code_editors.accumulate.code().to_string(),
        commits: single.len,
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
    code_editors: &mut std::sync::Arc<
        std::sync::Mutex<super::types::CodeEditors<code_editor_automerge::CodeEditor>>,
    >,
    trigger_compute: &mut bool,
    compute_single_result: &mut Option<
        poll_promise::Promise<
            Result<super::types::Resource<Result<ComputeResults, ScriptingError>>, String>,
        >,
    >,
) {
    if let Some(ws) = &mut single.ws {
        let ctx = ui.ctx().clone();
        // let mutex = single.quote.clone();
        let mutex = code_editors.clone();
        let doc = ws.doc.clone();
        let rt = single.rt.clone();
        ws.setup_atempt(
            |sender, mut receiver| {
                single.rt.spawn(async move {
                    use futures_util::StreamExt;
                    match receiver.next().await {
                        Some(Ok(tokio_tungstenite_wasm::Message::Binary(bin))) => {
                            let (doc, sync_state): &mut (_, _) = &mut doc.write().unwrap();
                            let message = automerge::sync::Message::decode(&bin).unwrap();
                            doc.sync()
                                .receive_sync_message(sync_state, message)
                                .unwrap();
                            wasm_rs_dbg::dbg!(&doc);
                            if let Ok(t) = autosurgeon::hydrate(&*doc) {
                                let mut text = mutex.lock().unwrap();
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
                                // let text = &mut mutex.lock().unwrap().code.text;
                                // text.splice(text.as_str().len(), 0, msg.to_string());
                                // text.splice(text.as_str().len(), 0, "\n");
                            }
                            tokio_tungstenite_wasm::Message::Binary(bin) => {
                                wasm_rs_dbg::dbg!();
                                let (doc, sync_state): &mut (_, _) = &mut doc.write().unwrap();
                                // let changes = automerge::Change::from_bytes(bin).into_iter();
                                // wasm_rs_dbg::dbg!(changes.clone().map(|x|x.decode()).collect::<Vec<_>>());
                                // doc.apply_changes(changes)
                                //     .unwrap();
                                let message = automerge::sync::Message::decode(&bin).unwrap();
                                // doc.merge(other)
                                match doc.sync().receive_sync_message(sync_state, message) {
                                    Ok(_) => (),
                                    Err(e) => {
                                        wasm_rs_dbg::dbg!(e);
                                        // doc
                                        // e.
                                    }
                                }
                                wasm_rs_dbg::dbg!(&doc);
                                match autosurgeon::hydrate(doc) {
                                    Ok(t) => {
                                        let mut text = mutex.lock().unwrap();
                                        *text = t;
                                    }
                                    Err(e) => {
                                        wasm_rs_dbg::dbg!(e);
                                    }
                                }
                                ctx.request_repaint();

                                wasm_rs_dbg::dbg!();
                                let mut sender = sender.clone();
                                if let Some(message) = doc.sync().generate_sync_message(sync_state)
                                {
                                    wasm_rs_dbg::dbg!();
                                    // use automerge::sync::SyncDoc;
                                    use futures_util::SinkExt;
                                    let message = tokio_tungstenite_wasm::Message::Binary(
                                        message.encode().to_vec(),
                                    );
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
                        ctx.request_repaint();
                    }
                })
            },
            &single.rt,
        )
        .unwrap()
    } else {
        single.ws = Some(exp::WsDoc::new(&single.rt, 42, ui.ctx().clone()));
    }
    const TIMER: u64 = 1;
    let is_portrait = ui.available_rect_before_wrap().aspect_ratio() < 1.0;
    if is_portrait {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::warn_if_debug_build(ui);
            ui.horizontal_wrapped(|ui| {
                for ex in EXAMPLES {
                    if ui.button(ex.name).clicked() {
                        // exp_ws::do_stuff(&single);
                        // exp::sync_stuff(&single);
                        single.commit = (&ex.commit).into();
                        single.len = ex.commits;
                        *code_editors.lock().unwrap() = (&ex.scripts).into();
                    }
                }
            });

            // if egui_addon::code_editor::generic_text_edit::TextEdit::<exp::Quot>::multiline(
            //     &mut single.quote.lock().unwrap(),
            // )
            // .show(ui)
            // .response
            // .changed()

            // if let Some(response) = {
            //     let mut qqq = code_editors.lock().unwrap();
            //     let resp = qqq.ui(ui);
            //     drop(qqq);
            //     resp
            // } {
            //     if response.changed() {
            //         // wasm_rs_dbg::dbg!(code_editors.auto.lock().unwrap().code.text.as_str());
            //         if let Some(ws) = &mut single.ws {
            //             let timer = if ws.timer != 0.0 {
            //                 let dt = ui.input(|mem| mem.unstable_dt);
            //                 ws.timer + dt
            //             } else {
            //                 0.01
            //             };
            //             if timer < std::time::Duration::from_secs(TIMER).as_secs_f32() {
            //                 ws.timer = timer;
            //                 ui.ctx()
            //                     .request_repaint_after(std::time::Duration::from_secs_f32(
            //                         TIMER as f32,
            //                     ));
            //             } else {
            //                 ws.timer = 0.0;
            //                 let quote: &mut CodeEditors<crate::app::code_editor_automerge::CodeEditor<
            //                     exp::Quot,
            //                 >> = &mut code_editors.lock().unwrap();
            //                 ws.changed(&single.rt, quote);
            //             }
            //         }
            //     } else if let Some(ws) = &mut single.ws {
            //         if ws.timer != 0.0 {
            //             let dt = ui.input(|mem| mem.unstable_dt);
            //             let timer = ws.timer + dt;
            //             if timer < std::time::Duration::from_secs(TIMER).as_secs_f32() {
            //                 ws.timer = timer;
            //                 ui.ctx()
            //                     .request_repaint_after(std::time::Duration::from_secs_f32(
            //                         TIMER as f32,
            //                     ));
            //             } else {
            //                 ws.timer = 0.0;
            //                 let quote: &mut CodeEditors<crate::app::code_editor_automerge::CodeEditor<
            //                     exp::Quot,
            //                 >> = &mut code_editors.lock().unwrap();
            //                 ws.changed(&single.rt, quote);
            //             }
            //         }
            //     }
            // }
            // ui.label(single.quote.lock().unwrap().text.as_str());
            let resps = {
                let mut ce = code_editors.lock().unwrap();
                [ce.init.ui(ui), ce.filter.ui(ui), ce.accumulate.ui(ui)]
            };
            if resps
                .iter()
                .any(|x| x.as_ref().map_or(false, |x| x.changed()))
            {
                // wasm_rs_dbg::dbg!(code_editors.lock().unwrap().code.text.as_str());
                if let Some(ws) = &mut single.ws {
                    let timer = if ws.timer != 0.0 {
                        let dt = ui.input(|mem| mem.unstable_dt);
                        ws.timer + dt
                    } else {
                        0.01
                    };
                    if timer < std::time::Duration::from_secs(TIMER).as_secs_f32() {
                        ws.timer = timer;
                        ui.ctx()
                            .request_repaint_after(std::time::Duration::from_secs_f32(
                                TIMER as f32,
                            ));
                    } else {
                        ws.timer = 0.0;
                        let quote: &mut CodeEditors<
                            crate::app::code_editor_automerge::CodeEditor<exp::Quot>,
                        > = &mut code_editors.lock().unwrap();
                        ws.changed(&single.rt, quote);
                    }
                }
            } else if let Some(ws) = &mut single.ws {
                if ws.timer != 0.0 {
                    let dt = ui.input(|mem| mem.unstable_dt);
                    let timer = ws.timer + dt;
                    if timer < std::time::Duration::from_secs(TIMER).as_secs_f32() {
                        ws.timer = timer;
                        ui.ctx()
                            .request_repaint_after(std::time::Duration::from_secs_f32(
                                TIMER as f32,
                            ));
                    } else {
                        ws.timer = 0.0;
                        let quote: &mut CodeEditors<
                            crate::app::code_editor_automerge::CodeEditor<exp::Quot>,
                        > = &mut code_editors.lock().unwrap();
                        ws.changed(&single.rt, quote);
                    }
                }
            }
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Compute")).clicked() {
                    *trigger_compute |= true;
                };
                show_short_result(&*compute_single_result, ui);
            });
            show_long_result(&*compute_single_result, ui);
        });
    } else {
        InteractiveSplitter::vertical()
            .ratio(0.7)
            .show(ui, |ui1, ui2| {
                ui1.push_id(ui1.id().with("input"), |ui| {
                    egui::warn_if_debug_build(ui);
                    ui.horizontal_wrapped(|ui| {
                        for ex in EXAMPLES {
                            if ui.button(ex.name).clicked() {
                                single.commit = (&ex.commit).into();
                                single.len = ex.commits;
                                *code_editors.lock().unwrap() = (&ex.scripts).into();
                            }
                        }
                    });
                    // let code_editor = &mut code_editors.lock().unwrap();
                    // if let Some(response) = code_editor.ui(ui) {
                    //     if response.changed() {
                    //         // wasm_rs_dbg::dbg!(code_editor.code.text.as_str());
                    //         if let Some(ws) = &mut single.ws {
                    //             let timer = if ws.timer != 0.0 {
                    //                 let dt = ui.input(|mem| mem.unstable_dt);
                    //                 ws.timer + dt
                    //             } else {
                    //                 0.01
                    //             };
                    //             if timer < std::time::Duration::from_secs(TIMER).as_secs_f32() {
                    //                 ws.timer = timer;
                    //                 ui.ctx().request_repaint_after(
                    //                     std::time::Duration::from_secs_f32(TIMER as f32),
                    //                 );
                    //             } else {
                    //                 ws.timer = 0.0;
                    //                 let quote: &mut crate::app::code_editor_automerge::CodeEditor<
                    //                     exp::Quot,
                    //                 > = code_editor;
                    //                 ws.changed(&single.rt, quote);
                    //             }
                    //         }
                    //     } else if let Some(ws) = &mut single.ws {
                    //         if ws.timer != 0.0 {
                    //             let dt = ui.input(|mem| mem.unstable_dt);
                    //             let timer = ws.timer + dt;
                    //             if timer < std::time::Duration::from_secs(TIMER).as_secs_f32() {
                    //                 ws.timer = timer;
                    //                 ui.ctx().request_repaint_after(
                    //                     std::time::Duration::from_secs_f32(TIMER as f32),
                    //                 );
                    //             } else {
                    //                 ws.timer = 0.0;
                    //                 let quote: &mut crate::app::code_editor_automerge::CodeEditor<
                    //                     exp::Quot,
                    //                 > = code_editor;
                    //                 ws.changed(&single.rt, quote);
                    //             }
                    //         }
                    //     }
                    // }
                    // code_editors.init.ui(ui);
                    // code_editors.filter.ui(ui);
                    // code_editors.accumulate.ui(ui);
                    let resps = {
                        let mut ce = code_editors.lock().unwrap();
                        [ce.init.ui(ui), ce.filter.ui(ui), ce.accumulate.ui(ui)]
                    };
                    if resps
                        .iter()
                        .any(|x| x.as_ref().map_or(false, |x| x.changed()))
                    {
                        // wasm_rs_dbg::dbg!(code_editors.lock().unwrap().code.text.as_str());
                        if let Some(ws) = &mut single.ws {
                            let timer = if ws.timer != 0.0 {
                                let dt = ui.input(|mem| mem.unstable_dt);
                                ws.timer + dt
                            } else {
                                0.01
                            };
                            if timer < std::time::Duration::from_secs(TIMER).as_secs_f32() {
                                ws.timer = timer;
                                ui.ctx()
                                    .request_repaint_after(std::time::Duration::from_secs_f32(
                                        TIMER as f32,
                                    ));
                            } else {
                                ws.timer = 0.0;
                                let quote: &mut CodeEditors<
                                    crate::app::code_editor_automerge::CodeEditor<exp::Quot>,
                                > = &mut code_editors.lock().unwrap();
                                ws.changed(&single.rt, quote);
                            }
                        }
                    } else if let Some(ws) = &mut single.ws {
                        if ws.timer != 0.0 {
                            let dt = ui.input(|mem| mem.unstable_dt);
                            let timer = ws.timer + dt;
                            if timer < std::time::Duration::from_secs(TIMER).as_secs_f32() {
                                ws.timer = timer;
                                ui.ctx()
                                    .request_repaint_after(std::time::Duration::from_secs_f32(
                                        TIMER as f32,
                                    ));
                            } else {
                                ws.timer = 0.0;
                                let quote: &mut CodeEditors<
                                    crate::app::code_editor_automerge::CodeEditor<exp::Quot>,
                                > = &mut code_editors.lock().unwrap();
                                ws.changed(&single.rt, quote);
                            }
                        }
                    }
                });
                let ui = ui2;
                // ui.painter().debug_rect(ui.max_rect(), egui::Color32::RED, "text");
                // ui.painter().debug_rect(ui.clip_rect(), egui::Color32::GREEN, "text");
                // ui.painter().debug_rect(ui.available_rect_before_wrap(), egui::Color32::BLUE, "text");
                // ui.set_clip_rect(ui.available_rect_before_wrap());
                // ui.set_max_size(ui.available_size_before_wrap());
                ui.horizontal(|ui| {
                    if ui.add(egui::Button::new("Compute")).clicked() {
                        *trigger_compute |= true;
                    };
                    show_short_result(&*compute_single_result, ui);
                });
                show_long_result(&*compute_single_result, ui);
            });
    }
}

impl Resource<Result<ComputeResults, ScriptingError>> {
    pub(super) fn from_response(
        ctx: &egui::Context,
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
    let header = content
        .results
        .iter()
        .find(|x| x.is_ok())
        .as_ref()
        .unwrap()
        .as_ref()
        .unwrap();
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

mod exp_crdt {

    #[test]
    fn f() {
        use automerge::ActorId;
        use autosurgeon::{hydrate, reconcile, Doc, Hydrate, Reconcile, Text};
        #[derive(Debug, Clone, Reconcile, Hydrate, PartialEq)]
        struct Contact {
            name: String,
            address: Address,
        }
        #[derive(Debug, Clone, Reconcile, Hydrate, PartialEq)]
        struct Address {
            line_one: String,
            line_two: Option<String>,
            city: String,
            postcode: String,
        }
        let mut contact = Contact {
            name: "Sherlock Holmes".to_string(),
            address: Address {
                line_one: "221B Baker St".to_string(),
                line_two: None,
                city: "London".to_string(),
                postcode: "42".to_string(),
            },
        };

        let mut doc = automerge::AutoCommit::new();
        reconcile(&mut doc, &contact).unwrap();

        let contact2: Contact = hydrate(&doc).unwrap();
        assert_eq!(contact, contact2);

        // Fork and make changes
        let mut doc2 = doc.fork().with_actor(automerge::ActorId::random());
        let mut contact2: Contact = hydrate(&doc2).unwrap();
        contact2.name = "Dangermouse".to_string();
        reconcile(&mut doc2, &contact2).unwrap();

        // Concurrently on doc1
        contact.address.line_one = "221C Baker St".to_string();
        reconcile(&mut doc, &contact).unwrap();

        // Now merge the documents
        doc.merge(&mut doc2).unwrap();

        let merged: Contact = hydrate(&doc).unwrap();
        assert_eq!(
            merged,
            Contact {
                name: "Dangermouse".to_string(), // This was updated in the first doc
                address: Address {
                    line_one: "221C Baker St".to_string(), // This was concurrently updated in doc2
                    line_two: None,
                    city: "London".to_string(),
                    postcode: "42".to_string(),
                }
            }
        )
    }

    #[test]
    fn g() {
        use automerge::ActorId;
        use autosurgeon::{hydrate, reconcile, Hydrate, Reconcile, Text};
        #[derive(Default, Debug, Reconcile, Hydrate)]
        pub(crate) struct Quote {
            pub(crate) text: Text,
        }
        let mut doc = automerge::AutoCommit::new();
        let quote = Quote {
            text: "glimmers".into(),
        };
        reconcile(&mut doc, &quote).unwrap();

        // Fork and make changes to the text
        let mut doc2 = doc.fork().with_actor(ActorId::random());
        let heads = doc2.get_heads();
        let mut quote2: Quote = hydrate(&doc2).unwrap();
        quote2.text.splice(0, 0, "All that ");
        let end_index = quote2.text.as_str().char_indices().last().unwrap().0;
        quote2.text.splice(end_index + 1, 0, " is not gold");
        reconcile(&mut doc2, &quote2).unwrap();
        dbg!(doc2.get_changes(&heads).unwrap());
        automerge::Change::try_from(
            doc2.get_changes(&heads)
                .unwrap()
                .get(0)
                .unwrap()
                .raw_bytes(),
        )
        .unwrap();

        // Concurrently modify the text in the original doc
        let mut quote: Quote = hydrate(&doc).unwrap();
        let m_index = quote.text.as_str().char_indices().nth(3).unwrap().0;
        quote.text.splice(m_index, 2, "tt");
        reconcile(&mut doc, quote).unwrap();

        // Merge the changes
        doc.merge(&mut doc2).unwrap();

        let quote: Quote = hydrate(&doc).unwrap();
        assert_eq!(quote.text.as_str(), "All that glitters is not gold");
    }

    #[test]
    fn h() {
        use automerge::transaction::CommitOptions;
        use automerge::transaction::Transactable;
        use automerge::AutomergeError;
        use automerge::ObjType;
        use automerge::{Automerge, ReadDoc, ROOT};
        let mut doc1 = Automerge::new();
        let (cards, card1) = doc1
            .transact_with::<_, _, automerge::AutomergeError, _>(
                |_| CommitOptions::default().with_message("Add card".to_owned()),
                |tx| {
                    let cards = tx.put_object(ROOT, "cards", ObjType::List).unwrap();
                    let card1 = tx.insert_object(&cards, 0, ObjType::Map)?;
                    tx.put(&card1, "title", "Rewrite everything in Clojure")?;
                    tx.put(&card1, "done", false)?;
                    let card2 = tx.insert_object(&cards, 0, ObjType::Map)?;
                    tx.put(&card2, "title", "Rewrite everything in Haskell")?;
                    tx.put(&card2, "done", false)?;
                    Ok((cards, card1))
                },
            )
            .unwrap()
            .result;

        let mut doc2 = Automerge::new();
        doc2.merge(&mut doc1).unwrap();

        let binary = doc1.save();
        let mut doc2 = Automerge::load(&binary).unwrap();

        doc1.transact_with::<_, _, AutomergeError, _>(
            |_| CommitOptions::default().with_message("Mark card as done".to_owned()),
            |tx| {
                tx.put(&card1, "done", true)?;
                Ok(())
            },
        )
        .unwrap();

        doc2.transact_with::<_, _, AutomergeError, _>(
            |_| CommitOptions::default().with_message("Delete card".to_owned()),
            |tx| {
                tx.delete(&cards, 0)?;
                Ok(())
            },
        )
        .unwrap();

        doc1.merge(&mut doc2).unwrap();

        for change in doc1.get_changes(&[]).unwrap() {
            let length = doc1.length_at(&cards, &[change.hash()]);
            println!("{} {}", change.message().unwrap(), length);
        }
    }
}

pub(crate) mod exp {

    use std::sync::{Arc, Mutex, RwLock};

    #[cfg(target_arch = "wasm32")]
    use async_executors::JoinHandle;
    use automerge::ActorId;
    use autosurgeon::{hydrate, reconcile, Hydrate, Reconcile, Text};
    use egui_addon::code_editor::generic_text_buffer::TextBuffer;
    use futures_util::{Future, SinkExt, StreamExt};
    #[cfg(not(target_arch = "wasm32"))]
    use tokio::task::JoinHandle;

    #[derive(Default, Debug, Reconcile, Hydrate)]
    pub(crate) struct Quot {
        pub(crate) text: Text,
    }

    impl<'de> serde::Deserialize<'de> for Quot {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: serde::Deserializer<'de>,
        {
            use de::Unexpected;
            use serde::de;
            use std::fmt;
            struct V;
            impl<'de> serde::de::Visitor<'de> for V {
                type Value = String;

                fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                    write!(formatter, "a string containing at least {} bytes", 0)
                }

                fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
                where
                    E: de::Error,
                {
                    if s.len() >= 0 {
                        Ok(s.to_owned())
                    } else {
                        Err(de::Error::invalid_value(Unexpected::Str(s), &self))
                    }
                }
            }
            deserializer.deserialize_string(V).map(|x| Quot {
                text: Text::with_value(x),
            })
        }
    }

    impl serde::Serialize for Quot {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            serializer.collect_str(self.text.as_str())
        }
    }

    // TODO take inspiration from Text tom impl From
    impl From<String> for Quot {
        fn from(value: String) -> Self {
            Quot { text: value.into() }
        }
    }
    impl From<&str> for Quot {
        fn from(value: &str) -> Self {
            Quot { text: value.into() }
        }
    }

    impl egui_addon::code_editor::generic_text_buffer::AsText for Quot {
        fn text(&self) -> &str {
            self.text.as_str()
        }
    }

    impl TextBuffer for Quot {
        type Ref = Quot;

        fn is_mutable(&self) -> bool {
            true
        }

        fn as_reference(&self) -> &Self::Ref {
            self
        }

        fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
            let l = text.len();
            self.text.splice(char_index, 0, text);
            l
        }

        fn delete_char_range(&mut self, char_range: std::ops::Range<usize>) {
            self.text
                .splice(char_range.start, char_range.end - char_range.start, "");
        }
    }

    pub(crate) type Quote = Arc<Mutex<Quot>>;

    pub(super) struct WsCont(tokio_tungstenite_wasm::WebSocketStream);
    unsafe impl Send for WsCont {}

    pub(super) struct WsDoc {
        ws: WsState,
        pub doc: Arc<RwLock<(automerge::AutoCommit, automerge::sync::State)>>,
        pub timer: f32,
    }

    #[derive(Default)]
    enum WsState {
        Init(poll_promise::Promise<tokio_tungstenite_wasm::Result<WsCont>>),
        Error(tokio_tungstenite_wasm::Error),
        Setup(
            // futures_util::stream::SplitSink<
            //     tokio_tungstenite_wasm::WebSocketStream,
            //     tokio_tungstenite_wasm::Message,
            // >,
            futures::channel::mpsc::Sender<tokio_tungstenite_wasm::Message>,
            H,
        ),
        #[default]
        Empty,
    }

    impl WsDoc {
        pub(super) fn new(rt: &Rt, who: usize, ctx: egui::Context) -> Self {
            let (s, p) = poll_promise::Promise::new();
            rt.spawn(async move {
                s.send(WsDoc::make_ws_async(who).await);
                ctx.request_repaint();
            });
            WsDoc {
                ws: WsState::Init(p),
                doc: Arc::new(RwLock::new((
                    automerge::AutoCommit::new(),
                    automerge::sync::State::new(),
                ))),
                timer: 0.0,
            }
        }
        async fn make_ws_async(who: usize) -> tokio_tungstenite_wasm::Result<WsCont> {
            let url = format!("ws://{}/ws", &API_URL[7..]);
            wasm_rs_dbg::dbg!(&url);
            match tokio_tungstenite_wasm::connect(url).await {
                Ok(stream) => {
                    wasm_rs_dbg::dbg!("Handshake for client {} has been completed", who);
                    Ok(WsCont(stream))
                }
                Err(e) => {
                    wasm_rs_dbg::dbg!("WebSocket handshake for client {who} failed with {e}!");
                    Err(e)
                }
            }
        }
        pub(super) fn setup_atempt<F>(&mut self, receiver_f: F, rt: &Rt) -> Result<(), String>
        where
            F: FnOnce(
                futures::channel::mpsc::Sender<tokio_tungstenite_wasm::Message>,
                futures_util::stream::SplitStream<tokio_tungstenite_wasm::WebSocketStream>,
            ) -> H,
        {
            // NOTE could replace Empty variant by the following default value
            // tokio_tungstenite_wasm::Error::from(http::status::StatusCode::from_u16(42).unwrap_err());
            match std::mem::take(&mut self.ws) {
                WsState::Init(prom) => {
                    match prom.try_take() {
                        Ok(Ok(ws)) => {
                            let (mut sender, receiver) = ws.0.split();
                            let (s, mut r) = futures::channel::mpsc::channel(50);
                            rt.spawn(async move {
                                wasm_rs_dbg::dbg!();
                                while let Some(x) = r.next().await {
                                    wasm_rs_dbg::dbg!();
                                    sender.send(x).await.expect("Can not send!");
                                }
                            });
                            self.ws = WsState::Setup(s.clone(), receiver_f(s, receiver))
                        }
                        Ok(Err(err)) => {
                            let error = err.to_string();
                            self.ws = WsState::Error(err);
                            return Err(error);
                        }
                        Err(prom) => self.ws = WsState::Init(prom),
                    };
                    Ok(())
                }
                WsState::Error(err) => {
                    let error = err.to_string();
                    self.ws = WsState::Error(err);
                    Err(error)
                }
                WsState::Setup(s, r) => {
                    self.ws = WsState::Setup(s, r);
                    Ok(())
                }
                WsState::Empty => panic!("unrecoverable state"),
            }
        }

        pub(crate) fn changed(&mut self, rt: &Rt, quote: &mut impl Reconcile) {
            wasm_rs_dbg::dbg!();
            let (doc, sync_state): &mut (_, _) = &mut self.doc.write().unwrap();
            // let heads = doc.get_heads();
            if let Err(e) = reconcile(doc, &*quote) {
                wasm_rs_dbg::dbg!(e);
            };
            match &mut self.ws {
                WsState::Init(_) => (),
                WsState::Error(_) => (),
                WsState::Setup(sender, _) => {
                    use automerge::sync::SyncDoc;
                    if let Some(x) = doc.sync().generate_sync_message(sync_state) {
                        let x = tokio_tungstenite_wasm::Message::Binary(x.encode().to_vec());
                        let mut sender = sender.clone();
                        rt.spawn(async move {
                            sender.send(x).await.unwrap();
                        });
                    } else {
                        wasm_rs_dbg::dbg!();
                    }
                    // let changes = doc
                    //     .get_changes(&heads)
                    //     .unwrap()
                    //     .into_iter()
                    //     .map(|x| tokio_tungstenite_wasm::Message::Binary(x.raw_bytes().to_vec()))
                    //     .collect::<Vec<_>>();
                    // doc.commit();
                    // wasm_rs_dbg::dbg!(&changes);
                    // let mut sender = sender.clone();
                    // rt.spawn(async move {
                    //     for x in changes {
                    //         sender.send(x).await.unwrap();
                    //     }
                    // });
                }
                WsState::Empty => panic!(),
            };
        }

        pub(crate) fn sender(
            &mut self,
        ) -> Option<futures::channel::mpsc::Sender<tokio_tungstenite_wasm::Message>> {
            match &mut self.ws {
                WsState::Init(_) => None,
                WsState::Error(_) => None,
                WsState::Setup(sender, _) => Some(sender.clone()),
                WsState::Empty => panic!(),
            }
        }
    }

    pub(super) struct WsIntern(
        Arc<
            Mutex<
                futures_util::stream::SplitSink<
                    tokio_tungstenite_wasm::WebSocketStream,
                    tokio_tungstenite_wasm::Message,
                >,
            >,
        >,
        H, // futures_util::stream::SplitStream<tokio_tungstenite_wasm::WebSocketStream>,
        Quote,
    );
    unsafe impl Send for WsIntern {}
    impl WsIntern {
        fn do_stuff(&self, rt: &Rt) {
            // let mutex = self.0.clone();
            // let mut sender = mutex.try_lock().unwrap();
            // let sent = sender
            //         .send(tokio_tungstenite_wasm::Message::Text(
            //             "Hello, Server!".into(),
            //         ));
            // rt.spawn(async move {
            //         sent.await
            //         .expect("Can not send!");
            // });
        }
    }

    pub(super) type Ws = poll_promise::Promise<tokio_tungstenite_wasm::Result<WsIntern>>;

    #[derive(Clone)]
    pub(super) struct Rt(
        #[cfg(not(target_arch = "wasm32"))] pub(super) Arc<tokio::runtime::Runtime>,
    );

    impl Default for Rt {
        fn default() -> Self {
            Self(
                #[cfg(not(target_arch = "wasm32"))]
                Arc::new(
                    tokio::runtime::Builder::new_multi_thread()
                        .enable_all()
                        .build()
                        .unwrap(),
                ),
            )
        }
    }

    pub(super) struct H(#[cfg(not(target_arch = "wasm32"))] JoinHandle<()>);

    impl Rt {
        #[cfg(not(target_arch = "wasm32"))]
        pub(super) fn spawn<F>(&self, future: F) -> H
        where
            F: Future<Output = ()> + Send + 'static,
        {
            H(self.0.spawn(future))
        }

        #[cfg(target_arch = "wasm32")]
        pub(super) fn spawn<F>(&self, future: F) -> H
        where
            F: Future<Output = ()> + 'static,
        {
            wasm_bindgen_futures::spawn_local(future);
            H()
        }
    }
    pub(super) fn make_ws(rt: &Rt, who: usize, quote: &Quote, ctx: egui::Context) -> Ws {
        let (s, p) = poll_promise::Promise::new();
        let rt0 = rt.clone();
        let quote = quote.clone();
        rt.spawn(async move {
            s.send(make_ws_async(&rt0, who, quote, ctx.clone()).await);
            ctx.request_repaint();
        });
        p
    }
    pub(super) async fn make_ws_async(
        rt: &Rt,
        who: usize,
        quote: Quote,
        ctx: egui::Context,
    ) -> tokio_tungstenite_wasm::Result<WsIntern> {
        let url = format!("ws://{}/ws", &API_URL[7..]);
        wasm_rs_dbg::dbg!(&url);
        match tokio_tungstenite_wasm::connect(url).await {
            Ok(stream) => {
                //(stream, response)) => {
                wasm_rs_dbg::dbg!("Handshake for client {} has been completed", who);
                // This will be the HTTP response, same as with server this is the last moment we
                // can still access HTTP stuff.
                // println!("Server response was {:?}", response);
                // let aaa = stream.next().await;
                // dbg!(aaa);
                let (sink, mut receiver) = stream.split();
                // let receiver = Arc::new(Mutex::new(receiver));

                let recv_task = rt.spawn(async move {
                    while let Some(Ok(msg)) = receiver.next().await {
                        let text = &mut quote.lock().unwrap().text;
                        text.splice(text.as_str().len(), 0, msg.to_string());
                        ctx.request_repaint();
                        // print message and break if instructed to do so
                        if process_message(msg, who).is_break() {
                            break;
                        }
                    }
                });
                let sink = Arc::new(Mutex::new(sink));
                Ok(WsIntern(sink, recv_task, Default::default()))
            }
            Err(e) => {
                wasm_rs_dbg::dbg!("WebSocket handshake for client {who} failed with {e}!");
                Err(e)
            }
        }
    }

    /// Function to handle messages we get (with a slight twist that Frame variant is visible
    /// since we are working with the underlying tungstenite library directly without axum here).
    fn process_message(
        msg: tokio_tungstenite_wasm::Message,
        who: usize,
    ) -> std::ops::ControlFlow<(), ()> {
        match msg {
            tokio_tungstenite_wasm::Message::Text(t) => {
                wasm_rs_dbg::dbg!(format!(">>> {} got str: {:?}", who, t));
            }
            tokio_tungstenite_wasm::Message::Binary(d) => {
                wasm_rs_dbg::dbg!(format!(">>> {} got {} bytes: {:?}", who, d.len(), d));
            }
            tokio_tungstenite_wasm::Message::Close(c) => {
                if let Some(cf) = c {
                    wasm_rs_dbg::dbg!(format!(
                        ">>> {} got close with code {} and reason `{}`",
                        who, cf.code, cf.reason
                    ));
                } else {
                    wasm_rs_dbg::dbg!(format!(
                        ">>> {} somehow got close message without CloseFrame",
                        who
                    ));
                }
                return std::ops::ControlFlow::Break(());
            } // Message::Pong(v) => {
              //     println!(">>> {} got pong with {:?}", who, v);
              // }
              // // Just as with axum server, the underlying tungstenite websocket library
              // // will handle Ping for you automagically by replying with Pong and copying the
              // // v according to spec. But if you need the contents of the pings you can see them here.
              // Message::Ping(v) => {
              //     println!(">>> {} got ping with {:?}", who, v);
              // }

              // Message::Frame(_) => {
              //     unreachable!("This is never supposed to happen")
              // }
        }
        std::ops::ControlFlow::Continue(())
    }

    use crate::app::API_URL;

    // #[cfg(not(target_arch = "wasm32"))]
    // pub(super) fn spawn<F>(rt: &Rt, future: F) -> JoinHandle<F::Output>
    // where
    //     F: Future + Send + 'static,
    //     F::Output: Send + 'static,
    // {
    //     rt.0.spawn(future)
    // }

    // #[cfg(target_arch = "wasm32")]
    // pub(super) fn spawn<F>(rt: &Rt, future: F) -> JoinHandle<F::Output>
    // where
    //     F: Future + Send + 'static,
    //     F::Output: Send + 'static,
    // {
    //     wasm_bindgen_futures::spawn_local(future);
    // }

    // #[cfg(not(target_arch = "wasm32"))]
    // pub(super) fn sync_stuff(rt: &Rt) {
    //     spawn(rt, async move {
    //         // spawn_client(42).await;
    //     });
    // }

    // #[cfg(target_arch = "wasm32")]
    // pub(super) fn sync_stuff(rt: &Rt) {
    //     wasm_rs_dbg::dbg!();
    //     spawn(rt, async move {
    //         // spawn_client(rt, 42).await;
    //     });
    //     // poll_promise::Promise::spawn_async(async move {
    //     //     spawn_client(42).await;
    //     // }).block_until_ready();
    // }
}

mod exp_ws {
    // use tokio_tungstenite::{connect_async, tungstenite::Message};

    use std::{borrow::Cow, ops::ControlFlow};

    use async_executors::{LocalSpawnHandle, LocalSpawnHandleExt, SpawnHandle, SpawnHandleExt};
    use egui_addon::async_exec::spawn_macrotask;
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite_wasm::Message;

    use crate::app::API_URL;

    //creates a client. quietly exits on failure.
    pub(crate) async fn spawn_client(who: usize) {
        //, exec: impl LocalSpawnHandle<()>
        let (mut sender, mut receiver) = match make_ws(who).await {
            Some(value) => value,
            None => return,
        };

        //we can ping the server for start
        sender
            .send(Message::Text("Hello, Server!".into()))
            .await
            .expect("Can not send!");

        // //spawn an async sender to push some more messages into the server
        // let mut send_task = exec.spawn_handle_local(async move {
        //     for i in 1..30 {
        //         // In any websocket error, break loop.
        //         if sender
        //             .send(Message::Text(format!("Message number {}...", i)))
        //             .await
        //             .is_err()
        //         {
        //             //just as with server, if send fails there is nothing we can do but exit.
        //             return;
        //         }

        //         // tokio::time::sleep(std::time::Duration::from_millis(300)).await;
        //     }

        //     // When we are done we may want our client to close connection cleanly.
        //     println!("Sending close to {}...", who);
        //     // if let Err(e) = sender
        //     //     .send(Message::Close(Some(CloseFrame {
        //     //         code: tokio_tungstenite_wasm::coding::CloseCode::Normal,
        //     //         reason: Cow::from("Goodbye"),
        //     //     })))
        //     //     .await
        //     // {
        //     //     println!("Could not send Close due to {:?}, probably it is ok?", e);
        //     // };
        // }).unwrap();

        while let Some(Ok(msg)) = receiver.next().await {
            // print message and break if instructed to do so
            if process_message(msg, who).is_break() {
                break;
            }
        }

        // //receiver just prints whatever it gets
        // let mut recv_task = exec.spawn_handle_local(async move {
        //     while let Some(Ok(msg)) = receiver.next().await {
        //         // print message and break if instructed to do so
        //         if process_message(msg, who).is_break() {
        //             break;
        //         }
        //     }
        // }).unwrap();

        // // //wait for either task to finish and kill the other task
        // // tokio::select! {
        // //     _ = (&mut send_task) => {
        // //         recv_task.abort();
        // //     },
        // //     _ = (&mut recv_task) => {
        // //         send_task.abort();
        // //     }
        // // }
        // exec.spawn_handle_local(async move {
        //     recv_task.await
        // }).unwrap();
    }

    async fn make_ws(
        who: usize,
    ) -> Option<(
        futures_util::stream::SplitSink<tokio_tungstenite_wasm::WebSocketStream, Message>,
        futures_util::stream::SplitStream<tokio_tungstenite_wasm::WebSocketStream>,
    )> {
        let url = format!("ws://{}/ws", &API_URL[7..]);
        wasm_rs_dbg::dbg!(&url);
        let ws_stream = match tokio_tungstenite_wasm::connect(url).await {
            Ok(mut stream) => {
                //(stream, response)) => {
                wasm_rs_dbg::dbg!("Handshake for client {} has been completed", who);
                // This will be the HTTP response, same as with server this is the last moment we
                // can still access HTTP stuff.
                // println!("Server response was {:?}", response);
                let aaa = stream.next().await;
                dbg!(aaa);
                stream
            }
            Err(e) => {
                wasm_rs_dbg::dbg!("WebSocket handshake for client {who} failed with {e}!");
                return None;
            }
        };
        let (mut sender, mut receiver) = ws_stream.split();
        Some((sender, receiver))
    }

    /// Function to handle messages we get (with a slight twist that Frame variant is visible
    /// since we are working with the underlying tungstenite library directly without axum here).
    fn process_message(msg: Message, who: usize) -> ControlFlow<(), ()> {
        match msg {
            Message::Text(t) => {
                wasm_rs_dbg::dbg!(format!(">>> {} got str: {:?}", who, t));
            }
            Message::Binary(d) => {
                wasm_rs_dbg::dbg!(format!(">>> {} got {} bytes: {:?}", who, d.len(), d));
            }
            Message::Close(c) => {
                if let Some(cf) = c {
                    wasm_rs_dbg::dbg!(format!(
                        ">>> {} got close with code {} and reason `{}`",
                        who, cf.code, cf.reason
                    ));
                } else {
                    wasm_rs_dbg::dbg!(format!(
                        ">>> {} somehow got close message without CloseFrame",
                        who
                    ));
                }
                return ControlFlow::Break(());
            } // Message::Pong(v) => {
              //     println!(">>> {} got pong with {:?}", who, v);
              // }
              // // Just as with axum server, the underlying tungstenite websocket library
              // // will handle Ping for you automagically by replying with Pong and copying the
              // // v according to spec. But if you need the contents of the pings you can see them here.
              // Message::Ping(v) => {
              //     println!(">>> {} got ping with {:?}", who, v);
              // }

              // Message::Frame(_) => {
              //     unreachable!("This is never supposed to happen")
              // }
        }
        ControlFlow::Continue(())
    }

    use crate::app::ComputeConfigSingle;

    #[cfg(not(target_arch = "wasm32"))]
    pub(super) fn do_stuff(single: &ComputeConfigSingle) {
        wasm_rs_dbg::dbg!();
        // poll_promise::Promise::spawn_async(async move {
        //     spawn_client(42).await;
        // }).block_until_ready();
        single.rt.0.spawn(async move {
            spawn_client(42).await;
        });
    }

    #[cfg(target_arch = "wasm32")]
    pub(super) fn do_stuff(single: &ComputeConfigSingle) {
        wasm_rs_dbg::dbg!();
        wasm_bindgen_futures::spawn_local(async move {
            spawn_client(42).await;
        });
        // poll_promise::Promise::spawn_async(async move {
        //     spawn_client(42).await;
        // }).block_until_ready();
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
