use std::{
    collections::{HashMap, VecDeque},
    ops::Range,
    time::{Duration, SystemTime},
};

use chrono::Timelike;
use poll_promise::Promise;

use crate::app::{
    code_editor::generic_text_buffer::byte_index_from_char_index, egui_utils::highlight_byte_range,
    long_tracking, show_remote_code, show_remote_code2, types::Resource, API_URL,
};

use super::{
    code_tracking::{self, RemoteFile, TrackingResult},
    egui_utils::{radio_collapsing, show_wip},
    show_repo,
    types::{self, CodeRange, Commit},
    Buffered,
};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct LongTacking {
    pub(crate) target: CodeRange,
    pub(crate) target_index: usize,
    pub(crate) results: VecDeque<(
        Buffered<Result<CommitMetadata, String>>,
        Buffered<Result<code_tracking::TrackingResult, String>>,
    )>,
}

impl Default for LongTacking {
    fn default() -> Self {
        Self {
            target: Default::default(),
            target_index: Default::default(),
            results: VecDeque::from(vec![Default::default()]),
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct CommitMetadata {
    /// commit message
    message: Option<String>,
    /// parents commits
    /// if multiple parents, the first one should be where the merge happends
    parents: Vec<String>,
    /// tree corresponding to version
    tree: Option<String>,
    /// offset in minutes
    timezone: i32,
    /// seconds
    time: i64,
}

pub(super) fn commit(
    ctx: &egui::Context,
    commit: &Commit,
) -> Promise<Result<CommitMetadata, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "{}/commit/github/{}/{}/{}",
        API_URL, &commit.repo.user, &commit.repo.name, &commit.id,
    );

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);
    // request
    //     .headers
    //     .insert("Content-Type".to_string(), "text".to_string());

    ehttp::fetch(request, move |response| {
        wasm_rs_dbg::dbg!(&response);
        ctx.request_repaint(); // wake up UI thread
        let resource = response
            .and_then(|response| Resource::<CommitMetadata>::from_response(&ctx, response))
            .and_then(|x| x.content.ok_or("No content".into()));
        sender.send(resource);
    });
    promise
}

