use std::{
    collections::{hash_map, HashMap},
    ops::Range,
};

use egui::Id;
use poll_promise::Promise;

use crate::app::{
    code_editor::generic_text_buffer::byte_index_from_char_index, egui_utils::highlight_byte_range,
    interactive_split, show_remote_code, API_URL,
};

use super::{
    egui_utils::{radio_collapsing, show_wip},
    show_repo,
    types::CodeRange,
    types::{self, Commit, Resource},
    Buffered,
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FetchedFile {
    pub content: String,
}

impl Resource<FetchedFile> {
    pub(super) fn from_response(ctx: &egui::Context, response: ehttp::Response) -> Self {
        wasm_rs_dbg::dbg!(&response);
        let content_type = response.content_type().unwrap_or_default();
        // let image = if content_type.starts_with("image/") {
        //     RetainedImage::from_image_bytes(&response.url, &response.bytes).ok()
        // } else {
        //     None
        // };

        let text = response.text();
        // let colored_text = text.and_then(|text| syntax_highlighting(ctx, &response, text));
        let text = text.map(|x| FetchedFile {
            content: x.to_string(),
        });

        Self {
            response,
            content: text,
            // image,
            // text: colored_text,
        }
    }
}

pub(super) type RemoteFile = Promise<ehttp::Result<Resource<FetchedFile>>>;

pub(super) fn remote_fetch_file(
    ctx: &egui::Context,
    commit: &types::Commit,
    file_path: &str,
) -> Promise<Result<Resource<FetchedFile>, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "{}/file/github/{}/{}/{}/{}",
        API_URL, &commit.repo.user, &commit.repo.name, &commit.id, &file_path,
    );

    wasm_rs_dbg::dbg!(&url);
    let mut request = ehttp::Request::get(&url);
    // request
    //     .headers
    //     .insert("Content-Type".to_string(), "text".to_string());

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        let resource =
            response.map(|response| Resource::<FetchedFile>::from_response(&ctx, response));
        sender.send(resource);
    });
    promise
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TrackingResult {
    pub compute_time: f64,
    pub src: CodeRange,
    pub matched: Vec<CodeRange>,
}

pub(super) type ComputeResult = Resource<TrackingResult>;
pub(super) type RemoteResult = ehttp::Result<Resource<TrackingResult>>;

pub(super) fn track(
    ctx: &egui::Context,
    commit: &Commit,
    file_path: &String,
    range: &Option<Range<usize>>,
) -> Promise<RemoteResult> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = if let Some(range) = range {
        format!(
            "{}/track/github/{}/{}/{}/{}?start={}&end={}",
            API_URL,
            &commit.repo.user,
            &commit.repo.name,
            &commit.id,
            &file_path,
            &range.start,
            &range.end,
        )
    } else {
        format!(
            "{}/track/github/{}/{}/{}/{}",
            API_URL, &commit.repo.user, &commit.repo.name, &commit.id, &file_path,
        )
    };

    wasm_rs_dbg::dbg!(&url);
    let mut request = ehttp::Request::get(&url);
    // request
    //     .headers
    //     .insert("Content-Type".to_string(), "text".to_string());

    ehttp::fetch(request, move |response| {
        wasm_rs_dbg::dbg!(&response);
        ctx.request_repaint(); // wake up UI thread
        let resource =
            response.and_then(|response| Resource::<TrackingResult>::from_response(&ctx, response));
        sender.send(resource);
    });
    promise
}

impl Resource<TrackingResult> {
    pub(super) fn from_response(ctx: &egui::Context, response: ehttp::Response) -> Result<Self, String> {
        wasm_rs_dbg::dbg!(&response);
        // let content_type = response.content_type().unwrap_or_default();

        let text = response.text();
        let text = text.ok_or("")?;
        let text = serde_json::from_str(text).map_err(|x| x.to_string())?;
        wasm_rs_dbg::dbg!(&text);

        Ok(Self {
            response,
            content: text,
        })
    }
}

pub(super) fn show_code_tracking_menu(
    ui: &mut egui::Ui,
    selected: &mut types::SelectedConfig,
    tracking: &mut types::ComputeConfigTracking,
    tracking_result: &mut Buffered<Result<Resource<TrackingResult>, String>>,
) {
    let title = "Code Tracking";
    let wanted = types::SelectedConfig::Tracking;
    let id = ui.make_persistent_id(title);

    let add_body = |ui: &mut egui::Ui| {
        let repo_changed = show_repo(ui, &mut tracking.target.file.commit.repo);
        let old = tracking.target.file.commit.id.clone();
        let commit_te = 
            egui::TextEdit::singleline(&mut tracking.target.file.commit.id)
                .clip_text(true)
                .desired_width(150.0)
                .desired_rows(1)
                .hint_text("commit")
                .id(ui.id().with("commit"))
                .interactive(true)
                .show(ui);
        if repo_changed || commit_te.response.changed() {
            tracking_result.take();
            *tracking_result = Default::default();
            tracking.target.range.take();
        } else {
            assert_eq!(old, tracking.target.file.commit.id.clone());
        };

        ui.add_enabled_ui(false, |ui| {
            // ui.add(
            //     egui::Slider::new(&mut tracking.len, 0..=200)
            //         .text("commits")
            //         .clamp_to_range(false)
            //         .integer()
            //         .logarithmic(true),
            // );
            show_wip(ui, Some("need more parameters ?"));
        });
    };

    radio_collapsing(ui, id, title, selected, wanted, add_body);
}

