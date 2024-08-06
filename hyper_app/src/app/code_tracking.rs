use std::{
    collections::{hash_map, HashMap},
    ops::Range,
};

use crate::app::code_editor::generic_text_buffer::byte_index_from_char_index;
use egui::Id;
use egui_addon::{
    egui_utils::{highlight_byte_range, radio_collapsing, show_wip},
    interactive_split::interactive_splitter::InteractiveSplitter,
};
use poll_promise::Promise;

use super::{
    show_repo_menu,
    types::{self, CodeRange, Commit, Resource},
    utils_egui::MyUiExt as _,
    utils_poll::{self, Accumulable, Buffered},
};

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FetchedFile {
    pub content: String,
    pub line_breaks: Vec<usize>,
}

impl Resource<FetchedFile> {
    pub(super) fn from_response(_ctx: &egui::Context, response: ehttp::Response) -> Self {
        // wasm_rs_dbg::dbg!(&response);
        let _content_type = response.content_type().unwrap_or_default();
        // let image = if content_type.starts_with("image/") {
        //     RetainedImage::from_image_bytes(&response.url, &response.bytes).ok()
        // } else {
        //     None
        // };

        let text = response.text();
        // let colored_text = text.and_then(|text| syntax_highlighting(ctx, &response, text));
        let text = text.map(|x| {
            let content = x.to_string();
            let line_breaks = content
                .bytes()
                .enumerate()
                .filter_map(|(i, b)| if b == b'\n' { Some(i) } else { None })
                .collect();
            FetchedFile {
                content,
                line_breaks,
            }
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
    api_addr: &str,
    commit: &types::Commit,
    file_path: &str,
) -> RemoteFile {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/file/github/{}/{}/{}/{}",
        api_addr, &commit.repo.user, &commit.repo.name, &commit.id, &file_path,
    );

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);
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
    pub commits_processed: usize,
    pub src: CodeRange,
    pub intermediary: Option<CodeRange>,
    pub fallback: Option<CodeRange>,
    pub matched: Vec<CodeRange>,
}

impl PartialEq for TrackingResult {
    fn eq(&self, other: &Self) -> bool {
        self.src == other.src
            && self.intermediary == other.intermediary
            && self.fallback == other.fallback
            && self.matched == other.matched
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct TrackingResultWithChanges {
    pub track: TrackingResult,
    pub(crate) src_changes: Option<SrcChanges>,
    pub(crate) dst_changes: Option<DstChanges>,
}

impl From<TrackingResult> for TrackingResultWithChanges {
    fn from(value: TrackingResult) -> Self {
        Self {
            track: value,
            src_changes: Default::default(),
            dst_changes: Default::default(),
        }
    }
}
#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct TrackingResults {
    pub results: Vec<TrackingResult>,
}

impl Accumulable<TrackingResult> for TrackingResults {
    fn acc(&mut self, other: TrackingResult) -> bool {
        if self.results.contains(&other) {
            return false;
        }
        self.results.push(other);
        true
    }
}

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct TrackingResultsWithChanges {
    pub track: TrackingResults,
    pub(crate) src_changes: Option<SrcChanges>,
    pub(crate) dst_changes: Option<DstChanges>,
}

impl Accumulable<TrackingResultWithChanges> for TrackingResultsWithChanges {
    fn acc(&mut self, other: TrackingResultWithChanges) -> bool {
        if self.src_changes.is_none() {
            self.src_changes = other.src_changes;
        }
        if self.dst_changes.is_none() {
            self.dst_changes = other.dst_changes;
        }
        self.track.acc(other.track)
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct SrcChanges {
    #[serde(flatten)]
    pub(crate) commit: Commit,
    /// Global position of deleted elements
    pub(crate) deletions: Vec<u32>, // TODO diff encode
}
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct DstChanges {
    #[serde(flatten)]
    pub(crate) commit: Commit,
    /// Global position of added elements
    pub(crate) additions: Vec<u32>, // TODO diff encode
}

pub(super) type ComputeResult = Resource<TrackingResult>;
pub(super) type RemoteResult = ehttp::Result<ComputeResult>;

pub(super) fn track(
    ctx: &egui::Context,
    api_addr: &str,
    commit: &Commit,
    file_path: &String,
    range: &Option<Range<usize>>,
) -> Promise<RemoteResult> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = if let Some(range) = range {
        format!(
            "http://{}/track/github/{}/{}/{}/{}?start={}&end={}",
            api_addr,
            &commit.repo.user,
            &commit.repo.name,
            &commit.id,
            &file_path,
            &range.start,
            &range.end,
        )
    } else {
        format!(
            "http://{}/track/github/{}/{}/{}/{}",
            api_addr, &commit.repo.user, &commit.repo.name, &commit.id, &file_path,
        )
    };

    // wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);
    // request
    //     .headers
    //     .insert("Content-Type".to_string(), "text".to_string());

    ehttp::fetch(request, move |response| {
        // wasm_rs_dbg::dbg!(&response);
        ctx.request_repaint(); // wake up UI thread
        let resource =
            response.and_then(|response| Resource::<TrackingResult>::from_response(&ctx, response));
        sender.send(resource);
    });
    promise
}

impl Resource<TrackingResult> {
    pub(super) fn from_response(
        _ctx: &egui::Context,
        response: ehttp::Response,
    ) -> Result<Self, String> {
        // wasm_rs_dbg::dbg!(&response);
        // let content_type = response.content_type().unwrap_or_default();

        let text = response.text();
        let text = text.ok_or("")?;

        let text = if response.status == 200 {
            serde_json::from_str(text).map_err(|x| x.to_string())?
        } else {
            return Err(text.into());
        };
        // wasm_rs_dbg::dbg!(&text);

        Ok(Self {
            response,
            content: text,
        })
    }
}

impl Resource<TrackingResultWithChanges> {
    pub(super) fn from_response(
        _ctx: &egui::Context,
        response: ehttp::Response,
    ) -> Result<Self, String> {
        // wasm_rs_dbg::dbg!(&response);
        // let content_type = response.content_type().unwrap_or_default();

        let text = response.text();
        let text = text.ok_or("")?;

        let text = if response.status == 200 {
            serde_json::from_str(text).map_err(|x| x.to_string())?
        } else {
            return Err(text.into());
        };
        // wasm_rs_dbg::dbg!(&text);

        Ok(Self {
            response,
            content: text,
        })
    }
}

pub(crate) const WANTED: types::SelectedConfig = types::SelectedConfig::Tracking;

pub(crate) fn show_config(
    ui: &mut egui::Ui,
    tracking: &mut types::ComputeConfigTracking,
    tracking_result: &mut Buffered<Result<Resource<TrackingResult>, String>>,
) {
    let repo_changed = show_repo_menu(ui, &mut tracking.target.file.commit.repo);
    let old = tracking.target.file.commit.id.clone();
    let commit_te = egui::TextEdit::singleline(&mut tracking.target.file.commit.id)
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
        ui.wip(Some("need more parameters ?"));
    });
}

pub(super) fn show_code_tracking_results(
    ui: &mut egui::Ui,
    api_addr: &str,
    tracking: &mut types::ComputeConfigTracking,
    tracking_result: &mut utils_poll::Buffered<RemoteResult>,
    fetched_files: &mut HashMap<types::FileIdentifier, RemoteFile>,
    ctx: &egui::Context,
) {
    let result_changed = tracking_result.try_poll();
    InteractiveSplitter::vertical().show(ui, |ui1, ui2| {
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

            let te = ui
                .show_remote_code(
                    api_addr,
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
                    let rect = highlight_byte_range(ui, &aa.inner, selected_node, color);
                    // aa.inner.response.context_menu(|ui| {
                    //     if ui.button("Close the menu").clicked() {
                    //         ui.close_menu();
                    //     }
                    // });
                    if result_changed {
                        // wasm_rs_dbg::dbg!(
                        //     aa.content_size,
                        //     aa.state.offset.y,
                        //     aa.inner_rect.height(),
                        //     rect.top(),
                        // );
                        pos_ratio = Some((rect.top() - aa.state.offset.y) / aa.inner_rect.height());
                        // wasm_rs_dbg::dbg!(pos_ratio);
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
                            // wasm_rs_dbg::dbg!(&r);
                            tracking.target.range = Some(r);
                            tracking_result.buffer(track(
                                ctx,
                                api_addr,
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
                            let te = ui
                                .show_remote_code(
                                    api_addr,
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
                                    let rect =
                                        highlight_byte_range(ui, &aa.inner, selected_node, color);
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
                                        // wasm_rs_dbg::dbg!(result_changed);
                                        // wasm_rs_dbg::dbg!(pos_ratio);
                                        if b.is_some() {
                                            ctx.memory_mut(|mem| {
                                                mem.data
                                                    .remove::<Option<f32>>(Id::new(&matched.file));
                                            });
                                        }

                                        // wasm_rs_dbg::dbg!(
                                        //     aa.content_size,
                                        //     aa.state.offset.y,
                                        //     aa.inner_rect.height(),
                                        //     rect.top(),
                                        // );
                                        let pos_ratio = pos_ratio.unwrap_or(b.unwrap_or(0.5));
                                        let qq = pos_ratio * aa.inner_rect.height();
                                        aa.state.offset.y = rect.top() - qq;
                                        aa.state.store(ui.ctx(), aa.id);
                                    }
                                }
                            }
                        }
                    } else {
                        // wasm_rs_dbg::dbg!(&track_result);
                    }
                }
                Some(Err(_err)) => {
                    // wasm_rs_dbg::dbg!(err);
                }
                None => {
                    // wasm_rs_dbg::dbg!();
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
