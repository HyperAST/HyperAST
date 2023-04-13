use egui::Id;
use poll_promise::Promise;

use crate::app::{utils::{file_save, self}, API_URL};

use super::{
    code_editor::{CodeEditor, EditorInfo},
    egui_utils::{radio_collapsing, show_wip},
    types::{CodeEditors, CommitId, Repo, Resource, SelectedConfig, Commit}, show_repo_menu,
};

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

impl Default for CodeEditors {
    fn default() -> Self {
        Self {
            init: CodeEditor {
                info: INFO_INIT.copied(),
                ..r##"#{ depth:0, files: 0, type_decl: 0 }"##.into()
            },
            filter: CodeEditor {
                info: INFO_FILTER.copied(),
                ..r##"if is_directory() {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        files: s.files,
        type_decl: s.type_decl,
    }])
} else if is_file() {
    children().map(|x| [x, #{
        depth: s.depth + 1,
        type_decl: s.type_decl,
    }])
} else {
    []
}"##
                .into()
            },
            accumulate: CodeEditor {
                info: INFO_ACCUMULATE.copied(),
                ..r##"if is_directory() {
    p.files += s.files;
    p.type_decl += s.type_decl;
} else if is_file() {
    p.files += 1;
    p.type_decl += s.type_decl;
} else if is_type_decl() {
    p.type_decl += 1; 
}"##
                .into()
            },
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(super) struct ComputeConfigSingle {
    commit: Commit,
    len: usize,
}

impl Default for ComputeConfigSingle {
    fn default() -> Self {
        Self {
            commit: Default::default(),
            // commit: "4acedc53a13a727be3640fe234f7e261d2609d58".into(),
            len: 1,
        }
    }
}

pub(super) type RemoteResult =
    Promise<ehttp::Result<Resource<Result<ComputeResult, ScriptingError>>>>;

pub(super) fn remote_compute_single(
    ctx: &egui::Context,
    single: &mut ComputeConfigSingle,
    code_editors: &mut CodeEditors,
) -> Promise<Result<Resource<Result<ComputeResult, ScriptingError>>, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "{}/script/github/{}/{}/{}",
        API_URL, &single.commit.repo.user, &single.commit.repo.name, &single.commit.id,
    );
    #[derive(serde::Serialize)]
    struct ScriptContent {
        init: String,
        filter: String,
        accumulate: String,
    }

    let script = ScriptContent {
        init: code_editors.init.code().to_string(),
        filter: code_editors.filter.code().to_string(),
        accumulate: code_editors.accumulate.code().to_string(),
    };

    let mut request = ehttp::Request::post(&url, serde_json::to_vec(&script).unwrap());
    request.headers.insert(
        "Content-Type".to_string(),
        "application/json; charset=utf-8".to_string(),
    );

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        let resource = response.and_then(|response| {
            Resource::<Result<ComputeResult, ScriptingError>>::from_response(&ctx, response)
        });
        sender.send(resource);
    });
    promise
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
}

impl Resource<Result<ComputeResult, ScriptingError>> {
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

        ui.add_enabled_ui(false, |ui| {
            ui.add(
                egui::Slider::new(&mut single.len, 0..=200)
                    .text("commits")
                    .clamp_to_range(false)
                    .integer()
                    .logarithmic(true),
            );
            show_wip(ui, Some("only process one commit"));
        });
    };

    radio_collapsing(ui, id, title, selected, wanted, add_body);
}

pub(super) fn show_short_result(promise: &Option<RemoteResult>, ui: &mut egui::Ui) {
    if let Some(promise) = &promise {
        if let Some(result) = promise.ready() {
            match result {
                Ok(resource) => {
                    // ui_resource(ui, resource);
                    dbg!(&resource.response);
                    if let Some(text) = &resource.content {
                        match text {
                            Ok(text) => {
                                if ui.add(egui::Button::new("Export")).clicked() {
                                    if let Ok(text) = serde_json::to_string_pretty(text) {
                                        utils::file_save(&text)
                                    }
                                };
                                ui.label(format!("compute time: {} seconds", text.compute_time));
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
                    if let Some(text) = &resource.content {
                        match text {
                            Ok(text) => {
                                egui::CollapsingHeader::new("Results (JSON)")
                                    .default_open(true)
                                    .show(ui, |ui| {
                                        let mut code: &str =
                                            &serde_json::to_string_pretty(&text.result).unwrap();
                                        let language = "json";
                                        let theme =
                                    egui_demo_lib::syntax_highlighting::CodeTheme::from_memory(
                                        ui.ctx(),
                                    );
                                        let mut layouter =
                                            |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                                                let layout_job =
                                                    egui_demo_lib::syntax_highlighting::highlight(
                                                        ui.ctx(),
                                                        &theme,
                                                        string,
                                                        language,
                                                    );
                                                // layout_job.wrap.max_width = wrap_width; // no wrapping
                                                ui.fonts(|f| f.layout_job(layout_job))
                                            };

                                        ui.add(
                                            egui::TextEdit::multiline(&mut code)
                                                .font(egui::TextStyle::Monospace) // for cursor height
                                                .code_editor()
                                                .desired_rows(1)
                                                .lock_focus(true)
                                                .layouter(&mut layouter),
                                        );
                                    });
                            }
                            Err(error) => {
                                let (h, c) = match error {
                                    ScriptingError::AtCompilation(err) => {
                                        ("Error at compilation:", err)
                                    }
                                    ScriptingError::AtEvaluation(err) => {
                                        ("Error at evaluation:", err)
                                    }
                                };
                                ui.label(
                                    egui::RichText::new(h)
                                        .heading()
                                        .color(ui.visuals().error_fg_color),
                                );
                                ui.colored_label(ui.visuals().error_fg_color, c);
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