pub(super) fn show_code_tracking_results(
    ui: &mut egui::Ui,
    tracking: &mut types::ComputeConfigTracking,
    tracking_result: &mut Buffered<RemoteResult>,
    fetched_files: &mut HashMap<types::FileIdentifier, RemoteFile>,
    ctx: &egui::Context,
) {
    use interactive_split::Splitter;
    let result_changed = tracking_result.try_poll();
    Splitter::vertical().show(ui, |ui1, ui2| {
        let mut pos_ratio = None;
        ui2.push_id(ui2.id().with("second"), |ui| {
            let file_result = fetched_files.entry(tracking.target.file.clone());
            let selected_node = tracking_result
                .get_mut()
                .and_then(|x| x.as_ref().ok())
                .and_then(|x| x.content.as_ref())
                .and_then(|x| x.src.range.clone());
            if tracking.target.range.is_none() {
                assert!(selected_node.is_none())
            }

            let te = show_remote_code(
                ui,
                &mut tracking.target.file.commit,
                &mut tracking.target.file.file_path,
                file_result,
                // ctx,
            )
            .2;
            if let Some(egui::InnerResponse {
                inner: Some(aa), ..
            }) = te
            {
                // if let Some(selected_node) = &tracking.target.range {
                //     let color = egui::Color32::RED.linear_multiply(0.1);
                //     let rect = highlight_byte_range(ui, &aa, selected_node,color);
                // }
                if let Some(selected_node) = &selected_node {
                    let color = egui::Color32::GREEN.linear_multiply(0.1);
                    let rect = highlight_byte_range(ui, &aa, selected_node, color);
                    // aa.inner.response.context_menu(|ui| {
                    //     if ui.button("Close the menu").clicked() {
                    //         ui.close_menu();
                    //     }
                    // });
                    if result_changed {
                        wasm_rs_dbg::dbg!(
                            aa.content_size,
                            aa.state.offset.y,
                            aa.inner_rect.height(),
                            rect.top(),
                        );
                        pos_ratio = Some((rect.top() - aa.state.offset.y) / aa.inner_rect.height());
                        wasm_rs_dbg::dbg!(pos_ratio);
                    }
                }
                if !aa.inner.response.is_pointer_button_down_on() {
                    let bb = &aa.inner.cursor_range;
                    if let Some(bb) = bb {
                        let s = aa.inner.galley.text();
                        let r = bb.as_sorted_char_range();
                        let r = Range {
                            start: byte_index_from_char_index(s, r.start),
                            end: byte_index_from_char_index(s, r.end),
                        };
                        if tracking.target.range != Some(r.clone()) {
                            wasm_rs_dbg::dbg!(&r);
                            tracking.target.range = Some(r);
                            tracking_result.buffer(track(
                                ctx,
                                &tracking.target.file.commit,
                                &tracking.target.file.file_path,
                                &tracking.target.range,
                            ));
                        }
                    }
                }
            };
            ui.separator();
        });
        ui1.push_id(ui1.id().with("first"), |ui| {
            match tracking_result.get_mut() {
                Some(Ok(track_result)) => {
                    if let Some(content) = &track_result.content {
                        if content.matched.is_empty() {
                            ui.label(format!(
                                "{} element matching for {} in previous commit",
                                content.matched.len(),
                                serde_json::to_string_pretty(&content.src)
                                    .unwrap_or_else(|x| x.to_string())
                            ));
                        }
                        for matched in &content.matched {
                            dbg!(matched);
                            let file_result = fetched_files.entry(matched.file.clone());
                            if let hash_map::Entry::Vacant(_) = file_result {
                                ctx.memory_mut(|mem| {
                                    mem.data.insert_temp(Id::new(&matched.file), pos_ratio)
                                });
                            }
                            let te = show_remote_code(
                                ui,
                                &mut matched.file.commit.clone(),
                                &mut matched.file.file_path.clone(),
                                file_result,
                                // ctx,
                            )
                            .2;
                            if let Some(egui::InnerResponse {
                                inner: Some(mut aa),
                                ..
                            }) = te
                            {
                                if let Some(selected_node) = &matched.range {
                                    let color = egui::Color32::GREEN.linear_multiply(0.1);
                                    let rect = highlight_byte_range(ui, &aa, selected_node, color);
                                    aa.inner.response.context_menu(|ui| {
                                        if ui.button("Close the menu").clicked() {
                                            ui.close_menu();
                                        }
                                    });
                                    let b = ctx.memory_mut(|mem| {
                                        mem.data
                                            .get_temp::<Option<f32>>(Id::new(&matched.file))
                                            .map(|x| x.unwrap_or(0.5))
                                    });
                                    if result_changed || b.is_some() {
                                        wasm_rs_dbg::dbg!(result_changed);
                                        wasm_rs_dbg::dbg!(pos_ratio);
                                        if b.is_some() {
                                            ctx.memory_mut(|mem| {
                                                mem.data
                                                    .remove::<Option<f32>>(Id::new(&matched.file));
                                            });
                                        }

                                        wasm_rs_dbg::dbg!(
                                            aa.content_size,
                                            aa.state.offset.y,
                                            aa.inner_rect.height(),
                                            rect.top(),
                                        );
                                        let pos_ratio = pos_ratio.unwrap_or(b.unwrap_or(0.5));
                                        let qq = pos_ratio * aa.inner_rect.height();
                                        aa.state.offset.y = rect.top() - qq;
                                        aa.state.store(ui.ctx(), aa.id);
                                    }
                                }
                            }
                        }
                    } else {
                        wasm_rs_dbg::dbg!(&track_result);
                    }
                }
                Some(Err(err)) => {
                    wasm_rs_dbg::dbg!(err);
                }
                None => {
                    wasm_rs_dbg::dbg!();
                    // *track_result = Some(code_tracking::track(
                    //     ctx,
                    //     &tracking.target.file.commit,
                    //     &tracking.target.file.file_path,
                    //     &tracking.target.range,
                    // ));
                }
            }
            ui.separator();
        });
    });
}