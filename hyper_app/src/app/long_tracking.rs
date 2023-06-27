use std::{
    collections::{HashMap, VecDeque},
    ops::Range,
    sync::Arc,
};

use egui_addon::{
    code_editor::generic_text_buffer::byte_index_from_char_index,
    egui_utils::{highlight_byte_range, radio_collapsing, show_wip},
    meta_edge::meta_egde,
    multi_split::multi_splitter::MultiSplitter,
};
use epaint::{ahash::HashSet, Pos2};
use hyper_ast::{
    store::nodes::fetched::NodeIdentifier,
    types::{HyperType, Labeled, TypeStore},
};
use poll_promise::Promise;

use crate::app::{
    code_aspects::{self, HightLightHandle},
    code_tracking::TrackingResultWithChanges,
    commit::fetch_commit,
    show_remote_code1, tree_view,
    types::Resource,
    API_URL,
};

use super::{
    code_aspects::FetchedView,
    code_tracking::{self, RemoteFile, TrackingResult},
    commit::CommitMetadata,
    show_commit_menu,
    tree_view::FetchedHyperAST,
    types::{self, CodeRange, Commit, ComputeConfigAspectViews},
    AccumulableResult, Buffered, MultiBuffered,
};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct DetatchedViewOptions {
    pub(crate) bezier: bool,
    pub(crate) meta: bool,
    pub(crate) three: bool,
    pub(crate) cable: bool,
}
impl Default for DetatchedViewOptions {
    fn default() -> Self {
        Self {
            bezier: false,
            meta: true,
            three: false,
            cable: false,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub(crate) struct LongTacking {
    pub(crate) flags: Flags,
    pub(crate) ser_view: bool,
    pub(crate) tree_view: bool,
    pub(crate) detatched_view: bool,
    pub(crate) detatched_view_options: DetatchedViewOptions,
    pub(crate) origins: Vec<CodeRange>,
    pub(crate) origin_index: usize,
    #[serde(skip)] // TODO remove that
    pub(crate) results: VecDeque<(
        Buffered<Result<CommitMetadata, String>>,
        MultiBuffered<
            AccumulableResult<code_tracking::TrackingResultsWithChanges, Vec<String>>,
            Result<code_tracking::TrackingResultWithChanges, String>,
        >,
    )>,
    #[serde(skip)]
    pub(crate) tree_viewer: HashMap<Commit, Buffered<Result<Resource<FetchedView>, String>>>,
    #[serde(skip)]
    pub(crate) additionnal_links: Vec<[CodeRange; 2]>,
}

impl Default for LongTacking {
    fn default() -> Self {
        Self {
            flags: Default::default(),
            ser_view: false,
            tree_view: true,
            detatched_view: false,
            detatched_view_options: Default::default(),
            origins: vec![Default::default()],
            origin_index: Default::default(),
            results: VecDeque::from(vec![Default::default()]),
            tree_viewer: Default::default(),
            additionnal_links: Default::default(),
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize, Default)]
#[serde(default)]
pub(crate) struct Flags {
    pub(crate) upd: bool,
    pub(crate) child: bool,
    pub(crate) parent: bool,
    pub(crate) exact_child: bool,
    pub(crate) exact_parent: bool,
    pub(crate) sim_child: bool,
    pub(crate) sim_parent: bool,
    pub(crate) meth: bool,
    pub(crate) typ: bool,
    pub(crate) top: bool,
    pub(crate) file: bool,
    pub(crate) pack: bool,
    pub(crate) dependency: bool,
    pub(crate) dependent: bool,
    pub(crate) references: bool,
    pub(crate) declaration: bool,
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
        show_commit_menu(ui, &mut tracking.origins[0].file.commit);
        // let repo_changed = show_repo_menu(ui, &mut tracking.origins[0].file.commit.repo);
        // let old = tracking.origins[0].file.commit.id.clone();
        // let commit_te = egui::TextEdit::singleline(&mut tracking.origins[0].file.commit.id)
        //     .clip_text(true)
        //     .desired_width(150.0)
        //     .desired_rows(1)
        //     .hint_text("commit")
        //     .id(ui.id().with("commit"))
        //     .interactive(true)
        //     .show(ui);
        // if repo_changed || commit_te.response.changed() {
        //     // todo!()
        // } else {
        //     assert_eq!(old, tracking.origins[0].file.commit.id.clone());
        // };
        ui.checkbox(&mut tracking.tree_view, "tree view");
        ui.checkbox(&mut tracking.ser_view, "serialized view");
        ui.checkbox(&mut tracking.detatched_view, "detatched view");
        if tracking.detatched_view {
            ui.indent("detached_options", |ui| {
                ui.checkbox(&mut tracking.detatched_view_options.bezier, "bezier");
                ui.checkbox(&mut tracking.detatched_view_options.meta, "meta");
                ui.checkbox(&mut tracking.detatched_view_options.three, "three");
                ui.checkbox(&mut tracking.detatched_view_options.cable, "cable");
            });
        }
        ui.add(egui::Label::new(
            egui::RichText::from("Triggers").font(egui::FontId::proportional(16.0)),
        ));
        let flags = &mut tracking.flags;
        ui.checkbox(&mut flags.upd, "updated");
        ui.checkbox(&mut flags.child, "children changed");
        ui.checkbox(&mut flags.parent, "parent changed");

        ui.add_enabled_ui(false, |ui| {
            ui.checkbox(&mut flags.exact_child, "children changed formatting");
            ui.checkbox(&mut flags.exact_parent, "parent changed formatting");
            ui.checkbox(&mut flags.sim_child, "children changed structure");
            ui.checkbox(&mut flags.sim_parent, "parent changed structure");
            ui.checkbox(&mut flags.meth, "method changed");
            ui.checkbox(&mut flags.typ, "type changed");
            ui.checkbox(&mut flags.top, "top-lvl type changed");
            ui.checkbox(&mut flags.file, "file changed");
            ui.checkbox(&mut flags.pack, "package changed");
            ui.checkbox(&mut flags.dependency, "dependency changed");
            ui.checkbox(&mut flags.dependent, "dependent changed");
            ui.checkbox(&mut flags.references, "references changed");
            ui.checkbox(&mut flags.declaration, "declaration changed");
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

    radio_collapsing(ui, id, title, selected, &wanted, add_body);
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
type PortId = egui::Id;
type Attacheds = Vec<(
    HashMap<usize, (PortId, Option<egui::Rect>)>,
    HashMap<usize, (PortId, Option<egui::Rect>)>,
)>;
// type Detacheds = Vec<(
//     HashMap<usize, (egui::Id, Option<egui::Pos2>)>,
//     HashMap<usize, (egui::Id, Option<egui::Pos2>)>,
// )>;

pub(crate) fn show_results(
    ui: &mut egui::Ui,
    aspects: &mut types::ComputeConfigAspectViews,
    store: Arc<FetchedHyperAST>,
    long_tracking: &mut LongTacking,
    fetched_files: &mut HashMap<types::FileIdentifier, RemoteFile>,
) {
    let w_id = ui.id().with("Tracking Timeline");
    let timeline_window = ui.available_rect_before_wrap();
    let spacing: egui::Vec2 = (0.0, 0.0).into(); //4.0 // ui.spacing().item_spacing;
                                                 // let spacing: egui::Vec2 = (4.0, 4.0).into(); //4.0 // ui.spacing().item_spacing;
                                                 // let spacing: egui::Vec2 = (30.0, 4.0).into(); //4.0 // ui.spacing().item_spacing;
    let mut w_state = State::load(ui.ctx(), w_id);
    let (total_cols, col_width) = if long_tracking.results.len() <= 2 {
        (
            long_tracking.results.len(),
            ui.available_width() / long_tracking.results.len() as f32,
        )
    } else {
        let width = if let Some(w_state) = w_state {
            w_state.width
        } else {
            w_state = Some(State {
                offset: 0.0,
                width: timeline_window.width() * 0.4,
            });
            timeline_window.width() * 0.4
        };
        (long_tracking.results.len(), width)
    };
    let col_width_with_spacing = col_width + spacing.x;
    let viewport_width = (col_width_with_spacing * total_cols as f32 - spacing.x).at_least(0.0);

    let timeline_window_width = timeline_window.width();
    let veiwport_left = timeline_window.left() - w_state.map_or(0.0, |x| x.offset);
    let viewport_x = veiwport_left..=veiwport_left + viewport_width;

    let mut min_col = (viewport_x.start() / col_width_with_spacing).floor() as usize;
    let saedfgwsef = w_state.map_or(0.0, |x| x.offset);
    let mut min_col = (saedfgwsef / col_width_with_spacing).floor() as usize;
    let jtyjfgnb = saedfgwsef + timeline_window_width;
    let mut max_col = (jtyjfgnb / col_width_with_spacing).ceil() as usize;
    if max_col > total_cols {
        let diff = max_col.saturating_sub(min_col);
        max_col = total_cols;
        min_col = total_cols.saturating_sub(diff);
    }

    egui::panel::TopBottomPanel::bottom("Timeline Map")
        .frame(egui::Frame::side_top_panel(ui.style()).inner_margin(0.0))
        .height_range(0.0..=ui.available_height() / 3.0)
        .default_height(ui.available_height() / 5.0)
        .resizable(true)
        .show_inside(ui, |ui| {
            ui.set_clip_rect(ui.max_rect().expand2((1.0, 0.0).into()));
            let mut add_content = |ui: &mut egui::Ui, col: usize| {
                let mut aaa = (Buffered::Empty, MultiBuffered::default());
                let (md, tracking_result) = if long_tracking.results.is_empty() {
                    &mut aaa //&long_tracking.target
                } else {
                    let res = &mut long_tracking.results[col];
                    res.1.try_poll();
                    res.0.try_poll();
                    res
                };
                let tracked;
                let (code_ranges, md) = if col == long_tracking.origin_index {
                    tracked = None;
                    if let Some(md) = md.get_mut() {
                        (long_tracking.origins.iter_mut().collect::<Vec<_>>(), md)
                    } else {
                        if !md.is_waiting() {
                            let code_range = &mut long_tracking.origins[0];
                            // wasm_rs_dbg::dbg!(&code_range);
                            md.buffer(fetch_commit(ui.ctx(), &code_range.file.commit));
                        }
                        return;
                    }
                } else if let (Some(tracking_result), Some(md)) =
                    (tracking_result.get_mut(), md.get_mut())
                {
                    if tracking_result.content.track.results.is_empty() {
                        panic!("{:?}", tracking_result.errors)
                    } else {
                        let track = tracking_result.content.track.results.get(0).unwrap();

                        tracked = Some(TrackingResultWithChanges {
                            track: TrackingResult {
                                compute_time: track.compute_time.clone(),
                                commits_processed: track.commits_processed.clone(),
                                src: track.src.clone(),
                                intermediary: track.intermediary.clone(),
                                fallback: track.fallback.clone(),
                                matched: vec![],
                            },
                            src_changes: tracking_result.content.src_changes.clone(),
                            dst_changes: tracking_result.content.dst_changes.clone(),
                        });
                        let track = &mut tracking_result.content.track.results;
                        (
                            track
                                .iter_mut()
                                .map(|track| {
                                    track
                                        .matched
                                        .get_mut(0)
                                        .unwrap_or_else(|| track.fallback.as_mut().unwrap())
                                })
                                .collect(),
                            md,
                        )
                    }
                    // match tracking_result {
                    //     Ok(tracking_result) => {
                    //         tracked = Some(TrackingResultWithChanges {
                    //             track: TrackingResult {
                    //                 compute_time: tracking_result.track.compute_time.clone(),
                    //                 commits_processed: tracking_result
                    //                     .track
                    //                     .commits_processed
                    //                     .clone(),
                    //                 src: tracking_result.track.src.clone(),
                    //                 intermediary: tracking_result.track.intermediary.clone(),
                    //                 fallback: tracking_result.track.fallback.clone(),
                    //                 matched: vec![],
                    //             },
                    //             src_changes: tracking_result.src_changes.clone(),
                    //             dst_changes: tracking_result.dst_changes.clone(),
                    //         });
                    //         (
                    //             tracking_result.track.matched.get_mut(0).unwrap_or_else(|| {
                    //                 tracking_result.track.fallback.as_mut().unwrap()
                    //             }),
                    //             md,
                    //         )
                    //     }
                    //     Err(err) => panic!("{}", err),
                    // }
                } else if let Some(tracking_result) = tracking_result.get_mut() {
                    if tracking_result.content.track.results.is_empty()
                        && !tracking_result.errors.is_empty()
                    {
                        ui.colored_label(
                            ui.visuals().error_fg_color,
                            tracking_result.errors.join("\n"),
                        );
                        return;
                    } else if !md.is_waiting() {
                        // tracked = Some(TrackingResultWithChanges {
                        //     track: TrackingResult {
                        //         compute_time: tracking_result.track.compute_time.clone(),
                        //         commits_processed: tracking_result.track.commits_processed.clone(),
                        //         src: tracking_result.track.src.clone(),
                        //         intermediary: tracking_result.track.intermediary.clone(),
                        //         fallback: tracking_result.track.fallback.clone(),
                        //         matched: vec![],
                        //     },
                        //     src_changes: tracking_result.src_changes.clone(),
                        //     dst_changes: tracking_result.dst_changes.clone(),
                        // });
                        let track = tracking_result.content.track.results.get(0).unwrap();
                        if let Some(code_range) = &track.intermediary {
                            // wasm_rs_dbg::dbg!(&code_range);
                            md.buffer(fetch_commit(ui.ctx(), &code_range.file.commit));
                        } else if let Some(code_range) = track.matched.get(0) {
                            // wasm_rs_dbg::dbg!(&code_range);
                            md.buffer(fetch_commit(ui.ctx(), &code_range.file.commit));
                        } else if let Some(code_range) = &track.fallback {
                            // wasm_rs_dbg::dbg!(&code_range);
                            md.buffer(fetch_commit(ui.ctx(), &code_range.file.commit));
                        } else {
                            unreachable!("should have been matched or been given a fallback")
                        }
                        ui.spinner();
                        return;
                    } else {
                        ui.spinner();
                        return;
                    }
                } else {
                    ui.spinner();
                    return;
                };
                // ui.label(format!("{} {} {}", min_col, col, max_col));
                show_commitid_info(tracked, ui, code_ranges);
                match md {
                    Ok(md) => {
                        md.show(ui);
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
                MultiSplitter::vertical().ratios(ratios).show(ui, |uis| {
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
                let map_left = egui::remap_clamp(
                    w_state.offset,
                    0.0..=viewport_width,
                    ui.max_rect().x_range(),
                );
                let map_right = egui::remap_clamp(
                    w_state.offset + timeline_window_width,
                    0.0..=viewport_width,
                    ui.max_rect().x_range(),
                );
                let map_width = timeline_window_width / viewport_width * ui.max_rect().width();
                let rect = egui::Rect::from_x_y_ranges(
                    map_left..=map_left + map_width,
                    ui.max_rect().y_range(),
                );
                let painter = ui.painter_at(ui.max_rect());
                {
                    let map_drag = ui.interact(
                        egui::Rect::from_center_size(rect.center(), (40., 40.).into()),
                        ui.id().with("map_drag"),
                        egui::Sense::drag(),
                    );
                    let fill_color;
                    if map_drag.hovered() {
                        ui.ctx().set_cursor_icon(egui::CursorIcon::ResizeHorizontal);
                    }
                    if map_drag.dragged() {
                        let delta = map_drag.drag_delta();
                        if delta.x != 0.0 {
                            w_state.offset = (w_state.offset
                                + delta.x / ui.max_rect().width() * viewport_width)
                                .clamp(0.0, viewport_width - timeline_window.width());
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
                    painter.text(
                        rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "↔",
                        egui::FontId::monospace(50.0),
                        egui::Color32::BLACK,
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
                                let x = pointer
                                    .x
                                    .clamp(timeline_window.min.x, timeline_window.max.x);
                                let f = (rect.max.x - x).at_least(0.0); // - rect.min.x;
                                let col_ratio = ui.max_rect().width() / total_cols as f32 / f;
                                w_state.width = timeline_window.width() * col_ratio;

                                let col_width_with_spacing = w_state.width + spacing.x;
                                let viewport_width = (col_width_with_spacing * total_cols as f32
                                    - spacing.x)
                                    .at_least(0.0);
                                w_state.offset = egui::remap_clamp(
                                    map_right,
                                    ui.max_rect().x_range(),
                                    0.0..=viewport_width,
                                ) - timeline_window_width;

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
                            if is_resizing {
                                // let width = (pointer.x - second_rect.left()).abs();
                                // let width =
                                //     clamp_to_range(width, width_range.clone()).at_most(available_rect.width());
                                // second_rect.min.x = second_rect.max.x - width;
                                let x = pointer.x.clamp(ui.max_rect().min.x, ui.max_rect().max.x);
                                let f = (x - rect.min.x).at_least(0.0);
                                // ratio = (f / rect.width()).clamp(0.1, 0.9);
                                let col_ratio = ui.max_rect().width() / total_cols as f32 / f;
                                // w_state.end = f;
                                w_state.width = timeline_window.width() * col_ratio;
                                // (f / ui.max_rect().width() * viewport_width)
                                //     .clamp(col_width, viewport_width - w_state.offset);

                                let col_width_with_spacing = w_state.width + spacing.x;
                                let viewport_width = (col_width_with_spacing * total_cols as f32
                                    - spacing.x)
                                    .at_least(0.0);
                                w_state.offset = egui::remap_clamp(
                                    map_left,
                                    ui.max_rect().x_range(),
                                    0.0..=viewport_width,
                                );

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
                    } else {
                        egui::Stroke::NONE
                    };

                    painter.vline(rect.right(), rect.y_range(), stroke);
                }
                w_state.store(ui.ctx(), w_id);
            } // ui.painter_at(ui.min_rect()).debug_rect(ui.min_rect(), egui::Color32::GREEN, "text");
        });
    let is_origin = |col| col == long_tracking.origin_index;
    let mut waiting = vec![];
    let mut new_origins = vec![];
    let mut add_contents = |ui: &mut egui::Ui, col_range: Range<usize>| -> Attacheds {
        let min_col = col_range.start;
        // // wasm_rs_dbg::dbg!(&col_range, &long_tracking.results);
        let mut attacheds: Attacheds = vec![];
        let mut defered_focus_scroll = None;
        // // wasm_rs_dbg::dbg!(&long_tracking.results);
        for col in col_range {
            attacheds.push((Default::default(), Default::default()));
            let x_range = ui.available_rect_before_wrap().x_range();
            let x_start = *x_range.start() + col_width_with_spacing * (col - min_col) as f32;
            let x_end = x_start + col_width;
            let max_rect = egui::Rect::from_x_y_ranges(x_start..=x_end, ui.max_rect().y_range());
            let x_start = timeline_window.x_range().start().max(x_start - spacing.x);
            let x_end = timeline_window.x_range().end().min(x_end);
            let clip_rect = egui::Rect::from_x_y_ranges(x_start..=x_end, ui.max_rect().y_range());
            let ui = &mut egui::Ui::new(
                ui.ctx().clone(),
                ui.layer_id(),
                w_id.with(col as isize - long_tracking.origin_index as isize),
                // w_id.with(col),
                max_rect,
                clip_rect,
            );
            // let ui = &mut ui.child_ui_with_id_source(
            //     rect,
            //     egui::Layout::top_down(egui::Align::Min),
            //     col,
            //     // 10000 + col as isize - long_tracking.target_index as isize,
            // );
            // ui.set_clip_rect(clip_rect);

            let has_past = |col| col != 0;
            let has_future = |col| col + 1 < total_cols;

            // let mut cond_path;

            let mut curr_view: ColView<'_> = ColView::default();
            if is_origin(col) {
                // curr_view.original_target = Some(&mut long_tracking.target);
                match (has_past(col), has_future(col)) {
                    (true, true) => {
                        let qqq = long_tracking
                            .results
                            .get_mut(col - 1)
                            .and_then(|x| x.1.get_mut());
                        if let Some(qqq) = qqq {
                            if let Some(qqq) = &mut qqq.content.dst_changes {
                                // assert_ne!(long_tracking.origins[0].file.commit, qqq.commit);
                                // curr_view.left_commit = Some(&mut qqq.commit);
                                curr_view.additions = Some(&mut qqq.additions);
                            }
                            if let Some(qqq) = &mut qqq.content.src_changes {
                                assert_ne!(long_tracking.origins[0].file.commit, qqq.commit);
                                curr_view.left_commit = Some(&mut qqq.commit);
                            }
                            let mut origins = long_tracking.origins.iter_mut();
                            for (i, qqq) in qqq.content.track.results.iter_mut().enumerate() {
                                curr_view.effective_targets.push((&mut qqq.src, i));
                                curr_view
                                    .effective_targets
                                    .push((origins.next().unwrap(), i));
                            }
                        } else {
                            curr_view
                                .original_targets
                                .push((&mut long_tracking.origins[0], 0));
                        }
                    }
                    (true, false) => {
                        let qqq = long_tracking
                            .results
                            .get_mut(col - 1)
                            .and_then(|x| x.1.get_mut());
                        // curr_view.effective_target = long_tracking
                        //     .results
                        //     .get_mut(col - 1)
                        //     .and_then(|x| x.1.get_mut())
                        //     .and_then(|x| x.as_mut().ok())
                        //     .map(|x| &mut x.track.src);
                        if let Some(qqq) = qqq {
                            if let Some(ppp) = &mut qqq.content.dst_changes {
                                // assert_ne!(long_tracking.origins[0].file.commit, ppp.commit);
                                // curr_view.left_commit = Some(&mut ppp.commit);
                                curr_view.additions = Some(&mut ppp.additions);
                            }
                            if let Some(qqq) = &mut qqq.content.src_changes {
                                assert_ne!(long_tracking.origins[0].file.commit, qqq.commit);
                                curr_view.left_commit = Some(&mut qqq.commit);
                            }
                            // // wasm_rs_dbg::dbg!(
                            //     total_cols,
                            //     col,
                            //     long_tracking.origins.len(),
                            //     qqq.content.track.results.len()
                            // );
                            let mut origins = long_tracking.origins.iter_mut();
                            for (i, qqq) in qqq.content.track.results.iter_mut().enumerate() {
                                curr_view.effective_targets.push((&mut qqq.src, i));
                                if let Some(origins) = origins.next() {
                                    curr_view.original_targets.push((origins, i));
                                }
                            }
                        } else {
                            curr_view
                                .original_targets
                                .push((&mut long_tracking.origins[0], 0));
                        }
                    }
                    (false, true) => todo!(),
                    (false, false) => {
                        // nothing to do
                        curr_view
                            .original_targets
                            .push((&mut long_tracking.origins[0], 0));
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

                    if let Some(qqq) = past.and_then(|x| x.1.get_mut()) {
                        if let Some(qqq) = &mut qqq.content.dst_changes {
                            curr_view.additions = Some(&mut qqq.additions);
                            // curr_view.left_commit = Some(&mut qqq.commit);
                        }
                        if let Some(ppp) = &mut qqq.content.src_changes {
                            curr_view.left_commit = Some(&mut ppp.commit);
                        }
                        for (i, qqq) in qqq.content.track.results.iter_mut().enumerate() {
                            assert_ne!(
                                qqq.src.file.commit,
                                **curr_view.left_commit.as_ref().unwrap()
                            );
                            curr_view.effective_targets.push((&mut qqq.src, i));
                        }
                    }
                    let curr = it.next();
                    if let Some(qqq) = curr.and_then(|x| x.1.get_mut()) {
                        for (i, qqq) in qqq.content.track.results.iter_mut().enumerate() {
                            curr_view.matcheds.push((
                                qqq.matched.get_mut(0).or(qqq.fallback.as_mut()).unwrap(),
                                i,
                            ));
                        }
                        if let Some(qqq) = &mut qqq.content.src_changes {
                            curr_view.deletions = Some(&mut qqq.deletions);
                        }
                    }
                    if curr_view.original_targets.is_empty() {
                        curr_view
                            .original_targets
                            .push((&mut long_tracking.origins[0], 0));
                    }
                    assert!(it.next().is_none());
                } else {
                    let result = long_tracking
                        .results
                        .get_mut(col)
                        .and_then(|x| x.1.get_mut());
                    match result {
                        Some(qqq) => {
                            if qqq.content.track.results.is_empty() {
                                ui.colored_label(
                                    ui.visuals().error_fg_color,
                                    qqq.errors.join("\n"),
                                );
                                continue;
                            }
                            if let Some(qqq) = &mut qqq.content.src_changes {
                                curr_view.deletions = Some(&mut qqq.deletions);
                            }
                            for (i, qqq) in qqq.content.track.results.iter_mut().enumerate() {
                                curr_view.matcheds.push((
                                    qqq.matched.get_mut(0).or(qqq.fallback.as_mut()).unwrap(),
                                    i,
                                ));
                            }
                        }
                        None => {
                            curr_view
                                .original_targets
                                .push((&mut long_tracking.origins[0], 0));
                        }
                    };
                }
            }
            // // wasm_rs_dbg::dbg!(col, total_cols);

            let curr_commit = {
                let curr = if curr_view.matcheds.get(0).is_some() {
                    curr_view.matcheds.get_mut(0)
                } else {
                    curr_view.original_targets.get_mut(0)
                };
                let Some((curr,_)) = curr else {
                        continue;
                    };

                &curr.file.commit
            };

            if long_tracking.tree_view {
                // let ui = &mut ui.child_ui_with_id_source(ui.max_rect(), ui.layout().clone(), col);
                let tree_viewer = long_tracking.tree_viewer.entry(curr_commit.clone());
                let tree_viewer = tree_viewer.or_insert_with(|| Buffered::default());
                let trigger = tree_viewer.try_poll();
                let Some(tree_viewer) = tree_viewer.get_mut() else {
                    if !tree_viewer.is_waiting() {

                        tree_viewer.buffer(code_aspects::remote_fetch_node(
                            ui.ctx(),
                            store.clone(),
                            &curr_commit,
                            "",
                        ));
                        // tree_viewer.buffer(code_aspects::remote_fetch_tree(
                        //     ui.ctx(),
                        //     &curr.file.commit,
                        //     "",
                        // ));
                    }
                    continue;
                };

                // ui.label(format!(
                //     "{} {} {} {} {}",
                //     col,
                //     curr_view.deletions.is_some(),
                //     curr_view.additions.is_some(),
                //     curr_view.effective_targets.len(),
                //     curr_view.matcheds.len(),
                // ));
                match tree_viewer {
                    Ok(tree_viewer) => {
                        if let Some(p) = show_tree_view(
                            ui,
                            tree_viewer,
                            &mut curr_view,
                            trigger,
                            aspects,
                            col,
                            min_col,
                            &mut attacheds,
                            &mut defered_focus_scroll,
                        ) {
                            let curr = if !curr_view.matcheds.is_empty() {
                                curr_view.matcheds.get_mut(0)
                            } else {
                                curr_view.original_targets.get_mut(0)
                            };
                            let Some((curr, _)) = curr else {
                                    panic!();
                                };
                            if is_origin(col) {
                                curr.path = p;
                                // curr.range = Some(r.clone());
                                if col == 0 {
                                    // TODO only request changes when we have none
                                    let track_at_path = track_at_path_with_changes(
                                        ui.ctx(),
                                        &curr.file.commit,
                                        &curr.path,
                                        &long_tracking.flags,
                                    );
                                    waiting.push((col, track_at_path));
                                } else {
                                    // TODO allow to reset tracking
                                    // if ! long_tracking.origins.contains(x) {
                                    new_origins.push(CodeRange {
                                        file: curr.file.clone(),
                                        range: None,
                                        path: curr.path.clone(),
                                        path_ids: vec![],
                                    });
                                    // }
                                    let past_commit = &curr_view.left_commit.unwrap();
                                    assert_ne!(&curr.file.commit, *past_commit);
                                    let track_at_path = track_at_path(
                                        ui.ctx(),
                                        &curr.file.commit,
                                        Some(past_commit),
                                        &curr.path,
                                        &Default::default(),
                                    );
                                    waiting.push((col, track_at_path));
                                }
                            } else {
                                // panic!("{:?}",p);
                                if col == 0 {
                                    let track_at_path = track_at_path_with_changes(
                                        ui.ctx(),
                                        &curr.file.commit,
                                        &p,
                                        &long_tracking.flags,
                                    );
                                    waiting.push((col, track_at_path));
                                } else {
                                    let past_commit = &curr_view.left_commit.unwrap();
                                    let present_commit = &curr.file.commit;
                                    // TODO allow to reset tracking
                                    assert_ne!(&present_commit, past_commit);
                                    let track_at_path = track_at_path(
                                        ui.ctx(),
                                        &present_commit,
                                        Some(past_commit),
                                        &p,
                                        &Default::default(),
                                    );
                                    waiting.push((col, track_at_path));
                                }
                            }
                        }
                    }
                    Err(err) => panic!("{}", err),
                }
            } else if long_tracking.ser_view {
                if let Some(te) = show_code_view(ui, &mut curr_view, fetched_files) {
                    let offset = 0; //aa.inner.0;
                    if !te.response.is_pointer_button_down_on() {
                        let bb = &te.cursor_range;
                        if let Some(bb) = bb {
                            let s = te.galley.text();
                            let r = bb.as_sorted_char_range();
                            let r = Range {
                                start: offset + byte_index_from_char_index(s, r.start),
                                end: offset + byte_index_from_char_index(s, r.end),
                            };
                            let curr = {
                                let curr = if curr_view.matcheds.get(0).is_some() {
                                    curr_view.matcheds.get_mut(0)
                                } else {
                                    curr_view.original_targets.get_mut(0)
                                };
                                let Some((curr,_)) = curr else {
                                    continue;
                                };
                                curr
                            };
                            if curr.range != Some(r.clone()) {
                                // // wasm_rs_dbg::dbg!(&r);
                                if is_origin(col) {
                                    curr.range = Some(r.clone());
                                }
                                if col == 0 {
                                    waiting.push((
                                        col,
                                        track(
                                            ui.ctx(),
                                            &curr.file.commit,
                                            &curr.file.file_path,
                                            &Some(r),
                                            &long_tracking.flags,
                                        ),
                                    ));
                                } else {
                                    // TODO allow to reset tracking
                                    waiting.push((
                                        col,
                                        track(
                                            ui.ctx(),
                                            &curr.file.commit,
                                            &curr.file.file_path,
                                            &Some(r),
                                            &long_tracking.flags,
                                        ),
                                    ));
                                }
                            }
                        }
                    }
                }
            }
        }

        if let Some((o, i, mut scroll)) = defered_focus_scroll {
            // // wasm_rs_dbg::dbg!(&o);
            let o: f32 = o;
            // // wasm_rs_dbg::dbg!(attacheds.get(i));
            // // wasm_rs_dbg::dbg!(i);
            let g_o = attacheds
                .get(i)
                .and_then(|a| a.0.get(&0))
                .and_then(|x| x.1)
                .map(|p| p.min.y)
                .unwrap_or(timeline_window.height() / 2000.0);
            // // wasm_rs_dbg::dbg!(scroll.state.offset);
            // // wasm_rs_dbg::dbg!(g_o);
            let g_o: f32 = 50.0;
            scroll.state.offset = (0.0, (o - g_o).max(0.0)).into();
            scroll.state.store(ui.ctx(), scroll.id);
        }

        attacheds
    };
    use egui::NumExt;
    let viewport =
        egui::Rect::from_x_y_ranges(viewport_x, ui.available_rect_before_wrap().y_range());
    let ui = &mut ui.child_ui(viewport, egui::Layout::left_to_right(egui::Align::BOTTOM));
    ui.set_clip_rect(timeline_window);

    let x_min = ui.max_rect().left() + min_col as f32 * col_width_with_spacing + spacing.x / 3.0;
    let x_max =
        ui.max_rect().left() + max_col as f32 * col_width_with_spacing - spacing.x * 2.0 / 3.0;
    let rect = egui::Rect::from_x_y_ranges(x_min..=x_max, ui.max_rect().y_range());
    let mut cui = ui.child_ui(rect, egui::Layout::left_to_right(egui::Align::BOTTOM));
    // cui.skip_ahead_auto_ids(min_col);
    // cui.set_clip_rect(timeline_window);
    let attacheds = (add_contents)(&mut cui, min_col..max_col);

    long_tracking.origins.extend(new_origins);
    for (col, waiting) in waiting {
        if col == 0 {
            long_tracking.results.push_front(Default::default());
            long_tracking.origin_index += 1;
            long_tracking.results[col].1.buffer(waiting);
        } else {
            // TODO handle more than going back in time
            long_tracking.results[col - 1].1.buffer(waiting);
        }
    }
    {
        use egui_cable::prelude::*;
        ui.set_clip_rect(timeline_window);
        for i in 0..attacheds.len() - 1 {
            let (left, right) = attacheds.split_at(i + 1);
            let (greens, blues) = (&left.last().unwrap().1, &right.first().unwrap().0);
            let mut done = HashSet::default();
            let cable = false;
            let mut min_right_x = 0.0;
            let mut min_left_x = 0.0;
            let l_bound = veiwport_left + (i + 1) as f32 * col_width_with_spacing - 15.0;
            let r_bound = l_bound + 25.0;
            let mut f = |&(green, g_rect), &(blue, b_rect)| {
                if cable {
                    let green: egui::Id = green;
                    let blue: egui::Id = blue;
                    // let in_plug = Plug::to(green.clone()).default_pos(egui::Pos2::ZERO);
                    let out_plug = Plug::to(blue.clone());
                    // ui.add(Cable::new(green.with(blue), in_plug, out_plug));
                }

                if let (Some(m_rect), Some(src_rect)) = (g_rect, b_rect) {
                    // let m_center = m_rect.right_center();
                    // let src_center = src_rect.left_center();
                    // let y_bary = (m_center.y + src_center.y) / 2.0;
                    // let m_pos =
                    //     pos2(m_rect.right(), y_bary.clamp(m_rect.top(), m_rect.bottom()));
                    // let src_pos = pos2(
                    //     src_rect.left(),
                    //     y_bary.clamp(src_rect.top(), src_rect.bottom()),
                    // );
                    let m_rect: egui::Rect = m_rect;
                    let src_rect: egui::Rect = src_rect;
                    let mut m_pos = m_rect.right_center();
                    let mut src_pos = src_rect.left_center();
                    let mut ctrl = (m_pos, src_pos);
                    ctrl.0.x = l_bound;
                    ctrl.1.x = r_bound;
                    m_pos.x = m_pos.x.at_most(l_bound);
                    src_pos.x = src_pos.x.at_least(r_bound);
                    // let b_d = (m_pos.x - src_pos.x).abs();
                    // ctrl.0.x += b_d / 2.0 * 1.0;
                    // if ctrl.0.x < min_left_x {
                    //     ctrl.0.x = min_left_x;
                    // } else {
                    //     min_left_x = ctrl.0.x;
                    // }
                    // ctrl.1.x -= b_d / 10.0 * 1.0;
                    // if ctrl.1.x < min_right_x {
                    //     if src_pos.x > min_right_x {
                    //         ctrl.1.x = min_right_x;
                    //     }
                    // } else {
                    //     min_right_x = ctrl.1.x;
                    // }
                    let color = ui.style().visuals.text_color();
                    let link =
                        epaint::PathShape::line(vec![m_pos, ctrl.0, ctrl.1, src_pos], (2.0, color));
                    ui.painter().add(link);
                }
            };
            for (k, g) in greens {
                done.insert(k);
                if let Some(b) = blues.get(&k) {
                    f(g, b)
                }
            }
            for (k, b) in blues {
                if done.contains(&k) {
                    continue;
                }
                if let Some(g) = greens.get(&k) {
                    f(g, b)
                }
            }
            // if let (Some((green, Some(green_pos))), Some((blue, Some(blue_pos)))) =
            //     (ports[i].1, ports[i + 1].0)
            // {
            //     let in_plug = Plug::to(green).default_pos(egui::Pos2::ZERO);
            //     let out_plug = Plug::to(blue);

            //     ui.add(Cable::new(i, in_plug, out_plug));
            // } else {
            //     // ui.add(Cable::new(i, Plug::unplugged(), Plug::unplugged()));
            // }
        }
    }

    if long_tracking.detatched_view {
        let line_id: egui::Id = egui::Id::new("drag line");
        let col_width = timeline_window_width / total_cols as f32;
        let mut rendered = HashMap::<CodeRange, epaint::Rect>::default();
        let mut hovered_fut = None;
        let mut released_past = None;
        for (col, (_, res)) in long_tracking.results.iter_mut().enumerate() {
            let default_x = timeline_window.left() + col as f32 * col_width;
            // ui.add(egui_cable::prelude::Port::new(id));
            res.try_poll();
            if let Some(res) = res.get_mut() {
                let res = &mut res.content.track.results;
                for (i, r) in res.iter_mut().enumerate() {
                    let src = &mut r.src;
                    let src_id = ui.id().with(&src);
                    let src_rect = {
                        let default_pos = (default_x + col_width / 2.0, i as f32 * 50.0);
                        let resp = show_detached_element(
                            ui,
                            &store,
                            &long_tracking.detatched_view_options,
                            src,
                            src_id,
                            default_pos,
                        );
                        if let Some(_) = resp.inner.2 {
                            hovered_fut = Some(src.clone());
                        }
                        if let Some(past) = resp.inner.1 {
                            // wasm_rs_dbg::dbg!(&past);
                            if past.double_clicked() {
                                // wasm_rs_dbg::dbg!(&past);
                            } else {
                                if past.is_pointer_button_down_on() {
                                    let id = src_id;
                                    ui.memory_mut(|mem| {
                                        if let Some(i) = mem.data.get_temp(line_id) {
                                            if id.with("past_interact") != i {
                                                panic!();
                                            }
                                        } else {
                                            mem.data.insert_temp(line_id, id.with("past_interact"));
                                        }
                                        mem.set_dragged_id(line_id);
                                    });
                                }
                            }
                        }
                        {
                            let is_dragged = ui.memory(|mem| mem.is_being_dragged(line_id));
                            // wasm_rs_dbg::dbg!(&is_dragged);
                            if is_dragged {
                                let state =
                                    ui.memory_mut(|mem| mem.data.get_temp::<(Pos2, Pos2)>(line_id));
                                // wasm_rs_dbg::dbg!(&state, ui.ctx().pointer_latest_pos());
                                let state = if let Some(mut state) = state {
                                    if let Some(pos) = ui.ctx().pointer_latest_pos() {
                                        state.1 = pos;
                                    }
                                    Some(state)
                                } else {
                                    ui.ctx().pointer_latest_pos().map(|x| (x, x))
                                };
                                // wasm_rs_dbg::dbg!(&state);
                                if let Some(state) = state {
                                    ui.painter().line_segment(
                                        [state.0, state.1],
                                        (2.0, egui::Color32::BLUE),
                                    );
                                    // ui.input(|i| {
                                    //     i.pointer.any_pressed() && i.pointer.any_down()
                                    // });

                                    ui.memory_mut(|mem| {
                                        mem.data.insert_temp::<(Pos2, Pos2)>(line_id, state)
                                    });
                                }
                            } else if ui.memory_mut(|mem| {
                                mem.data.get_temp(line_id) == Some(src_id.with("past_interact"))
                            }) {
                                let Some(mut state) =
                                ui.memory_mut(|mem| mem.data.get_temp::<(Pos2, Pos2)>(line_id)) else {
                                    panic!()
                                };
                                // wasm_rs_dbg::dbg!(&state);
                                if let Some(pos) = ui.ctx().pointer_latest_pos() {
                                    state.1 = pos;
                                }
                                ui.painter()
                                    .line_segment([state.0, state.1], (2.0, egui::Color32::BLUE));
                                released_past = Some(src.clone());
                                // if let Some(hovered_src_fut) = hovered_src_fut {
                                //     panic!();
                                // }
                                ui.memory_mut(|mem| {
                                    mem.data.remove::<(Pos2, Pos2)>(line_id);
                                    mem.data.remove::<egui::Id>(line_id)
                                });
                            }
                        }
                        rendered.insert(src.clone(), resp.inner.0.rect);
                        resp.inner.0.rect
                    };
                    for m in &mut r.matched {
                        let id = ui.id().with(&m);
                        let m_rect = if let Some(m_pos) = rendered.get(&m) {
                            m_pos.clone()
                        } else {
                            let default_pos = (default_x, i as f32 * 50.0);
                            let resp = show_detached_element(
                                ui,
                                &store,
                                &long_tracking.detatched_view_options,
                                &m,
                                id,
                                default_pos,
                            );
                            rendered.insert(m.clone(), resp.inner.0.rect);
                            if let Some(_) = resp.inner.2 {
                                hovered_fut = Some(m.clone());
                            }
                            if let Some(past) = resp.inner.1 {
                                // wasm_rs_dbg::dbg!(&past);
                                if past.double_clicked() {
                                    // wasm_rs_dbg::dbg!(&past);
                                } else {
                                    if past.is_pointer_button_down_on() {
                                        ui.memory_mut(|mem| {
                                            if let Some(i) = mem.data.get_temp(line_id) {
                                                if id.with("past_interact") != i {
                                                    panic!();
                                                }
                                            } else {
                                                mem.data
                                                    .insert_temp(line_id, id.with("past_interact"));
                                            }
                                            mem.set_dragged_id(line_id);
                                        });
                                    }
                                }
                            }
                            {
                                let is_dragged = ui.memory(|mem| mem.is_being_dragged(line_id));
                                // wasm_rs_dbg::dbg!(&is_dragged);
                                if is_dragged {
                                    let state = ui.memory_mut(|mem| {
                                        mem.data.get_temp::<(Pos2, Pos2)>(line_id)
                                    });
                                    // wasm_rs_dbg::dbg!(&state, ui.ctx().pointer_latest_pos());
                                    let state = if let Some(mut state) = state {
                                        if let Some(pos) = ui.ctx().pointer_latest_pos() {
                                            state.1 = pos;
                                        }
                                        Some(state)
                                    } else {
                                        ui.ctx().pointer_latest_pos().map(|x| (x, x))
                                    };
                                    // wasm_rs_dbg::dbg!(&state);
                                    if let Some(state) = state {
                                        ui.painter().line_segment(
                                            [state.0, state.1],
                                            (2.0, egui::Color32::BLUE),
                                        );
                                        // ui.input(|i| {
                                        //     i.pointer.any_pressed() && i.pointer.any_down()
                                        // });

                                        ui.memory_mut(|mem| {
                                            mem.data.insert_temp::<(Pos2, Pos2)>(line_id, state)
                                        });
                                    }
                                } else if ui.memory_mut(|mem| {
                                    mem.data.get_temp(line_id) == Some(id.with("past_interact"))
                                }) {
                                    let Some(mut state) =
                                    ui.memory_mut(|mem| mem.data.get_temp::<(Pos2, Pos2)>(line_id)) else {
                                        panic!()
                                    };
                                    // wasm_rs_dbg::dbg!(&state);
                                    if let Some(pos) = ui.ctx().pointer_latest_pos() {
                                        state.1 = pos;
                                    }
                                    ui.painter().line_segment(
                                        [state.0, state.1],
                                        (2.0, egui::Color32::BLUE),
                                    );
                                    released_past = Some(m.clone());
                                    ui.memory_mut(|mem| {
                                        mem.data.remove::<(Pos2, Pos2)>(line_id);
                                        mem.data.remove::<egui::Id>(line_id)
                                    });
                                }
                            }
                            resp.inner.0.rect
                        };
                        if long_tracking.detatched_view_options.cable {
                            let in_id = src_id.with("right");
                            // let in_plug = Plug::to(in_id).default_pos(egui::Pos2::ZERO);
                            let out_id = id.with("left");
                            // let out_plug = Plug::to(out_id).default_pos(egui::Pos2::ZERO);
                            // ui.add(Cable::new(in_id.with(out_id), in_plug, out_plug));
                        }
                        let m_pos = m_rect.center();
                        let src_pos = src_rect.center();
                        let x = m_pos.x - src_pos.x + 350.0;
                        let x = if x < 0.0 {
                            -(m_pos.x - src_pos.x).abs() * 100.0 / x
                        } else {
                            2.0 * x
                        };
                        let y = ((m_pos.y - src_pos.y) / 2.0).clamp(-100.0, 100.0);
                        let b_d = m_rect.right() - src_rect.left();
                        let mut color = egui::Color32::RED;
                        let mut ctrl = (m_pos, src_pos);
                        // egui::Color32::BLACK
                        use std::f32::consts::TAU;
                        let center_v = m_rect.center() - src_rect.center();
                        let angle =
                            ((center_v) * epaint::vec2(0.5, 1.0)).normalized().angle() / TAU + 0.5
                                - 0.125;
                        // // wasm_rs_dbg::dbg!(angle);
                        if (0.03..0.25).contains(&angle) {
                            // ctrl.0.x += m_rect.width()/2.0 + 150.0;//-b_d / 3.0;
                            // ctrl.1.x -= src_rect.width()/2.0 + 150.0;//-b_d / 3.0;
                            // let b_d = b_d + 100.0;
                            // ctrl.0.x += m_rect.width()/2.0 + b_d.abs() * 50.0 / (center_v.y.abs() + 1.0);
                            // ctrl.1.x -= src_rect.width()/2.0 + b_d.abs() * 50.0 / (center_v.y.abs() + 1.0);
                            // ctrl.0.y -= center_v.y / 2.0;
                            // ctrl.1.y += center_v.y / 2.0;
                            // let center = m_rect.center()-center_v/2.0;
                            // let link = epaint::PathShape::line(
                            //     vec![m_pos, ctrl.0, (ctrl.0.x, center.y).into(),center, (ctrl.1.x, center.y).into(),ctrl.1, src_pos],
                            //     (5.0, color),
                            // );
                            // ui.painter().add(link);
                            // color = egui::Color32::??;
                            // let link = epaint::CubicBezierShape::from_points_stroke(
                            //     [m_pos, ctrl.0, (ctrl.0.x, center.y).into(),center],
                            //     false,
                            //     egui::Color32::TRANSPARENT,
                            //     (5.0, color),
                            // );
                            // ui.painter().add(link);
                            // let link = epaint::CubicBezierShape::from_points_stroke(
                            //     [center, (ctrl.1.x, center.y).into(),ctrl.1, src_pos],
                            //     false,
                            //     egui::Color32::TRANSPARENT,
                            //     (5.0, color),
                            // );
                            // ui.painter().add(link);
                        } else if (0.25..0.5).contains(&angle) {
                            // color = egui::Color32::GREEN;
                            // let link = epaint::PathShape::line(
                            //     vec![m_pos, ctrl.0, ctrl.1, src_pos],
                            //     (5.0, color),
                            // );
                            // ui.painter().add(link);
                        } else if (0.5..0.71).contains(&angle) {
                            // color = egui::Color32::BLUE;
                        } else {
                            ctrl.0.x += m_rect.width() / 2.0 - b_d / 2.0 * 1.0;
                            ctrl.1.x -= src_rect.width() / 2.0 - b_d / 10.0 * 1.0;
                            color = egui::Color32::BLACK;
                            if long_tracking.detatched_view_options.meta {
                                meta_egde(m_pos, src_pos, m_rect, ctrl, src_rect, color, ui);
                            }
                            if long_tracking.detatched_view_options.bezier {
                                let link = epaint::CubicBezierShape::from_points_stroke(
                                    [m_pos, ctrl.0, ctrl.1, src_pos],
                                    false,
                                    egui::Color32::TRANSPARENT,
                                    (5.0, color),
                                );
                                let tolerance = (m_pos.x - src_pos.x).abs() * 0.01;
                                ui.painter().extend(
                                    link.to_path_shapes(Some(tolerance), None)
                                        .into_iter()
                                        .map(|x| epaint::Shape::Path(x)),
                                );
                            }
                            if long_tracking.detatched_view_options.three {
                                let link = epaint::PathShape::line(
                                    vec![m_pos, ctrl.0, ctrl.1, src_pos],
                                    (1.0, color),
                                );
                                ui.painter().add(link);
                            }
                            continue;
                        }
                        let link = epaint::PathShape::line(vec![m_pos, src_pos], (1.0, color));
                        ui.painter().add(link);
                    }
                }
            }
            // for (x, i) in curr_view.effective_targets.iter() {
            //     let id = ui.id().with("target").with(x);
            //     let default_pos = (default_x, *i as f32 * 10.0);
            //     show_detached_element(ui, x, id, default_pos);
            // }
            // for (x, i) in curr_view.matcheds.iter() {
            //     let id = ui.id().with("matched").with(x);
            //     let default_pos = (default_x, *i as f32 * 10.0);
            //     show_detached_element(ui, x, id, default_pos);
            // }
        }
        if let (Some(hovered_fut), Some(released_past)) = (hovered_fut, released_past) {
            long_tracking
                .additionnal_links
                .push([hovered_fut, released_past]);
        }
        for [m, src] in &long_tracking.additionnal_links {
            use egui_cable::prelude::*;
            let m_rect = *rendered.get(m).unwrap();
            let src_rect = *rendered.get(src).unwrap();
            let m_pos = m_rect.center();
            let src_pos = src_rect.center();
            let x = m_pos.x - src_pos.x + 350.0;
            let x = if x < 0.0 {
                -(m_pos.x - src_pos.x).abs() * 100.0 / x
            } else {
                2.0 * x
            };
            let y = ((m_pos.y - src_pos.y) / 2.0).clamp(-100.0, 100.0);
            let b_d = m_rect.right() - src_rect.left();
            let mut color = egui::Color32::RED;
            let mut ctrl = (m_pos, src_pos);
            use std::f32::consts::TAU;
            let center_v = m_rect.center() - src_rect.center();
            let angle =
                ((center_v) * epaint::vec2(0.5, 1.0)).normalized().angle() / TAU + 0.5 - 0.125;
            if (0.03..0.25).contains(&angle) {
            } else if (0.25..0.5).contains(&angle) {
            } else if (0.5..0.71).contains(&angle) {
            } else {
                ctrl.0.x += m_rect.width() / 2.0 - b_d / 2.0 * 1.0;
                ctrl.1.x -= src_rect.width() / 2.0 - b_d / 10.0 * 1.0;
                color = egui::Color32::BLACK;
                if long_tracking.detatched_view_options.meta {
                    meta_egde(m_pos, src_pos, m_rect, ctrl, src_rect, color, ui);
                }
                if long_tracking.detatched_view_options.bezier {
                    let link = epaint::CubicBezierShape::from_points_stroke(
                        [m_pos, ctrl.0, ctrl.1, src_pos],
                        false,
                        egui::Color32::TRANSPARENT,
                        (5.0, color),
                    );
                    let tolerance = (m_pos.x - src_pos.x).abs() * 0.01;
                    ui.painter().extend(
                        link.to_path_shapes(Some(tolerance), None)
                            .into_iter()
                            .map(|x| epaint::Shape::Path(x)),
                    );
                }
                if long_tracking.detatched_view_options.three {
                    let link =
                        epaint::PathShape::line(vec![m_pos, ctrl.0, ctrl.1, src_pos], (1.0, color));
                    ui.painter().add(link);
                }
                continue;
            }
            let link = epaint::PathShape::line(vec![m_pos, src_pos], (1.0, color));
            ui.painter().add(link);
        }
    }
}

struct LineDrag {
    origin_code_ele: CodeRange,
    line: (egui::Pos2, egui::Pos2),
    ori_trap: egui::Id,
}

fn color_gr_shift(color: epaint::Color32, step: f32) -> epaint::Color32 {
    egui::Color32::from_rgba_unmultiplied(
        color.r().saturating_add(step.round() as u8),
        color.g().saturating_sub(step.round() as u8),
        color.b(),
        color.a(),
    )
}

fn show_detached_element(
    ui: &mut egui::Ui,
    store: &Arc<FetchedHyperAST>,
    global_opt: &DetatchedViewOptions,
    x: &CodeRange,
    id: egui::Id,
    default_pos: (f32, f32),
) -> egui::InnerResponse<(
    egui::Response,
    Option<egui::Response>,
    Option<egui::Response>,
)> {
    let p = ui.available_rect_before_wrap().left_bottom();
    #[derive(Clone)]
    struct O {
        commit: bool,
        file: bool,
        path: bool,
        id: bool,
        kind: bool,
        label: bool,
        size: bool,
        /// search number literal
        extra: bool,
    }
    impl Default for O {
        fn default() -> Self {
            Self {
                commit: true,
                file: true,
                path: true,
                id: true,
                kind: true,
                label: true,
                size: true,
                extra: false,
            }
        }
    }
    let options = ui
        .memory_mut(|mem| mem.data.get_temp::<O>(id))
        .unwrap_or_default();
    let area = egui::Area::new(id)
        .default_pos(default_pos)
        .show(ui.ctx(), |ui| {
            let past_resp;
            let fut_resp;
            let past = ui.painter().add(egui::Shape::Noop);
            let futur = ui.painter().add(egui::Shape::Noop);
            let mut prepared = egui::Frame::window(&ui.style()).begin(ui);
            let cui = &mut prepared.content_ui;
            if options.commit {
                cui.label(format!(
                    "{}",
                    &x.file.commit.id[..x.file.commit.id.len().min(6)]
                ));
            }
            if options.file {
                if let Some(range) = &x.range {
                    cui.label(format!("{}:{:?}", x.file.file_path, range));
                } else {
                    cui.label(format!("{}", x.file.file_path));
                }
            }
            if options.path {
                cui.label(format!("{:?}", x.path));
            }
            if let Some(id) = x.path_ids.first() {
                if options.id {
                    cui.label(format!("{:?}", id));
                }
                if let Some(r) = store.node_store.read().unwrap().try_resolve(*id) {
                    use hyper_ast::types::{Tree, Typed, WithStats};
                    if options.kind {
                        let kind = store.type_store.resolve_type(&r);
                        cui.label(format!("{}", kind));
                    }
                    if options.label {
                        let l = r.try_get_label().copied();
                        if let Some(l) = l {
                            if let Some(l) = store.label_store.read().unwrap().try_resolve(&l) {
                                cui.label(format!("{:?}", l));
                            }
                        }
                    }
                    // let cs = r.children();
                    if options.size {
                        let size = r.size();
                        cui.label(format!("size: {}", size));
                    }
                    if options.extra {
                        use hyper_ast::types::{Labeled, WithChildren};
                        let mut q: VecDeque<NodeIdentifier> = Default::default();
                        if let Some(cs) = r.children() {
                            cs.0.iter().for_each(|x| q.push_back(*x));
                        }
                        let mut value = None;
                        let mut name = None;
                        while let Some(r) = q.pop_front() {
                            if value.is_some() && name.is_some() {
                                break;
                            }
                            if let Some(r) = store.node_store.read().unwrap().try_resolve(r) {
                                let t = store.type_store.resolve_type(&r);
                                // wasm_rs_dbg::dbg!(t);
                                if t.generic_eq(&hyper_ast_gen_ts_cpp::types::Type::NumberLiteral) {
                                    if value.is_none() {
                                        let l = r.get_label_unchecked();
                                        if let Some(l) =
                                            store.label_store.read().unwrap().try_resolve(&l)
                                        {
                                            value = Some(l.to_owned());
                                        } else {
                                            if !store
                                                .labels_pending
                                                .lock()
                                                .unwrap()
                                                .iter()
                                                .any(|x| x.contains(l))
                                            {
                                                store
                                                    .labels_waiting
                                                    .lock()
                                                    .unwrap()
                                                    .get_or_insert(Default::default())
                                                    .insert(*l);
                                            }
                                        }
                                    }
                                } else if t
                                    .generic_eq(&hyper_ast_gen_ts_cpp::types::Type::Identifier)
                                {
                                    if name.is_none() {
                                        let l = r.get_label_unchecked();
                                        if let Some(l) =
                                            store.label_store.read().unwrap().try_resolve(&l)
                                        {
                                            name = Some(l.to_owned());
                                        } else {
                                            if !store
                                                .labels_pending
                                                .lock()
                                                .unwrap()
                                                .iter()
                                                .any(|x| x.contains(l))
                                            {
                                                store
                                                    .labels_waiting
                                                    .lock()
                                                    .unwrap()
                                                    .get_or_insert(Default::default())
                                                    .insert(*l);
                                            }
                                        }
                                    }
                                } else if let Some(cs) = r.children() {
                                    cs.0.iter().for_each(|x| q.push_back(*x));
                                }
                            } else {
                                if !store
                                    .nodes_pending
                                    .lock()
                                    .unwrap()
                                    .iter()
                                    .any(|x| x.contains(&r))
                                {
                                    store
                                        .nodes_waiting
                                        .lock()
                                        .unwrap()
                                        .get_or_insert(Default::default())
                                        .insert(r);
                                }
                            }
                        }
                        if let Some(l) = name {
                            cui.label(format!("name: {}", l));
                        }
                        if let Some(l) = value {
                            cui.label(format!("value: {}", l));
                        }
                    }
                } else {
                    if !store
                        .nodes_pending
                        .lock()
                        .unwrap()
                        .iter()
                        .any(|x| x.contains(id))
                    {
                        store
                            .nodes_waiting
                            .lock()
                            .unwrap()
                            .get_or_insert(Default::default())
                            .insert(*id);
                    }
                }
            }
            // ui.text_edit_multiline(&mut format!("{:#?}", x));
            if global_opt.cable {
                // cui.add(egui_cable::prelude::Port::new(id.with("left")));
                // cui.add_space(10.0);
                // cui.add(egui_cable::prelude::Port::new(id.with("right")));
            }
            cui.min_rect();
            let min = cui.min_rect().min;
            let size = cui.min_rect().size();
            let s = 25.0;
            if false {
                let mut out = epaint::Mesh::default();
                let mut path = epaint::tessellator::Path::default();
                path.clear();
                let top = min;
                let mut bot = min;
                bot.y += size.y;
                path.add_line_loop(&[
                    epaint::pos2(top.x - s, top.y - s),
                    epaint::pos2(top.x, top.y),
                    epaint::pos2(bot.x, bot.y),
                    epaint::pos2(bot.x - s, bot.y + s),
                ]);
                path.fill(10.0, egui::Color32::RED, &mut out);
                ui.painter().set(past, out);
                // path.stroke_closed(self.feathering, stroke, &mut out);
                past_resp = Some(ui.interact_with_hovered(
                    egui::Rect::NOTHING,
                    false,
                    id.with("past_interact"),
                    egui::Sense::click(),
                ));
            } else {
                let mut out = epaint::Mesh::default();
                let top = min;
                let mut bot = min;
                bot.y += size.y;
                let rect = egui::Rect::from_min_max(top + (-2.0 * s, 0.0).into(), bot);
                let right_paint = |col| {
                    let transp = egui::Color32::TRANSPARENT;
                    out.colored_vertex(epaint::pos2(top.x - s, top.y - s), transp);
                    out.colored_vertex(epaint::pos2(top.x, top.y), col);
                    out.colored_vertex(epaint::pos2(bot.x, bot.y), col);
                    out.colored_vertex(epaint::pos2(bot.x - s, bot.y + s), transp);
                    out.add_triangle(0, 1, 2);
                    out.add_triangle(0, 2, 3);
                    ui.painter().set(past, out);
                };
                if ui
                    .ctx()
                    .pointer_hover_pos()
                    .map_or(false, |x| rect.contains(x))
                {
                    let resp = ui.interact_with_hovered(
                        rect,
                        true,
                        id.with("past_interact"),
                        egui::Sense::click(),
                    );
                    let col = if resp.clicked() {
                        egui::Color32::BLUE //.gamma_multiply(0.5)
                    } else {
                        egui::Color32::RED.gamma_multiply(0.5)
                    };
                    past_resp = Some(resp);

                    right_paint(col);
                } else if ui.memory_mut(|mem| {
                    mem.data.get_temp::<egui::Id>(egui::Id::new("drag line"))
                        == Some(id.with("past_interact"))
                }) {
                    right_paint(egui::Color32::BLUE);
                    past_resp = None;
                } else {
                    past_resp = None;
                }
            }

            // ui.painter().set(
            //     past,
            //     epaint::RectShape::filled(
            //         egui::Rect::from_min_max(
            //             min - epaint::Vec2::new(10.0, 10.0),
            //             (min.x, min.y + size.y + 10.0).into(),
            //         )
            //         .expand(20.0),
            //         egui::Rounding::same(1.0),
            //         egui::Color32::RED,
            //     ),
            // );
            if false {
                ui.painter().set(
                    futur,
                    epaint::RectShape::filled(
                        egui::Rect::from_min_max(
                            (min.x + size.x, min.y - 10.0).into(),
                            (min.x + size.x + 10.0, min.y + size.y + 10.0).into(),
                        )
                        .expand(20.0),
                        egui::Rounding::same(1.0),
                        egui::Color32::GREEN,
                    ),
                );
                fut_resp = None;
            } else {
                let mut out = epaint::Mesh::default();
                let mut top = min;
                top.x += size.x;
                let mut bot = top;
                bot.y += size.y;
                let rect = egui::Rect::from_min_max(top, bot + (2.0 * s, 0.0).into());
                let left_paint = |col| {
                    let transp = egui::Color32::TRANSPARENT;
                    out.colored_vertex(epaint::pos2(top.x, top.y), col);
                    out.colored_vertex(epaint::pos2(top.x + s, top.y - s), transp);
                    out.colored_vertex(epaint::pos2(bot.x + s, bot.y + s), transp);
                    out.colored_vertex(epaint::pos2(bot.x, bot.y), col);
                    out.add_triangle(0, 1, 2);
                    out.add_triangle(0, 2, 3);
                    ui.painter().set(futur, out);
                };
                if ui
                    .ctx()
                    .pointer_hover_pos()
                    .map_or(false, |x| rect.contains(x))
                {
                    let resp = ui.interact_with_hovered(
                        rect,
                        true,
                        id.with("fut_interact"),
                        egui::Sense::click(),
                    );
                    let col = if resp.clicked() {
                        egui::Color32::BLUE //.gamma_multiply(0.5)
                    } else {
                        egui::Color32::GREEN.gamma_multiply(0.5)
                    };
                    fut_resp = Some(resp);

                    left_paint(col);
                } else if ui.memory_mut(|mem| {
                    mem.data.get_temp::<egui::Id>(egui::Id::new("drag line"))
                        == Some(id.with("fut_interact"))
                }) {
                    left_paint(egui::Color32::BLUE);
                    fut_resp = None;
                } else {
                    fut_resp = None;
                }
            }
            let response = prepared.end(ui);
            (response, past_resp, fut_resp)
        });
    if area.response.hovered() {
        let options = ui.ctx().input_mut(|inp| O {
            id: options.id ^ inp.consume_key(egui::Modifiers::NONE, egui::Key::I),
            commit: options.commit ^ inp.consume_key(egui::Modifiers::NONE, egui::Key::C),
            file: options.file ^ inp.consume_key(egui::Modifiers::NONE, egui::Key::F),
            path: options.path ^ inp.consume_key(egui::Modifiers::NONE, egui::Key::P),
            kind: options.kind ^ inp.consume_key(egui::Modifiers::NONE, egui::Key::K),
            label: options.label ^ inp.consume_key(egui::Modifiers::NONE, egui::Key::L),
            size: options.size ^ inp.consume_key(egui::Modifiers::NONE, egui::Key::S),
            extra: options.extra ^ inp.consume_key(egui::Modifiers::NONE, egui::Key::Y),
        });
        ui.memory_mut(|mem| mem.data.insert_temp::<O>(id, options));
        egui::Area::new("full")
            .fixed_pos(p)
            .anchor(egui::Align2::LEFT_BOTTOM, (0.0, 0.0))
            .show(ui.ctx(), |ui| {
                let text = if let Some(range) = &x.range {
                    format!(
                        "{}/{}:{:?}",
                        &x.file.commit.id[..x.file.commit.id.len().min(6)],
                        x.file.file_path,
                        range
                    )
                } else {
                    format!(
                        "{}{}",
                        &x.file.commit.id[..x.file.commit.id.len().min(6)],
                        x.file.file_path
                    )
                };
                ui.label(egui::RichText::new(text).background_color(egui::Color32::GRAY))
            });
    }
    area
}

#[derive(Default, Debug)]
struct ColView<'a> {
    left_commit: Option<&'a mut Commit>,
    effective_targets: Vec<(&'a mut CodeRange, usize)>,
    original_targets: Vec<(&'a mut CodeRange, usize)>,
    matcheds: Vec<(&'a mut CodeRange, usize)>,
    additions: Option<&'a [u32]>,
    deletions: Option<&'a [u32]>,
}

fn show_code_view(
    ui: &mut egui::Ui,
    curr_view: &mut ColView<'_>,
    fetched_files: &mut HashMap<
        types::FileIdentifier,
        Promise<Result<Resource<code_tracking::FetchedFile>, String>>,
    >,
) -> Option<egui::text_edit::TextEditOutput> {
    let curr_file = {
        let curr = if curr_view.matcheds.get(0).is_some() {
            curr_view.matcheds.get_mut(0)
        } else {
            curr_view.original_targets.get_mut(0)
        };
        let Some((curr,_)) = curr else {
            return None;
        };

        &mut curr.file
    };

    let file_result = fetched_files.entry(curr_file.clone());
    let te = show_remote_code1(
        ui,
        &mut curr_file.commit,
        &mut curr_file.file_path,
        file_result,
        f32::INFINITY,
        false,
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
            .original_targets
            .get(0)
            .as_ref()
            .and_then(|(x, _)| x.range.as_ref())
        {
            let te = &aa.inner; //&aa.inner.1;
            let offset = 0; //aa.inner.0;
            let range = range.start.saturating_sub(offset)..range.end.saturating_sub(offset);
            let color = egui::Color32::RED.linear_multiply(0.1);
            let rect = highlight_byte_range(ui, te, &range, color);
            // if result_changed {
            //     // wasm_rs_dbg::dbg!(
            //         aa.content_size,
            //         aa.state.offset.y,
            //         aa.inner_rect.height(),
            //         rect.top(),
            //     );
            //     pos_ratio = Some((rect.top() - aa.state.offset.y) / aa.inner_rect.height());
            //     // wasm_rs_dbg::dbg!(pos_ratio);
            // }
        }
        if let Some(
            CodeRange {
                range: Some(range), ..
            },
            ..,
        ) = &curr_view.effective_targets.get(0).map(|x| &x.0)
        {
            let te = &aa.inner; //&aa.inner.1;
            let offset = 0; //aa.inner.0;
            let range = range.start.saturating_sub(offset)..range.end.saturating_sub(offset);
            let color = egui::Color32::BLUE.linear_multiply(0.1);
            // let rect = highlight_byte_range(ui, te, &range, color);
            // if result_changed {
            //     // wasm_rs_dbg::dbg!(
            //         aa.content_size,
            //         aa.state.offset.y,
            //         aa.inner_rect.height(),
            //         rect.top(),
            //     );
            //     pos_ratio = Some((rect.top() - aa.state.offset.y) / aa.inner_rect.height());
            //     // wasm_rs_dbg::dbg!(pos_ratio);
            // }
        }
        if let Some(
            CodeRange {
                range: Some(range), ..
            },
            ..,
        ) = &curr_view.matcheds.get(0).map(|x| &x.0)
        {
            let te = &aa.inner; //&aa.inner.1;
            let offset = 0; //aa.inner.0;
            let range = range.start.saturating_sub(offset)..range.end.saturating_sub(offset);
            let color = egui::Color32::GREEN.linear_multiply(0.1);
            let rect = highlight_byte_range(ui, te, &range, color);
            // if result_changed {
            //     // wasm_rs_dbg::dbg!(
            //         aa.content_size,
            //         aa.state.offset.y,
            //         aa.inner_rect.height(),
            //         rect.top(),
            //     );
            //     pos_ratio = Some((rect.top() - aa.state.offset.y) / aa.inner_rect.height());
            //     // wasm_rs_dbg::dbg!(pos_ratio);
            // }
        }

        let te = aa.inner; //&aa.inner.1;
        Some(te)
    } else {
        None
    }
}

fn show_tree_view(
    ui: &mut egui::Ui,
    tree_viewer: &mut Resource<FetchedView>,
    curr_view: &mut ColView<'_>,
    trigger: bool,
    aspects: &mut ComputeConfigAspectViews,
    col: usize,
    min_col: usize,
    ports: &mut Attacheds,
    defered_focus_scroll: &mut Option<(
        f32,
        usize,
        egui::scroll_area::ScrollAreaOutput<Option<Vec<usize>>>,
    )>,
) -> Option<Vec<usize>> {
    let mut scroll_focus = None;
    let mut scroll = egui::ScrollArea::both()
        .auto_shrink([false, false])
        .show_viewport(ui, |ui, viewport| {
            ui.set_height(3_000.0);
            ui.set_width(ui.available_width() - 15.0);
            // ui.set_clip_rect(ui.ctx().screen_rect());
            if let Some(content) = &mut tree_viewer.content {
                let mut hightlights = vec![];
                let mut focus = None;
                let mut blue_pos = HashMap::<usize, std::option::Option<egui::Rect>>::default();
                let mut green_pos = HashMap::<usize, std::option::Option<egui::Rect>>::default();
                for (_, i) in curr_view.effective_targets.iter() {
                    blue_pos.insert(*i, None);
                }
                for (i, b_p) in blue_pos.iter_mut() {
                    hightlights.push(HightLightHandle {
                        path: &curr_view
                            .effective_targets
                            .iter()
                            .find(|x| x.1 == *i)
                            .unwrap()
                            .0
                            .path[..],
                        color: &egui::Color32::BLUE,
                        id: *i,
                        screen_pos: b_p,
                    });
                }
                let a = if curr_view.matcheds.len() == 1 {
                    let Some((foc, i)) = curr_view.matcheds.get(0) else {
                        unreachable!()
                    };
                    green_pos.insert(*i, None);
                    hightlights.push(HightLightHandle {
                        path: &foc.path[..],
                        color: &TARGET_COLOR,
                        id: *i,
                        screen_pos: green_pos.get_mut(i).unwrap(),
                    });
                    if trigger {
                        let mut pi = foc.path_ids.clone();
                        pi.reverse();
                        focus = Some((&foc.path[..], &pi[..]));
                        let id = ui.id();
                        let a = content.show(
                            ui,
                            aspects,
                            focus,
                            hightlights,
                            curr_view.additions,
                            curr_view.deletions,
                            "",
                        );
                        let bool = match a {
                            tree_view::Action::Focused(_) => false,
                            tree_view::Action::PartialFocused(_) => true,
                            x => panic!("{:?}", x),
                        };
                        if bool {
                            ui.ctx().memory_mut(|mem| {
                                *mem.data.get_temp_mut_or_default::<bool>(id) = true;
                            });
                        }
                        a
                    } else {
                        let id = ui.id();
                        let bool = ui
                            .ctx()
                            .memory_mut(|mem| mem.data.get_temp::<bool>(id).unwrap_or(false));
                        let mut pi = foc.path_ids.clone();
                        pi.reverse();
                        if bool {
                            focus = Some((&foc.path[..], &pi[..]));
                        }
                        let a = content.show(
                            ui,
                            aspects,
                            focus,
                            hightlights,
                            curr_view.additions,
                            curr_view.deletions,
                            "",
                        );
                        let bool = match a {
                            tree_view::Action::Focused(_) => false,
                            tree_view::Action::PartialFocused(_) => true,
                            _ => false,
                        };
                        if !bool {
                            ui.ctx().memory_mut(|mem| {
                                mem.data.remove::<bool>(id);
                            });
                        }
                        a
                    }
                } else {
                    for (_, i) in curr_view.matcheds.iter() {
                        green_pos.insert(*i, None);
                    }
                    for (i, g_p) in green_pos.iter_mut() {
                        hightlights.push(HightLightHandle {
                            path: &curr_view
                                .matcheds
                                .iter()
                                .find(|x| x.1 == *i)
                                .unwrap()
                                .0
                                .path[..],
                            color: &TARGET_COLOR,
                            id: *i,
                            screen_pos: g_p,
                        });
                    }
                    content.show(
                        ui,
                        aspects,
                        focus,
                        hightlights,
                        curr_view.additions,
                        curr_view.deletions,
                        "",
                    )
                };
                // let a = content.show(ui, aspects, focus, hightlights, "");
                for (k, blue_pos) in blue_pos {
                    ports[col - min_col]
                        .0
                        .insert(k, (ui.id().with("blue_highlight").with(k), blue_pos));
                }
                for (k, green_pos) in green_pos {
                    ports[col - min_col]
                        .1
                        .insert(k, (ui.id().with("green_highlight").with(k), green_pos));
                }
                match a {
                    tree_view::Action::Focused(p) => {
                        scroll_focus = Some(p);
                        None
                    }
                    tree_view::Action::Clicked(p) => Some(p),
                    tree_view::Action::SerializeKind(k) => {
                        let k = &k.as_any();
                        if let Some(k) = k.downcast_ref::<hyper_ast_gen_ts_cpp::types::Type>() {
                            aspects.ser_opt_cpp.insert(k.to_owned());
                        } else if let Some(k) =
                            k.downcast_ref::<hyper_ast_gen_ts_java::types::Type>()
                        {
                            aspects.ser_opt_java.insert(k.to_owned());
                        }

                        None
                    }
                    _ => None,
                }
            } else {
                None
            }
        });
    if let Some(o) = scroll_focus {
        // // wasm_rs_dbg::dbg!(o);
        // // wasm_rs_dbg::dbg!(&ports);
        // // wasm_rs_dbg::dbg!(ports.get(col - min_col + 1));
        *defered_focus_scroll = Some((o, col - min_col + 1, scroll));
        None
    } else {
        scroll.inner
    }
    // egui::Window::new("scroller button").show(ui.ctx(), |ui| {
    //     egui::Slider::new(&mut scroll.state.offset.y, 0.0..=200.0).ui(ui);

    //     scroll.state.store(ui.ctx(), scroll.id);
    // });
    // // wasm_rs_dbg::dbg!(scroll.state.offset);
}
const SC_COPY: egui::KeyboardShortcut =
    egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::C);
fn show_commitid_info(
    tracked: Option<TrackingResultWithChanges>,
    ui: &mut egui::Ui,
    code_ranges: Vec<&mut CodeRange>,
) {
    let f_commit = |ui: &mut egui::Ui, id: &str| {
        if ui.available_width() > 300.0 {
            ui.label(format!("commit {}", id));
        } else {
            let label = ui.label(format!("commit {}", &id[..8]));
            if label.hovered() {
                egui::show_tooltip(ui.ctx(), label.id.with("tooltip"), |ui| {
                    ui.label(id);
                    ui.label("CTRL+C to copy (and send in the debug console)");
                });
                if ui.input_mut(|mem| mem.consume_shortcut(&SC_COPY)) {
                    ui.output_mut(|mem| mem.copied_text = id.to_string());
                    wasm_rs_dbg::dbg!(id);
                }
            }
        }
    };
    let Some(tracked) = tracked  else {
        let id = &code_ranges[0].file.commit.id;
        f_commit(ui,id);
        return;
    };
    if let Some(cr) = tracked
        .track
        .intermediary
        .as_ref()
        .or(tracked.track.matched.get(0).as_ref().copied())
        .or(tracked.track.fallback.as_ref())
    {
        let id = &cr.file.commit.id;
        f_commit(ui, id);
    } else {
        let id = &code_ranges[0].file.commit.id;
        f_commit(ui, id);
    }
    let commits_processed = tracked.track.commits_processed - 1;
    if commits_processed > 1 {
        ui.label(format!("skipped {} commits", commits_processed));
    }
}
pub(crate) const TARGET_COLOR: egui::Color32 = egui::Color32::from_rgb(255, 100, 0);

pub(super) fn track(
    ctx: &egui::Context,
    commit: &Commit,
    file_path: &String,
    range: &Option<Range<usize>>,
    flags: &Flags,
) -> Promise<ehttp::Result<TrackingResultWithChanges>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    // TODO flags should not need the "=true"
    let flags = serde_qs::to_string(flags).unwrap(); //.replace("=true", "1").replace("=false", "0");
    let url = if let Some(range) = range {
        let flags = if flags.is_empty() {
            format!("")
        } else {
            format!("&{}", flags)
        };
        format!(
            "{}/track/github/{}/{}/{}/{}?start={}&end={}{}",
            API_URL,
            &commit.repo.user,
            &commit.repo.name,
            &commit.id,
            &file_path,
            &range.start,
            &range.end,
            flags
        )
    } else {
        let flags = if flags.is_empty() {
            format!("")
        } else {
            format!("?{}", flags)
        };
        format!(
            "{}/track/github/{}/{}/{}/{}{}",
            API_URL, &commit.repo.user, &commit.repo.name, &commit.id, &file_path, flags
        )
    };

    // // wasm_rs_dbg::dbg!(&url);
    let mut request = ehttp::Request::get(&url);
    // request
    //     .headers
    //     .insert("Content-Type".to_string(), "text".to_string());

    ehttp::fetch(request, move |response| {
        // // wasm_rs_dbg::dbg!(&response);
        ctx.request_repaint(); // wake up UI thread
        let resource = response
            .and_then(|response| {
                Resource::<TrackingResult>::from_response(&ctx, response)
                    .map(|x| x.map(|x| x.into()))
            })
            .and_then(|x| x.content.ok_or("Empty body".into()));
        sender.send(resource);
    });
    promise
}

pub(super) fn track_at_path(
    ctx: &egui::Context,
    commit: &Commit,
    exact_commit: Option<&Commit>,
    path: &[usize],
    flags: &Flags,
) -> Promise<ehttp::Result<TrackingResultWithChanges>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    // TODO flags should not need the "=true"
    let flags = serde_qs::to_string(flags).unwrap(); //.replace("=true", "1").replace("=false", "0");
    let url = {
        format!(
            "{}/track_at_path/github/{}/{}/{}/{}?{}{}",
            API_URL,
            &commit.repo.user,
            &commit.repo.name,
            &commit.id,
            path.into_iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join("/"),
            if let Some(exact_commit) = exact_commit {
                format!("before={}&", exact_commit.id)
            } else {
                "".to_string()
            },
            flags
        )
    };

    // wasm_rs_dbg::dbg!(&url);
    let mut request = ehttp::Request::get(&url);
    // request
    //     .headers
    //     .insert("Content-Type".to_string(), "text".to_string());

    ehttp::fetch(request, move |response| {
        // wasm_rs_dbg::dbg!(&response);
        ctx.request_repaint(); // wake up UI thread
        let resource = response
            .and_then(|response| {
                Resource::<TrackingResult>::from_response(&ctx, response)
                    .map(|x| x.map(|x| x.into()))
            })
            .and_then(|x| x.content.ok_or("Empty body".into()));
        sender.send(resource);
    });
    promise
}

pub(super) fn track_at_path_with_changes(
    ctx: &egui::Context,
    commit: &Commit,
    path: &[usize],
    flags: &Flags,
) -> Promise<ehttp::Result<TrackingResultWithChanges>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    // TODO flags should not need the "=true"
    let flags = serde_qs::to_string(flags).unwrap(); //.replace("=true", "1").replace("=false", "0");
    let url = {
        format!(
            "{}/track_at_path_with_changes/github/{}/{}/{}/{}?{}",
            API_URL,
            &commit.repo.user,
            &commit.repo.name,
            &commit.id,
            path.into_iter()
                .map(|x| x.to_string())
                .collect::<Vec<_>>()
                .join("/"),
            flags
        )
    };

    // wasm_rs_dbg::dbg!(&url);
    let mut request = ehttp::Request::get(&url);
    // request
    //     .headers
    //     .insert("Content-Type".to_string(), "text".to_string());

    ehttp::fetch(request, move |response| {
        // wasm_rs_dbg::dbg!(&response);
        ctx.request_repaint(); // wake up UI thread
        let resource = response
            .and_then(|response| {
                Resource::<TrackingResultWithChanges>::from_response(&ctx, response)
            })
            .and_then(|x| x.content.ok_or("Empty body".into()));
        sender.send(resource);
    });
    promise
}
