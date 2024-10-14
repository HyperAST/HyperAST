use poll_promise::Promise;

use crate::app::types;
use crate::app::utils;
use crate::app::utils::SecFmt;

use super::types::CommitId;
use super::types::Resource;
use super::ProjectId;

pub(crate) trait ComputeError {
    fn head(&self) -> &str;
    fn content(&self) -> &str;
}

pub(super) fn show_short_result(
    promise: &Option<RemoteResult<impl ComputeError + Send + Sync>>,
    ui: &mut egui::Ui,
) {
    let Some(promise) = &promise else {
        ui.label("click on Compute");
        return;
    };
    let Some(result) = promise.ready() else {
        ui.spinner();
        return;
    };
    let Ok(resource) = result else {
        ui.label(format!("compute time: N/A"));
        return;
    };
    let Some(Ok(content)) = &resource.content else {
        ui.label(format!("compute time: N/A"));
        return;
    };
    if ui.add(egui::Button::new("Export")).clicked() {
        if let Ok(text) = serde_json::to_string_pretty(content) {
            utils::file_save("query_results", ".json", &text);
        }
    };
    show_short_result_aux(content, ui);
}

pub(crate) fn show_short_result_aux(content: &ComputeResults, ui: &mut egui::Ui) {
    if content.results.len() == 1 {
        if let Ok(res) = &content.results[0] {
            ui.label(format!(
                "compute time: {:.3}",
                SecFmt(content.prepare_time + res.inner.compute_time)
            ));
        }
    } else {
        let compute_time = content
            .results
            .iter()
            .filter_map(|x| x.as_ref().ok())
            .map(|x| x.inner.compute_time)
            .sum::<f64>();
        ui.label(format!(
            "compute time: {:.3} + {:.3}",
            SecFmt(content.prepare_time),
            SecFmt(compute_time)
        ));
    }
}

pub(super) type Remote<R> = Promise<ehttp::Result<Resource<R>>>;
pub(super) type RemoteResult<E> = Remote<Result<ComputeResults, E>>;

pub(crate) fn show_long_result(
    promise: &Option<RemoteResult<impl ComputeError + Send + Sync>>,
    ui: &mut egui::Ui,
) {
    let Some(promise) = &promise else {
        ui.label("click on Compute");
        return;
    };
    let Some(result) = promise.ready() else {
        ui.spinner();
        return;
    };
    match result {
        Ok(resource) => match &resource.content {
            Some(Ok(content)) => {
                show_long_result_success(ui, content);
            }
            Some(Err(error)) => {
                show_long_result_compute_failure(ui, error);
            }
            _ => (),
        },
        Err(error) => {
            wasm_rs_dbg::dbg!();
            // This should only happen if the fetch API isn't available or something similar.
            ui.colored_label(
                ui.visuals().error_fg_color,
                if error.is_empty() { "Error" } else { error },
            );
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
pub struct ComputeResults {
    pub prepare_time: f64,
    pub results: Vec<Result<ComputeResultIdentified, String>>,
}

impl std::hash::Hash for ComputeResults {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.results.hash(state);
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Hash)]
pub struct ComputeResultIdentified {
    pub commit: types::CommitId,
    #[serde(flatten)]
    pub inner: ComputeResult,
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct ComputeResult {
    pub compute_time: f64,
    pub result: serde_json::Value,
}

impl std::hash::Hash for ComputeResult {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.result.hash(state);
    }
}

fn show_long_result_compute_failure<'a>(ui: &mut egui::Ui, error: &impl ComputeError) {
    ui.label(
        egui::RichText::new(error.head())
            .heading()
            .color(ui.visuals().error_fg_color),
    );
    ui.colored_label(ui.visuals().error_fg_color, error.content());
}

pub(crate) fn show_long_result_success(ui: &mut egui::Ui, content: &ComputeResults) {
    if content.results.len() > 5 {
        let header = content.results.iter().find(|x| x.is_ok());
        let Some(header) = header.as_ref() else {
            wasm_rs_dbg::dbg!("issue with header");
            return;
        };
        let header = header.as_ref().unwrap();
        // TODO
        let header = if let Some(result) = header.inner.result.as_object() {
            // for (name, v) in result {
            //     f(&mut head, name, v)
            // }
            (todo!(), Some(todo!()))
        } else if let Some(result) = header.inner.result.as_array() {
            // for (i, v) in result.iter().enumerate() {
            //     f(&mut head, &i.to_string(), v)
            // }
            (todo!(), None)
        } else {
            panic!()
        };
        egui::ScrollArea::horizontal()
            .scroll_bar_visibility(
                egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible,
            )
            .auto_shrink([false, false])
            .show(ui, |ui| {
                show_long_result_table(
                    ui,
                    (header.0, header.1, &content.results[1..]),
                    &mut None,
                    |_| None,
                )
            });
    } else {
        egui::CollapsingHeader::new("Results (JSON)")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(
                        egui::containers::scroll_area::ScrollBarVisibility::AlwaysHidden,
                    )
                    .auto_shrink([false, false])
                    .show(ui, |ui| show_long_result_list(ui, content));
            });
    }
}

