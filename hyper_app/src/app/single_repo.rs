use poll_promise::Promise;

use crate::app::{utils, API_URL};

use self::example_scripts::EXAMPLES;

use super::{
    code_editor::{CodeEditor, EditorInfo},
    egui_utils::radio_collapsing,
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

impl From<&example_scripts::Scripts> for CodeEditors {
    fn from(value: &example_scripts::Scripts) -> Self {
        Self {
            init: CodeEditor {
                info: INFO_INIT.copied(),
                ..value.init.into()
            },
            filter: CodeEditor {
                info: INFO_FILTER.copied(),
                ..value.filter.into()
            },
            accumulate: CodeEditor {
                info: INFO_ACCUMULATE.copied(),
                ..value.accumulate.into()
            },
        }
    }
}

impl Default for CodeEditors {
    fn default() -> Self {
        (&example_scripts::EXAMPLES[0].scripts).into()
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
            commit: From::from(&example_scripts::EXAMPLES[0].commit),
            // commit: "4acedc53a13a727be3640fe234f7e261d2609d58".into(),
            len: example_scripts::EXAMPLES[0].commits,
        }
    }
}

pub(super) type RemoteResult =
    Promise<ehttp::Result<Resource<Result<ComputeResults, ScriptingError>>>>;

pub(super) fn remote_compute_single(
    ctx: &egui::Context,
    single: &mut ComputeConfigSingle,
    code_editors: &mut CodeEditors,
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

    let script = ScriptContent {
        init: code_editors.init.code().to_string(),
        filter: code_editors.filter.code().to_string(),
        accumulate: code_editors.accumulate.code().to_string(),
        commits: single.len,
    };

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
    code_editors: &mut super::types::CodeEditors,
    trigger_compute: &mut bool,
    compute_single_result: &mut Option<
        poll_promise::Promise<
            Result<super::types::Resource<Result<ComputeResults, ScriptingError>>, String>,
        >,
    >,
) {
    let is_portrait = ui.available_rect_before_wrap().aspect_ratio() < 1.0;
    if is_portrait {
        egui::ScrollArea::vertical().show(ui, |ui| {
            egui::warn_if_debug_build(ui);
            code_editors.init.ui(ui);
            code_editors.filter.ui(ui);
            code_editors.accumulate.ui(ui);
            ui.horizontal(|ui| {
                if ui.add(egui::Button::new("Compute")).clicked() {
                    *trigger_compute |= true;
                };
                show_short_result(&*compute_single_result, ui);
            });
            show_long_result(&*compute_single_result, ui);
        });
    } else {
        super::interactive_split::Splitter::vertical()
            .ratio(0.7)
            .show(ui, |ui1, ui2| {
                ui1.push_id(ui1.id().with("input"), |ui| {
                    ui.horizontal_wrapped(|ui| {
                        for ex in EXAMPLES {
                            if ui.button(ex.name).clicked() {
                                single.commit = (&ex.commit).into();
                                single.len = ex.commits;
                                *code_editors = (&ex.scripts).into();
                            }
                        }
                    });
                    egui::warn_if_debug_build(ui);
                    code_editors.init.ui(ui);
                    code_editors.filter.ui(ui);
                    code_editors.accumulate.ui(ui);
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

    radio_collapsing(ui, id, title, selected, wanted, add_body);
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
                                ui.label(format!(
                                    "{:.3}",
                                    SecFmt(cont.inner.compute_time)
                                ));
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