impl Resource<CommitMetadata> {
    fn from_response(ctx: &egui::Context, response: ehttp::Response) -> Result<Self, String> {
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

pub(super) fn show_menu(
    ui: &mut egui::Ui,
    selected: &mut types::SelectedConfig,
    tracking: &mut LongTacking,
) {
    let title = "Long Tracking";
    let wanted = types::SelectedConfig::LongTracking;
    let id = ui.make_persistent_id(title);

    let add_body = |ui: &mut egui::Ui| {
        let repo_changed = show_repo(ui, &mut tracking.target.file.commit.repo);
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
            todo!()
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

#[derive(Clone, Copy, Debug)]
// #[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
// #[cfg_attr(feature = "serde", serde(default))]
pub struct State {
    pub(crate) offset: f32,
    pub(crate) width: f32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            offset: 0.0,
            width: 1.0,
        }
    }
}

impl State {
    pub fn load(ctx: &egui::Context, id: egui::Id) -> Option<Self> {
        ctx.data_mut(|d| d.get_persisted(id))
    }

    pub fn store(self, ctx: &egui::Context, id: egui::Id) {
        ctx.data_mut(|d| d.insert_persisted(id, self));
    }
}

pub(crate) fn show_results(
    ui: &mut egui::Ui,
    long_tracking: &mut LongTacking,
    fetched_files: &mut HashMap<types::FileIdentifier, RemoteFile>,
) {
    use super::multi_split::Splitter;
    let w_id = ui.id().with("state");
    let timeline_window = ui.available_rect_before_wrap();
    let spacing: egui::Vec2 = (0.0, 0.0).into(); //4.0 // ui.spacing().item_spacing;
    let mut w_state = State::load(ui.ctx(), w_id);
    // wasm_rs_dbg::dbg!(&long_tracking.results);
    let (total_cols, col_width) = if long_tracking.results.len() <= 2 {
        (
            long_tracking.results.len(),
            ui.available_width() / long_tracking.results.len() as f32,
        )
    } else {
        let width = if let Some(w_state) = w_state {
            // ui.painter().debug_rect(
            //     egui::Rect::from_x_y_ranges(w_state.offset..=w_state.width, timeline_window.y_range()),
            //     egui::Color32::GREEN,
            //     format!("text\n\n{}", w_state.width),
            // );
            // ui.painter().debug_rect(
            //     egui::Rect::from_x_y_ranges(w_state.offset..=w_state.width, timeline_window.y_range()),
            //     egui::Color32::RED,
            //     format!("text\n\n{}", w_state.width),
            // );
            w_state.width
        } else {
            w_state = Some(State {
                offset: 0.0,
                width: timeline_window.width() * 0.4,
            });
            timeline_window.width() * 0.4
        };
        // let s = s.map_or(0.4, |x| x.col_ratio);
        (long_tracking.results.len(), width)
    };
    let col_width_with_spacing = col_width + spacing.x;
    let viewport_width = (col_width_with_spacing * total_cols as f32 - spacing.x).at_least(0.0);

    // let w_state = ui.make_persistent_id(egui::Id::new("Timeline"));
    let timeline_window_width = timeline_window.width();
    egui::panel::TopBottomPanel::bottom("Timeline Map")
        .height_range(0.0..=ui.available_height() / 3.0)
        .default_height(ui.available_height() / 5.0)
        .resizable(true)
        .show_inside(ui, |ui| {
            let mut add_content = |ui: &mut egui::Ui, col: usize| {
                let mut aaa = (Buffered::Empty, Buffered::Empty);
                let (md, code_range) = if long_tracking.results.is_empty() {
                    &mut aaa //&long_tracking.target
                } else {
                    let res = &mut long_tracking.results[col];
                    res.1.try_poll();
                    res.0.try_poll();
                    res
                };
                let (code_range, md) = if col == long_tracking.target_index {
                    if let Some(md) = md.get_mut() {
                        let code_range = &mut long_tracking.target;
                        (code_range, md)
                    } else {
                        if !md.is_waiting() {
                            let code_range = &mut long_tracking.target;
                            wasm_rs_dbg::dbg!(&code_range);
                            md.buffer(commit(ui.ctx(), &code_range.file.commit));
                        }
                        return;
                    }
                } else if let (Some(code_range), Some(md)) = (code_range.get_mut(), md.get_mut()) {
                    match code_range {
                        Ok(code_range) => (&mut code_range.matched[0], md),
                        Err(err) => panic!("{}", err),
                    }
                } else if let Some(Ok(code_range)) = code_range.get_mut() {
                    if !md.is_waiting() {
                        let code_range = &mut code_range.matched[0];
                        wasm_rs_dbg::dbg!(&code_range);
                        md.buffer(commit(ui.ctx(), &code_range.file.commit));
                    }
                    ui.spinner();
                    return;
                } else {
                    ui.spinner();
                    return;
                };
                ui.label(format!("commit {}", &code_range.file.commit.id));
                match md {
                    Ok(md) => {
                        let tz = &chrono::FixedOffset::west_opt(md.timezone * 60).unwrap();
                        let date = chrono::Duration::seconds(md.time);
                        let date = chrono::DateTime::<chrono::FixedOffset>::default()
                            .with_timezone(tz)
                            .checked_add_signed(date);
                        if let Some(date) = date {
                            ui.label(format!("Date:\t{:?}", date));
                        } else {
                            // wasm_rs_dbg::dbg!(md.timezone, md.time);
                        }
                        if let Some(msg) = &md.message {
                            ui.text_edit_multiline(&mut msg.to_string());
                        }
                        ui.label(format!("Parents: {}", md.parents.join(" + ")));
                        // ui.with_layout(egui::Layout::right_to_left(egui::Align::Min), |ui| {
                        //     ui.label(format!("Pparents: {}",md.parents.join(" + ")));
                        // });
                    }
                    Err(err) => {
                        ui.colored_label(ui.visuals().error_fg_color, err);
                    }
                }
                // if let Some(scroll_state) = scroll_state {
                //     ui.label(format!(
                //         "Timeline left pos {:?}/{}",
                //         scroll_state.offset.x,
                //         col_width_with_spacing * total_cols as f32 - spacing.x,
                //     ));
                //     ui.label(format!(
                //         "Timeline right pos {:?}/{}",
                //         scroll_state.offset.x + timeline_window_width,
                //         col_width_with_spacing * total_cols as f32 - spacing.x,
                //     ));
                // }
            };
            if total_cols == 0 {
                ui.spinner();
            } else if total_cols == 1 {
                add_content(ui, 0);
            } else {
                let ratios = (0..total_cols - 1)
                    .map(|_| 1.0 / (total_cols) as f32)
                    .collect();
                Splitter::vertical().ratios(ratios).show(ui, |uis| {
                    for (col, ui) in uis.into_iter().enumerate() {
                        add_content(ui, col);
                    }
                });
            }
            if let Some(mut w_state) = w_state {
                let tl_win_left = w_state.offset;
                let tl_win_right = tl_win_left + timeline_window_width;
                let tl_rel_range = tl_win_left / viewport_width..=tl_win_right / viewport_width;
                let tl_range = egui::lerp(ui.max_rect().x_range(), *tl_rel_range.start())
                    ..=egui::lerp(ui.max_rect().x_range(), *tl_rel_range.end());
                let map_left = egui::remap_clamp(w_state.offset, 0.0..=viewport_width, ui.max_rect().x_range());
                let map_right = egui::remap_clamp(w_state.offset+timeline_window_width, 0.0..=viewport_width, ui.max_rect().x_range());
                let map_width = timeline_window_width/viewport_width*ui.max_rect().width();
                // assert!(map_left+map_width-map_right<1.0,"{} {} {}",map_left,map_width,map_right);
                // w = rs + tww / vw * rw
                // ui.painter_at(ui.max_rect()).debug_rect(
                //     egui::Rect::from_x_y_ranges(tl_range, ui.max_rect().y_range()),
                //     egui::Color32::RED,
                //     "",
                // );
                let rect = egui::Rect::from_x_y_ranges(map_left..=map_left+map_width, ui.max_rect().y_range());//tl_range
                let painter = ui.painter_at(ui.max_rect());
                {
                    let map_drag = egui::Area::new(ui.id().with("map_drag"))
                        .drag_bounds(ui.max_rect())
                        .current_pos(rect.center())
                        .show(ui.ctx(), |ui| ui.label("↔"))
                        .response;
                    let fill_color;
                    if map_drag.dragged() {
                        let delta = map_drag.drag_delta();
                        if delta.x != 0.0 {
                            // ui.painter().debug_rect(
                            //     ui.max_rect(),
                            //     egui::Color32::GREEN,
                            //     format!("text\n\n{}", w_state.width),
                            // );
                            w_state.offset = (w_state.offset
                                + delta.x / ui.max_rect().width() * viewport_width)
                                .clamp(0.0, viewport_width - timeline_window.width());
                            // w_state.width = (w_state.width + delta.x).at_most(timeline_window.right());
                            // w_state.store(ui.ctx(), w_id);
                        }
                        fill_color = egui::Color32::DARK_GRAY.linear_multiply(0.8)
                    } else {
                        fill_color = egui::Color32::DARK_GRAY.linear_multiply(0.4)
                    }
                    painter.rect(
                        rect,
                        egui::Rounding::none(),
                        fill_color,
                        egui::Stroke::new(1.0, egui::Color32::DARK_GRAY),
                    );
                }
                {
                    let resizable = true;
                    let mut resize_hover = false;
                    let mut is_resizing = false;
                    if resizable {
                        let resize_id = ui.id().with("__resize_l");
                        if let Some(pointer) = ui.ctx().pointer_latest_pos() {
                            let we_are_on_top = ui
                                .ctx()
                                .layer_id_at(pointer)
                                .map_or(true, |top_layer_id| top_layer_id == ui.layer_id());
                            let mouse_over_resize_line = we_are_on_top
                                && rect.y_range().contains(&pointer.y)
                                && (rect.left() - pointer.x).abs()
                                    <= ui.style().interaction.resize_grab_radius_side;

                            if ui.input(|i| i.pointer.any_pressed() && i.pointer.any_down())
                                && mouse_over_resize_line
                            {
                                ui.memory_mut(|mem| mem.set_dragged_id(resize_id));
                            }
                            is_resizing = ui.memory(|mem| mem.is_being_dragged(resize_id));
                            if is_resizing {
                                // let width = (pointer.x - second_rect.left()).abs();
                                // let width =
                                //     clamp_to_range(width, width_range.clone()).at_most(available_rect.width());
                                // second_rect.min.x = second_rect.max.x - width;
                                let x = pointer.x.clamp(ui.max_rect().min.x, ui.max_rect().max.x);
                                let f = (rect.max.x - x).at_least(10.0);// - rect.min.x;
                                let col_ratio = ui.max_rect().width() / total_cols as f32 / f;
                                w_state.width = timeline_window.width() * col_ratio;

                                let col_width_with_spacing = w_state.width + spacing.x;
                                let viewport_width = (col_width_with_spacing * total_cols as f32 - spacing.x).at_least(0.0);
                                w_state.offset = egui::remap_clamp(map_right, ui.max_rect().x_range(), 0.0..=viewport_width)-timeline_window_width;

                                // let map_r = egui::remap_clamp(w_state.offset+timeline_window_width, 0.0..=viewport_width, ui.max_rect().x_range());
                                // assert!((map_r-map_right).abs()<1.0);

                            }

                            let dragging_something_else =
                                ui.input(|i| i.pointer.any_down() || i.pointer.any_pressed());
                            resize_hover = mouse_over_resize_line && !dragging_something_else;

                            if resize_hover || is_resizing {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                            }
                        }
                    }

                    let stroke = if is_resizing {
                        ui.style().visuals.widgets.active.fg_stroke // highly visible
                    } else if resize_hover {
                        ui.style().visuals.widgets.hovered.fg_stroke // highly visible
                    } else if true {
                        //show_separator_line {
                        // TOOD(emilk): distinguish resizable from non-resizable
                        ui.style().visuals.widgets.noninteractive.bg_stroke // dim
                    } else {
                        egui::Stroke::NONE
                    };

                    painter.vline(rect.left(), rect.y_range(), stroke);
                }
                {
                    let resizable = true;
                    let mut resize_hover = false;
                    let mut is_resizing = false;
                    if resizable {
                        let resize_id = ui.id().with("__resize_r");
                        if let Some(pointer) = ui.ctx().pointer_latest_pos() {
                            let we_are_on_top = ui
                                .ctx()
                                .layer_id_at(pointer)
                                .map_or(true, |top_layer_id| top_layer_id == ui.layer_id());
                            let mouse_over_resize_line = we_are_on_top
                                && rect.y_range().contains(&pointer.y)
                                && (rect.right() - pointer.x).abs()
                                    <= ui.style().interaction.resize_grab_radius_side;

                            if ui.input(|i| i.pointer.any_pressed() && i.pointer.any_down())
                                && mouse_over_resize_line
                            {
                                ui.memory_mut(|mem| mem.set_dragged_id(resize_id));
                            }
                            is_resizing = ui.memory(|mem| mem.is_being_dragged(resize_id));
                            let x = pointer.x;
                            let f = x - rect.min.x;
                            let f = f.at_least(0.0);
                            let r = f / ui.max_rect().width();
                            let r2 = r / total_cols as f32;
                            // ui.painter().debug_rect(
                            //     ui.max_rect(),
                            //     egui::Color32::RED,
                            //     format!(
                            //         "{}\n{}\n{}\n{}",
                            //         f,
                            //         r,
                            //         r2,
                            //         (ui.max_rect().width() / total_cols as f32) / f
                            //     ),
                            // );
                            if is_resizing {
                                // let width = (pointer.x - second_rect.left()).abs();
                                // let width =
                                //     clamp_to_range(width, width_range.clone()).at_most(available_rect.width());
                                // second_rect.min.x = second_rect.max.x - width;
                                let x = pointer.x.clamp(ui.max_rect().min.x, ui.max_rect().max.x);
                                let f = (x - rect.min.x).at_least(10.0);
                                // ratio = (f / rect.width()).clamp(0.1, 0.9);
                                let col_ratio = ui.max_rect().width() / total_cols as f32 / f;
                                // w_state.end = f;
                                w_state.width = timeline_window.width() * col_ratio;
                                // (f / ui.max_rect().width() * viewport_width)
                                //     .clamp(col_width, viewport_width - w_state.offset);

                                let col_width_with_spacing = w_state.width + spacing.x;
                                let viewport_width = (col_width_with_spacing * total_cols as f32 - spacing.x).at_least(0.0);
                                w_state.offset = egui::remap_clamp(map_left, ui.max_rect().x_range(), 0.0..=viewport_width);

                                // let map_l = egui::remap_clamp(w_state.offset, 0.0..=viewport_width, ui.max_rect().x_range());
                                // assert!((map_l-map_left).abs()<f32::EPSILON);
                            }

                            let dragging_something_else =
                                ui.input(|i| i.pointer.any_down() || i.pointer.any_pressed());
                            resize_hover = mouse_over_resize_line && !dragging_something_else;

                            if resize_hover || is_resizing {
                                ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                            }
                        }
                    }

                    let stroke = if is_resizing {
                        ui.style().visuals.widgets.active.fg_stroke // highly visible
                    } else if resize_hover {
                        ui.style().visuals.widgets.hovered.fg_stroke // highly visible
                    } else if true {
                        //show_separator_line {
                        // TOOD(emilk): distinguish resizable from non-resizable
                        ui.style().visuals.widgets.noninteractive.bg_stroke // dim
                    } else {
                        egui::Stroke::NONE
                    };

                    painter.vline(rect.right(), rect.y_range(), stroke);
                }
                w_state.store(ui.ctx(), w_id);
            } // ui.painter_at(ui.min_rect()).debug_rect(ui.min_rect(), egui::Color32::GREEN, "text");
        });
    let mut add_contents = |ui: &mut egui::Ui, col_range: Range<usize>| {
        let min = col_range.start;

        ui.push_id(ui.id().with("aaa"), |ui| {
            wasm_rs_dbg::dbg!(&col_range, &long_tracking.results);
            #[derive(Default)]
            struct AAA<'a> {
                effective_target: Option<&'a mut CodeRange>,
                original_target: Option<&'a mut CodeRange>,
                matched: Option<&'a mut CodeRange>,
            }
            // wasm_rs_dbg::dbg!(&long_tracking.results);
            for col in col_range {
                let is_origin = |col| col == long_tracking.target_index;
                let has_past = |col| col != 0;
                let has_future = |col| col + 1 < total_cols;

                // let mut cond_path;

                let mut curr_view: AAA<'_> = AAA::default();

                if is_origin(col) {
                    curr_view.original_target = Some(&mut long_tracking.target);
                    match (has_past(col), has_future(col)) {
                        (true, true) => {
                            curr_view.effective_target = long_tracking
                                .results
                                .get_mut(col - 1)
                                .and_then(|x| x.1.get_mut())
                                .and_then(|x| x.as_mut().ok())
                                .map(|x| &mut x.src);
                        }
                        (true, false) => {
                            curr_view.effective_target = long_tracking
                                .results
                                .get_mut(col - 1)
                                .and_then(|x| x.1.get_mut())
                                .and_then(|x| x.as_mut().ok())
                                .map(|x| &mut x.src);
                        }
                        (false, true) => todo!(),
                        (false, false) => {
                            // nothing to do
                        }
                    }
                // } else if is_origin(col + 1) {
                //     if has_past(col) {
                //         let mut it = long_tracking.results.range_mut(col - 1..=col);
                //         curr_view.effective_target = it
                //             .next()
                //             .and_then(|x| x.1.get_mut())
                //             .and_then(|x| x.as_mut().ok())
                //             .map(|x| &mut x.src);
                //         curr_view.matched = it
                //             .next()
                //             .and_then(|x| x.1.get_mut())
                //             .and_then(|x| x.as_mut().ok())
                //             .map(|x| &mut x.matched[0]);
                //     } else {
                //         curr_view.matched = long_tracking
                //             .results
                //             .get_mut(col)
                //             .and_then(|x| x.1.get_mut())
                //             .and_then(|x| x.as_mut().ok())
                //             .map(|x| &mut x.matched[0]);
                //     }
                } else {
                    if has_past(col) {
                        let mut it = long_tracking.results.range_mut(col - 1..=col);
                        let past = it.next();
                        curr_view.effective_target = past
                            .and_then(|x| x.1.get_mut())
                            .and_then(|x| x.as_mut().ok())
                            .map(|x| &mut x.src);
                        let curr = it.next();
                        curr_view.matched = curr
                            .and_then(|x| x.1.get_mut())
                            .and_then(|x| x.as_mut().ok())
                            .map(|x| &mut x.matched[0]);
                        assert!(it.next().is_none());
                    } else {
                        curr_view.matched = long_tracking
                            .results
                            .get_mut(col)
                            .and_then(|x| x.1.get_mut())
                            .and_then(|x| x.as_mut().ok())
                            .map(|x| &mut x.matched[0]);
                    }
                }
                let curr = if curr_view.matched.is_some() {
                    &mut curr_view.matched
                } else {
                    &mut curr_view.original_target
                };
                let Some(curr) = curr else {
                    continue;
                };
                let file_result = fetched_files.entry(curr.file.clone());
                let x_range = ui.available_rect_before_wrap().x_range();
                let x_start = *x_range.start() + col_width_with_spacing * (col - min) as f32;
                let x_end = x_start + col_width;
                let rect = egui::Rect::from_x_y_ranges(x_start..=x_end, ui.max_rect().y_range());
                let mut ui = &mut ui.child_ui_with_id_source(
                    rect,
                    egui::Layout::top_down(egui::Align::Min),
                    col,
                );
                let x_start = timeline_window.x_range().start().max(x_start - spacing.x);
                let x_end = timeline_window.x_range().end().min(x_end);
                let clip_rect =
                    egui::Rect::from_x_y_ranges(x_start..=x_end, ui.max_rect().y_range());
                ui.set_clip_rect(clip_rect);
                let te = show_remote_code2(
                    ui,
                    &mut curr.file.commit,
                    &mut curr.file.file_path,
                    file_result,
                    col_width,
                    true,
                )
                .2;
                if let Some(egui::InnerResponse {
                    inner: Some(aa), ..
                }) = te
                {
                    // ui.painter().debug_rect(
                    //     ui.max_rect(),
                    //     egui::Color32::RED,
                    //     format!("{:?}", curr.range),
                    // );
                    if let Some(range) = curr_view
                        .original_target
                        .as_ref()
                        .and_then(|x| x.range.as_ref())
                    {
                        let color = egui::Color32::RED.linear_multiply(0.1);
                        let rect = highlight_byte_range(ui, &aa, range, color);
                        // if result_changed {
                        //     wasm_rs_dbg::dbg!(
                        //         aa.content_size,
                        //         aa.state.offset.y,
                        //         aa.inner_rect.height(),
                        //         rect.top(),
                        //     );
                        //     pos_ratio = Some((rect.top() - aa.state.offset.y) / aa.inner_rect.height());
                        //     wasm_rs_dbg::dbg!(pos_ratio);
                        // }
                    }
                    if let Some(
                        CodeRange {
                            range: Some(range), ..
                        },
                        ..,
                    ) = &curr_view.effective_target
                    {
                        let color = egui::Color32::BLUE.linear_multiply(0.1);
                        let rect = highlight_byte_range(ui, &aa, range, color);
                        // if result_changed {
                        //     wasm_rs_dbg::dbg!(
                        //         aa.content_size,
                        //         aa.state.offset.y,
                        //         aa.inner_rect.height(),
                        //         rect.top(),
                        //     );
                        //     pos_ratio = Some((rect.top() - aa.state.offset.y) / aa.inner_rect.height());
                        //     wasm_rs_dbg::dbg!(pos_ratio);
                        // }
                    }
                    if let Some(
                        CodeRange {
                            range: Some(range), ..
                        },
                        ..,
                    ) = &curr_view.matched
                    {
                        let color = egui::Color32::GREEN.linear_multiply(0.1);
                        let rect = highlight_byte_range(ui, &aa, range, color);
                        // if result_changed {
                        //     wasm_rs_dbg::dbg!(
                        //         aa.content_size,
                        //         aa.state.offset.y,
                        //         aa.inner_rect.height(),
                        //         rect.top(),
                        //     );
                        //     pos_ratio = Some((rect.top() - aa.state.offset.y) / aa.inner_rect.height());
                        //     wasm_rs_dbg::dbg!(pos_ratio);
                        // }
                    }
                    let curr = if curr_view.matched.is_some() {
                        &mut curr_view.matched
                    } else {
                        &mut curr_view.original_target
                    };
                    let Some(curr) = curr else {
                        continue
                    };
                    if !aa.inner.response.is_pointer_button_down_on() {
                        let bb = &aa.inner.cursor_range;
                        if let Some(bb) = bb {
                            let s = aa.inner.galley.text();
                            let r = bb.as_sorted_char_range();
                            let r = Range {
                                start: byte_index_from_char_index(s, r.start),
                                end: byte_index_from_char_index(s, r.end),
                            };
                            if curr.range != Some(r.clone()) {
                                // wasm_rs_dbg::dbg!(&r);
                                if is_origin(col) {
                                    curr.range = Some(r.clone());
                                }
                                if col != 0 {
                                    // TODO allow to reset tracking
                                    continue;
                                }
                                let waiting = track(
                                    ui.ctx(),
                                    &curr.file.commit,
                                    &curr.file.file_path,
                                    &Some(r),
                                );
                                let res = if col == 0 {
                                    long_tracking.results.push_front(Default::default());
                                    long_tracking.target_index += 1;
                                    &mut long_tracking.results[col].1
                                } else {
                                    // TODO allow to reset tracking
                                    continue;
                                };
                                res.buffer(waiting);
                            }
                        }
                    }
                };
            }
        });
    };
    use egui::NumExt;
    let veiwport_left = ui.available_rect_before_wrap().left() - w_state.map_or(0.0, |x| x.offset);
    let viewport = egui::Rect::from_x_y_ranges(
        veiwport_left..=veiwport_left + viewport_width,
        ui.available_rect_before_wrap().y_range(),
    );
    let ui = &mut ui.child_ui(viewport, egui::Layout::left_to_right(egui::Align::BOTTOM));
    ui.set_clip_rect(timeline_window);

    let mut min_col = (viewport.min.x / col_width_with_spacing).floor() as usize;
    let mut max_col = (viewport.max.x / col_width_with_spacing).ceil() as usize + 1;
    if max_col > total_cols {
        let diff = max_col.saturating_sub(min_col);
        max_col = total_cols;
        min_col = total_cols.saturating_sub(diff);
    }

    let x_min = ui.max_rect().left() + min_col as f32 * col_width_with_spacing;
    let x_max = ui.max_rect().left() + max_col as f32 * col_width_with_spacing;
    let rect = egui::Rect::from_x_y_ranges(
        x_min..=x_max,
        *ui.max_rect().y_range().start()..=ui.max_rect().y_range().end() * 1.0,
    );
    let mut cui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::BOTTOM));
    cui.skip_ahead_auto_ids(min_col);
    (add_contents)(&mut cui, min_col..max_col);
    // ui.allocate_ui_at_rect(rect, |viewport_ui| {
    //     viewport_ui.skip_ahead_auto_ids(min_col); // Make sure we get consistent IDs.
    //     (add_contents)(viewport_ui, min_col..max_col)
    // })
    // .inner
}

pub(super) fn track(
    ctx: &egui::Context,
    commit: &Commit,
    file_path: &String,
    range: &Option<Range<usize>>,
) -> Promise<ehttp::Result<TrackingResult>> {
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
        let resource = response
            .and_then(|response| Resource::<TrackingResult>::from_response(&ctx, response))
            .and_then(|x| x.content.ok_or("Empty body".into()));
        sender.send(resource);
    });
    promise
}
