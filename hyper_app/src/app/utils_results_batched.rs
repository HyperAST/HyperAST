use poll_promise::Promise;

use crate::app::types;
use crate::app::utils;
use crate::app::utils::SecFmt;

use super::types::Resource;

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

pub(super) fn show_long_result(
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
                show_long_result_compute_failure(error, ui);
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

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ComputeResults {
    pub prepare_time: f64,
    pub results: Vec<Result<ComputeResultIdentified, String>>,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ComputeResultIdentified {
    pub commit: types::CommitId,
    #[serde(flatten)]
    pub inner: ComputeResult,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct ComputeResult {
    pub compute_time: f64,
    pub result: serde_json::Value,
}

fn show_long_result_compute_failure<'a>(error: &impl ComputeError, ui: &mut egui::Ui) {
    ui.label(
        egui::RichText::new(error.head())
            .heading()
            .color(ui.visuals().error_fg_color),
    );
    ui.colored_label(ui.visuals().error_fg_color, error.content());
}

pub(crate) fn show_long_result_success(ui: &mut egui::Ui, content: &ComputeResults) {
    if content.results.len() > 5 {
        egui::ScrollArea::horizontal()
            .scroll_bar_visibility(
                egui::containers::scroll_area::ScrollBarVisibility::AlwaysVisible,
            )
            .auto_shrink([false, false])
            .show(ui, |ui| show_long_result_table(content, ui));
    } else {
        egui::CollapsingHeader::new("Results (JSON)")
            .default_open(true)
            .show(ui, |ui| {
                egui::ScrollArea::vertical()
                    .scroll_bar_visibility(
                        egui::containers::scroll_area::ScrollBarVisibility::AlwaysHidden,
                    )
                    .auto_shrink([false, false])
                    .show(ui, |ui| show_long_result_list(content, ui));
            });
    }
}

pub(crate) fn show_long_result_list(content: &ComputeResults, ui: &mut egui::Ui) {
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

pub(crate) fn show_long_result_table(content: &ComputeResults, ui: &mut egui::Ui) {
    // header
    let header = content.results.iter().find(|x| x.is_ok());
    let Some(header) = header.as_ref() else {
        wasm_rs_dbg::dbg!("issue with header");
        return;
    };
    let header = header.as_ref().unwrap();
    use egui_extras::{Column, TableBuilder};
    let len = |v: &serde_json::Value| v.as_object().map_or(1, |x| x.len());
    let count = if let Some(result) = header.inner.result.as_object() {
        result.iter().map(|(_, v)| len(v)).sum()
    } else if let Some(result) = header.inner.result.as_array() {
        result.iter().map(|v| len(v)).sum()
    } else {
        panic!()
    };
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .auto_shrink([true, true])
        .column(Column::auto().resizable(true).clip(false))
        // .column(Column::remainder())
        .columns(Column::auto().resizable(true), count)
        .column(Column::auto().resizable(true).clip(false))
        .header(20.0, |head| {
            show_table_header(head, header);
        })
        .body(|body| {
            show_table_body(body, content);
        });
}

fn show_table_header(mut head: egui_extras::TableRow<'_, '_>, header: &ComputeResultIdentified) {
    let hf = |ui: &mut egui::Ui, name: &str| {
        ui.label(
            egui::RichText::new(name)
                .size(15.0)
                .text_style(egui::TextStyle::Monospace),
        )
    };
    head.col(|ui| {
        hf(ui, " commit");
    });
    use serde_json::Value;
    let f = |head: &mut egui_extras::TableRow<'_, '_>, name: &str, v: &Value| {
        if let Some(obj) = v.as_object() {
            for (n, _) in obj {
                let name = format!("{}_{}", name, n);
                head.col(move |ui| {
                    hf(ui, &name);
                });
            }
        } else {
            head.col(|ui| {
                hf(ui, name);
            });
        }
    };
    if let Some(result) = header.inner.result.as_object() {
        for (name, v) in result {
            f(&mut head, name, v)
        }
    } else if let Some(result) = header.inner.result.as_array() {
        for (i, v) in result.iter().enumerate() {
            f(&mut head, &i.to_string(), v)
        }
    } else {
        panic!()
    };
    head.col(|ui| {
        hf(ui, "compute time");
    });
}

fn show_table_body(mut body: egui_extras::TableBody<'_>, content: &ComputeResults) {
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
    let show_ok = |row: &mut egui_extras::TableRow<'_, '_>, cont: &ComputeResultIdentified| {
        row.col(|ui| {
            ui.label(&cont.commit[..8]);
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
    for cont in &content.results {
        match cont {
            Ok(cont) => body.row(30.0, |mut row| show_ok(&mut row, cont)),
            Err(err) => {
                body.row(30.0, |mut row| {
                    row.col(|ui| {
                        ui.colored_label(ui.visuals().error_fg_color, err);
                    });
                });
            }
        }
    }
}