pub(crate) fn show_long_result_list(ui: &mut egui::Ui, content: &ComputeResults) {
    for cont in &content.results {
        match cont {
            Ok(cont) => {
                let mut code: &str = &serde_json::to_string_pretty(&cont.inner.result).unwrap();
                let language = "json";
                let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
                let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                    let layout_job = egui_extras::syntax_highlighting::highlight(
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

pub trait PartialError<T> {
    fn error(&self) -> impl ToString;
    fn try_partial(&self) -> Option<&T>;
}

impl<T> PartialError<T> for String {
    fn error(&self) -> impl ToString {
        self
    }

    fn try_partial(&self) -> Option<&T> {
        None
    }
}

pub(crate) fn show_long_result_table(
    ui: &mut egui::Ui,
    content: (
        &[String],
        Option<&[String]>,
        &[Result<ComputeResultIdentified, impl PartialError<ComputeResultIdentified>>],
    ),
    selected_commit: &mut Option<usize>,
    commit_info: impl Fn(&str) -> Option<String>,
) {
    // header
    let header = content.0;
    // let len = |v: &serde_json::Value| v.as_object().map_or(1, |x| x.len());
    // let count = if let Some(result) = header.inner.result.as_object() {
    //     result.iter().map(|(_, v)| len(v)).sum()
    // } else if let Some(result) = header.inner.result.as_array() {
    //     result.iter().map(|v| len(v)).sum()
    // } else {
    //     panic!()
    // };
    let count = content.1.map_or(content.0.len(), |x| x.len());
    use egui_extras::{Column, TableBuilder};
    let mut table = TableBuilder::new(ui);
    if let Some(row) = selected_commit {
        table = table.scroll_to_row(*row, None);
    }
    table
        .striped(true)
        .resizable(true)
        // .auto_shrink([true, true])
        .column(Column::auto().resizable(true).clip(false))
        // .column(Column::remainder())
        .columns(Column::auto().resizable(true), count)
        // .column(Column::auto().resizable(true).clip(false))
        .column(Column::remainder())
        .header(20.0, |head| {
            show_table_header(head, header, None);
        })
        .body(|body| {
            show_table_body(body, content.2, selected_commit, commit_info);
        });
}

fn show_table_header(
    mut head: egui_extras::TableRow<'_, '_>,
    header: &[String],
    sub_header: Option<&[String]>,
) {
    let hf = |ui: &mut egui::Ui, name: &str| {
        ui.label(
            egui::RichText::new(name)
                .size(15.0)
                .text_style(egui::TextStyle::Monospace),
        )
    };
    head.col(|ui| {
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            hf(ui, " commit")
        });
    });
    // use serde_json::Value;
    // let f = |head: &mut egui_extras::TableRow<'_, '_>, name: &str, v: &Value| {
    //     if let Some(obj) = v.as_object() {
    //         for (n, _) in obj {
    //             let name = format!("{}_{}", name, n);
    //             head.col(move |ui| {
    //                 hf(ui, &name);
    //             });
    //         }
    //     } else {
    //         head.col(|ui| {
    //             hf(ui, name);
    //         });
    //     }
    // };

    // if let Some(result) = header.inner.result.as_object() {
    //     for (name, v) in result {
    //         f(&mut head, name, v)
    //     }
    // } else if let Some(result) = header.inner.result.as_array() {
    //     for (i, v) in result.iter().enumerate() {
    //         f(&mut head, &i.to_string(), v)
    //     }
    // } else {
    //     panic!()
    // };
    for h in header {
        head.col(|ui| {
            hf(ui, h);
        });
    }
    head.col(|ui| {
        hf(ui, "compute time");
    });
}

fn show_table_body(
    body: egui_extras::TableBody<'_>,
    content: &[Result<ComputeResultIdentified, impl PartialError<ComputeResultIdentified>>],
    selected_index: &mut Option<usize>,
    commit_info: impl Fn(&str) -> Option<String>,
) {
    use serde_json::Value;
    let f = |row: &mut egui_extras::TableRow<'_, '_>, v: &Value| {
        row.col(|ui| {
            ui.label(v.to_string());
        })
    };
    let g = |row: &mut egui_extras::TableRow<'_, '_>, v: &Value| {
        if let Some(obj) = v.as_object() {
            for (_, v) in obj {
                f(row, v);
            }
        } else {
            f(row, v);
        }
    };
    let show_row = |row: &mut egui_extras::TableRow<'_, '_>,
                    cont: &ComputeResultIdentified,
                    err: Option<String>| {
        // if let Some((_, c)) = &selected_commit {
        //     row.set_selected(c == &cont.commit);
        // }
        let i = row.index();
        row.col(|ui| {
            // if let Some((_, c)) = &selected_commit {
            //     // if c == &cont.commit {
            //     //     ui.scroll_to_rect(ui.min_rect(), Some(egui::Align::Center));
            //     // }
            // }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                if err.is_some() {
                    ui.colored_label(egui::Color32::RED, &cont.commit[..8])
                } else {
                    ui.label(&cont.commit[..8])
                }
                .on_hover_ui(|ui| {
                    if let Some(err) = err {
                        ui.colored_label(ui.visuals().error_fg_color, err);
                    } else {
                        ui.label(&cont.commit);
                    }
                })
            });
        })
        .1
        .on_hover_ui(|ui| {
            if let Some(text) = commit_info(&cont.commit) {
                ui.label("commit message:");
                ui.label(text);
            }
        });
        if let Some(result) = cont.inner.result.as_object() {
            for (_, v) in result {
                g(row, v);
            }
        } else if let Some(result) = cont.inner.result.as_array() {
            for v in result {
                g(row, v);
            }
        } else {
            panic!()
        };
        row.col(|ui| {
            ui.label(format!("{:.3}", SecFmt(cont.inner.compute_time)));
        });
    };
    body.rows(20.0, content.len(), |mut row| {
        if selected_index == &Some(row.index()) {
            row.set_selected(true);
        }
        match &content[row.index()] {
            Ok(cont) => show_row(&mut row, cont, None),
            Err(err) => {
                if let Some(cont) = err.try_partial() {
                    show_row(&mut row, cont, Some(err.error().to_string()))
                }
            }
        }
    });
}
