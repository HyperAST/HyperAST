use std::{
    hash::{DefaultHasher, Hash, Hasher},
    ops::Mul,
};

use crate::app::commit;

use lazy_static::lazy_static;

use super::{ProjectId, QueryId};

lazy_static! {
    static ref RES_PER_COMMIT: std::sync::Arc<
        std::sync::Mutex<
            crate::app::utils_commit::BorrowFrameCache<
                super::ResultsPerCommit,
                ComputeResPerCommit,
            >,
        >,
    > = Default::default();
}

lazy_static! {
    static ref LAYOUT: std::sync::Arc<
        std::sync::Mutex<
            crate::app::utils_commit::BorrowFrameCache<commit::CommitsLayoutTimed, ComputeLayout>,
        >,
    > = Default::default();
}

#[derive(Default)]
pub struct ComputeResPerCommit {}

type ResHash = u64;

impl
    egui::util::cache::ComputerMut<
        (
            (ProjectId, QueryId, ResHash),
            &crate::app::utils_results_batched::ComputeResults,
        ),
        super::ResultsPerCommit,
    > for ComputeResPerCommit
{
    fn compute(
        &mut self,
        ((pid, qid, _), r): (
            (ProjectId, QueryId, ResHash),
            &crate::app::utils_results_batched::ComputeResults,
        ),
    ) -> super::ResultsPerCommit {
        let mut res = super::ResultsPerCommit::default();

        update_results_per_commit(&mut res, r);
        res
    }
}

#[derive(Default)]
pub struct ComputeLayout {}

impl
    egui::util::cache::ComputerMut<
        ((&str, &str, usize), &super::CommitMdStore),
        commit::CommitsLayoutTimed,
    > for ComputeLayout
{
    fn compute(
        &mut self,
        ((name, target, _), fetched_commit_metadata): ((&str, &str, usize), &super::CommitMdStore),
    ) -> commit::CommitsLayoutTimed {
        commit::compute_commit_layout_timed(
            |id| fetched_commit_metadata.get(id)?.as_ref().ok().cloned(),
            Some((name.to_string(), target.to_string())).into_iter(),
        )
    }
}

impl crate::HyperApp {
    pub(crate) fn print_commit_graph_timed(&mut self, ui: &mut egui::Ui) {
        let res_per_commit = &mut RES_PER_COMMIT.lock().unwrap();
        let layout_cache = &mut LAYOUT.lock().unwrap();
        ui.add_space(20.0);
        let ready_count = self.data.fetched_commit_metadata.len_local();
        for repo_id in self.data.selected_code_data.project_ids() {
            let Some((r, mut c)) = self.data.selected_code_data.get_mut(repo_id) else {
                continue;
            };
            let commit_slice = &mut c.iter_mut();
            let Some(branch) = commit_slice
                .next()
                .map(|c| (format!("{}/{}", r.user, r.name), c.clone()))
            else {
                continue;
            };
            let id = ui.make_persistent_id("bottom_cache_layout");
            let r = r.clone();

            let results_per_commit: Option<&super::ResultsPerCommit> = {
                if let Some(r) = self.data.queries_results.iter().find(|x| x.0 == repo_id) {
                    let qid = r.1;
                    if let Some(Ok(r)) = r.2.get() {
                        Some(res_per_commit.get2((repo_id, qid, r.h()), r))
                    } else {
                        None
                    }
                } else {
                    None
                }
            };

            let cached = {
                if self.data.fetched_commit_metadata.is_absent(&branch.1) {
                    let commit = r.clone().with(&branch.1);
                    let fetching = commit::fetch_commit(ui.ctx(), &self.data.api_addr, &commit);
                    self.data
                        .fetched_commit_metadata
                        .insert(commit.id, fetching);
                }
                layout_cache.get2(
                    (
                        &branch.0,
                        &branch.1,
                        ready_count, // TODO find something more reliable
                    ),
                    &self.data.fetched_commit_metadata,
                )
            };

            // { // TODO now use egui_plot, it will handle interation properly and should not be difficult to migrate I think.
            //     use egui_plot::*;
            //     let sin: PlotPoints = (0..1000)
            //         .map(|i| {
            //             let x = i as f64 * 0.01;
            //             [x, x.sin()]
            //         })
            //         .collect();
            //     let line = Line::new(sin);
            //     Plot::new("my_plot")
            //         .view_aspect(2.0)
            //         .show(ui, |plot_ui| plot_ui.line(line));
            // }

            let max_time = cached
                .min_time
                .max(cached.max_time - self.data.offset_fetch);
            let min_time = cached.min_time.max(max_time - self.data.max_fetch);
            // cached.min_time = cached.min_time.max(cached.max_time - self.data.max_fetch);
            let cached = cached;
            let width = ui.available_width() - 120.0;
            let _time_norm = |t: i64| t as f32 / (max_time - min_time).max(1) as f32;
            let _time_scaling = |t: i64| _time_norm(t) * width;
            assert!(cached.max_delta >= 0);
            let time_scaling = |t: i64| _time_scaling(t - min_time);
            let time_scaling_y = |t: i64| {
                _time_norm(t)
                    .clamp(0.0, 1.0)
                    .sqrt()
                    .mul((width / 4.0).min(300.0))
                    .max(0.0)
            };

            assert!(cached.max_delta < i64::MAX);
            let desired_size = egui::vec2(
                ui.available_width(),
                if cached.max_delta <= 0 {
                    0.0
                } else {
                    time_scaling_y(cached.max_delta)
                },
            ) + egui::vec2(0.0, 60.0);
            let (_rect, resp) = ui.allocate_exact_size(desired_size, egui::Sense::click());
            let rect = _rect.shrink(30.0);
            let min = rect.min;

            // background
            ui.painter().rect_filled(
                _rect
                    .shrink2(egui::vec2(55.0, 0.0))
                    .translate(egui::vec2(-30.0, 0.0)),
                ui.visuals().window_rounding,
                ui.visuals().extreme_bg_color,
            );

            let parent_rel_color = if ui.visuals().dark_mode {
                egui::Color32::WHITE
            } else {
                egui::Color32::BLACK
            };

            // region:   --- Ticks
            const SECONDS_PER_DAY: i64 = 60 * 60 * 24;
            let day_count = (max_time - min_time) / (SECONDS_PER_DAY);
            let day_width = _time_scaling(SECONDS_PER_DAY);
            let day_offset = min.x + _time_scaling(min_time % (SECONDS_PER_DAY));
            for d in 0..day_count {
                let x = day_offset + d as f32 * day_width;
                ui.painter().line_segment(
                    [egui::pos2(x, _rect.min.y), egui::pos2(x, _rect.min.y + 5.0)],
                    egui::Stroke::new(2.0, egui::Color32::GRAY),
                );
            }
            // endregion --- Ticks

            let mut to_fetch = vec![];
            let mut to_poll = vec![];
            assert!(cached.branches.len() <= 1);
            'subs: for commit::SubsTimed {
                prev,
                prev_sub,
                start,
                end,
                succ,
                succ_sub,
                delta_time,
            } in &cached.subs
            {
                let mut prev_p = min
                    + egui::vec2(
                        if cached.times[*prev] == -1 {
                            0.0
                        } else {
                            time_scaling(cached.times[*prev])
                        },
                        time_scaling_y(cached.subs[*prev_sub].delta_time),
                    );
                let curr_y = min.y + time_scaling_y(*delta_time);
                for i in *start..*end {
                    let t = cached.times[i];
                    if t == -1 {
                        if self
                            .data
                            .fetched_commit_metadata
                            .is_absent(&cached.commits[i])
                        {
                            to_fetch.push(&cached.commits[i]);
                        } else if self
                            .data
                            .fetched_commit_metadata
                            .get(&cached.commits[i])
                            .is_none()
                        {
                            to_poll.push(&cached.commits[i]);
                        }
                        break;
                    }
                    let center = egui::pos2(min.x + time_scaling(t), curr_y);

                    let commit = &cached.commits[i];

                    // region: painter queried values when there is a change
                    let before = if i != *start {
                        Some(cached.commits[i - 1].as_str())
                    } else if *prev != usize::MAX {
                        Some(cached.commits[*prev].as_str())
                    } else {
                        None
                    };
                    let after = if i + 1 < *end {
                        Some(cached.commits[i + 1].as_str())
                    } else if *succ != usize::MAX {
                        Some(cached.commits[*succ].as_str())
                    } else {
                        None
                    };

                    let circle_fill_color = if results_per_commit
                        .and_then(|x| x._get_offset(commit))
                        .is_some()
                    {
                        egui::Color32::DARK_GREEN
                    } else {
                        egui::Color32::GREEN
                    };

                    let diff = results_per_commit
                        .zip(before)
                        .and_then(|(x, c2)| x.try_diff_as_string(commit, c2));

                    if i == *start {
                        let corner_p = egui::pos2(prev_p.x.max(center.x - 10.0), center.y);
                        let parent_rel_color = egui::Color32::LIGHT_GRAY;
                        let stroke = egui::Stroke::new(2.0, parent_rel_color);
                        ui.painter().line_segment([prev_p, corner_p], stroke);
                        ui.painter().line_segment([corner_p, center], stroke);
                    } else {
                        ui.painter().line_segment(
                            [prev_p, center],
                            egui::Stroke::new(2.0, parent_rel_color),
                        );
                        if let Some(text) = diff {
                            let pos = egui::Rect::from_min_max(prev_p, center).center();
                            // let text_color = ui.style().visuals.text_color();
                            let text_color = egui::Color32::YELLOW;
                            let font_id = egui::TextStyle::Body.resolve(ui.style());
                            let anchor = egui::Align2::RIGHT_BOTTOM;
                            ui.painter().text(pos, anchor, text, font_id, text_color);
                        }
                    }

                    // stop rendering when reached limit
                    if max_time - t > self.data.max_fetch {
                        let fill_color = egui::Color32::RED;
                        ui.painter().circle_filled(center, 10.0, fill_color);
                        continue 'subs;
                    }

                    ui.painter().circle_filled(center, 4.0, circle_fill_color);

                    let resp = ui.interact(
                        egui::Rect::from_center_size(center, egui::Vec2::splat(8.0)),
                        ui.id().with(&cached.commits[i]),
                        egui::Sense::click(),
                    );
                    let vals_offset = results_per_commit
                        .and_then(|x| x.offset_with_variation(commit.as_str(), before, after));
                    if let Some(offset) = vals_offset {
                        let pos = center + (0.0, -10.0).into();
                        let text_color = ui.style().visuals.text_color();
                        let text = results_per_commit.unwrap().vals_to_string(offset);
                        let font_id = egui::TextStyle::Body.resolve(ui.style());
                        let anchor = egui::Align2::RIGHT_BOTTOM;
                        // let rect = egui::Align2::RIGHT_BOTTOM.anchor_size(pos, galley.size());
                        // ui.painter().galley(rect.min, galley.clone(), text_color);
                        ui.painter().text(pos, anchor, text, font_id, text_color);
                    }
                    // endregion

                    // similar to a tooltip
                    let resp = resp.on_hover_ui(|ui| {
                        let vals_offset =
                            results_per_commit.and_then(|x| x.offset(commit.as_str()));
                        let text = if let Some(v) = vals_offset {
                            format!(
                                "{}\n{}",
                                commit,
                                results_per_commit.unwrap().vals_to_string(v)
                            )
                        } else {
                            format!("{}", commit)
                        };
                        ui.label(text);
                    });
                    const SC_COPY: egui::KeyboardShortcut =
                        egui::KeyboardShortcut::new(egui::Modifiers::CTRL, egui::Key::C);
                    if resp.hovered() {
                        if ui.input_mut(|mem| mem.consume_shortcut(&SC_COPY)) {
                            ui.output_mut(|mem| mem.copied_text = commit.to_string());
                        }
                    }
                    if resp.clicked() {
                        log::debug!("");
                        self.selected_commit = Some((repo_id, commit.to_string()));
                    }
                    // if let Some(pos) = resp.hover_pos() {
                    //     let vals_offset =
                    //         results_per_commit.and_then(|x| x.offset(commit.as_str()));
                    //     let text = if let Some(v) = vals_offset {
                    //         format!(
                    //             "{}\n{}",
                    //             commit,
                    //             results_per_commit.unwrap().vals_to_string(v)
                    //         )
                    //     } else {
                    //         format!("{}", commit)
                    //     };
                    //     let font_id = egui::TextStyle::Button.resolve(ui.style());
                    //     ui.painter().text(
                    //         pos + (20.0, 0.0).into(),
                    //         egui::Align2::RIGHT_BOTTOM,
                    //         text,
                    //         font_id,
                    //         ui.style().visuals.text_color(),
                    //     );
                    // }
                    prev_p = center;
                }

                if *succ < usize::MAX {
                    let y = min.y + time_scaling_y(cached.subs[*succ_sub].delta_time);
                    let x = min.x + time_scaling(cached.times[*succ]);
                    let succ_p = egui::pos2(x, y);
                    let mid_p = egui::pos2(prev_p.x.min(x + 5.0), prev_p.y);
                    ui.painter()
                        .line_segment([prev_p, mid_p], egui::Stroke::new(2.0, parent_rel_color));
                    ui.painter()
                        .line_segment([mid_p, succ_p], egui::Stroke::new(2.0, parent_rel_color));
                }
            }
            for &b in &cached.branches {
                let b = cached.subs[b].prev;
                let p = (
                    cached.times[b],
                    min.y + time_scaling_y(cached.subs[b].delta_time),
                );
                let center = egui::pos2(time_scaling(p.0), p.1);
                let font_id = egui::TextStyle::Button.resolve(ui.style());
                let pos = center + (70.0, -10.0).into();
                let text = &cached.commits[b];
                let text_color = ui.style().visuals.text_color();
                let galley = ui
                    .painter()
                    .layout(text.to_string(), font_id, text_color, 100.0);
                let angle = 0.9;
                let rect = egui::Rect::from_min_size(pos, galley.size());
                if !galley.is_empty() {
                    let shape = epaint::TextShape::new(rect.min, galley, text_color);
                    ui.painter().add(shape.with_angle(angle));
                };
            }
            for id in to_fetch {
                if !self.data.fetched_commit_metadata.is_absent(id) {
                    continue;
                }
                let repo = r.clone();
                let id = id.clone();
                let commit = crate::app::types::Commit { repo, id };
                let v = commit::fetch_commit(ui.ctx(), &self.data.api_addr, &commit);
                self.data.fetched_commit_metadata.insert(commit.id, v);
            }
            for id in to_poll {
                if self.data.fetched_commit_metadata.try_poll_with(id, |x| x) {}
            }
            if let Some(pos) = resp.hover_pos() {
                let top = egui::pos2(pos.x, _rect.top());
                let bot = egui::pos2(pos.x, _rect.bottom());
                ui.painter()
                    .line_segment([top, bot], egui::Stroke::new(2.0, parent_rel_color));
                let Some(x_ratio) = egui::emath::inverse_lerp(rect.x_range().into(), pos.x) else {
                    continue;
                };
                if min_time == i64::MAX {
                    continue;
                }
                let timestamp = min_time + (x_ratio * (max_time - min_time) as f32) as i64;
                let Some(naive_datetime) = chrono::DateTime::from_timestamp(timestamp, 0) else {
                    continue;
                };
                let datetime_again = naive_datetime.to_utc();
                let text = format!("{}", datetime_again);
                let font_id = egui::TextStyle::Button.resolve(ui.style());
                ui.painter().text(
                    pos + (20.0, 0.0).into(),
                    egui::Align2::LEFT_TOP,
                    text,
                    font_id,
                    ui.style().visuals.text_color(),
                );
            }
        }
        ui.add_space(20.0);
        res_per_commit.evice_cache();
        layout_cache.evice_cache();
    }
}

// Non timed
impl crate::HyperApp {
    pub(crate) fn print_commit_graph(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        let id = ui.make_persistent_id("bottom_cache_layout");
        let cached = ctx.data_mut(|mem| {
            let _v = (
                self.data.selected_code_data.len(), // TODO use something better
                self.data.fetched_commit_metadata.len_local(),
                commit::CommitsLayout::default(),
            );
            let v = mem.get_temp_mut_or_insert_with(id, || _v.clone());
            if &_v == v {
                return v.2.clone();
            }

            let layout = commit::compute_commit_layout(
                |id| {
                    self.data
                        .fetched_commit_metadata
                        .get(id)
                        .and_then(|x| x.as_ref().ok())
                        .map(|x| x.clone())
                },
                self.data.selected_code_data.project_ids().filter_map(|i| {
                    let (r, mut c) = self.data.selected_code_data.get_mut(i)?;
                    let commit_slice = &mut c.iter_mut();
                    commit_slice
                        .next()
                        .map(|c| (format!("b:{}/{}", r.user, r.name), c.clone()))
                }),
            );
            v.2 = layout;

            v.2.clone()
        });
        let min = ui.available_rect_before_wrap().min.to_vec2() + egui::vec2(20.0, 20.0);
        let rect = &(cached.rect * 3.0 + egui::Margin::same(20.0)).translate(min);
        ui.painter()
            .debug_rect(*rect, egui::Color32::RED, "cached rect");
        let desired_size = rect.size();
        ui.allocate_exact_size(desired_size, egui::Sense::click());

        // log::debug!("{:?}", cached.subs);
        let parent_rel_color = if ui.visuals().dark_mode {
            egui::Color32::WHITE
        } else {
            egui::Color32::BLACK
        };
        for commit::Subs {
            prev: child_id,
            start,
            end,
            succ,
        } in &cached.subs
        {
            let mut prev_p = cached.pos[*child_id] * 3.0 + min;
            for i in *start..*end {
                let p = cached.pos[i];
                let center = p * 3.0 + min;
                ui.painter()
                    .line_segment([prev_p, center], egui::Stroke::new(2.0, parent_rel_color));
                let fill_color = if cached.commits[i].starts_with("m:") {
                    egui::Color32::DARK_GREEN
                } else {
                    egui::Color32::GREEN
                };
                ui.painter().circle_filled(center, 6.0, fill_color);
                prev_p = center;
            }

            if *succ < usize::MAX {
                let p = cached.pos[*succ];
                let succ_p = p * 3.0 + min;
                ui.painter()
                    .line_segment([prev_p, succ_p], egui::Stroke::new(2.0, parent_rel_color));
            }
        }
        for b in cached.branches {
            let b = cached.subs[b].prev;
            let p = cached.pos[b];
            let center = p * 3.0 + min - egui::vec2(2.0, 6.0) * 3.0;
            ui.painter().debug_rect(
                egui::Rect::from_center_size(center, egui::vec2(7.0, 4.0) * 3.0),
                egui::Color32::KHAKI,
                &cached.commits[b],
            );
        }
    }
}

fn update_results_per_commit(
    results_per_commit: &mut super::ResultsPerCommit,
    r: &super::utils_results_batched::ComputeResults,
) {
    let header = r.results.iter().find(|x| x.is_ok());
    let Some(header) = header.as_ref() else {
        wasm_rs_dbg::dbg!("issue with header");
        panic!("issue with header");
    };
    // let font_id = egui::TextStyle::Body.resolve(ui.style());
    // let text_color = ui.style().visuals.text_color();
    let header = header.as_ref().unwrap();
    let h: Vec<String> = header
        .inner
        .result
        .as_array()
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(i, h)| i.to_string())
        .collect();
    let mut vals = vec![0; h.len()];
    results_per_commit.set_cols(&h);
    for r in &r.results {
        if let Ok(r) = r {
            // let c: [u8;8] = r.commit.as_bytes().as_chunks::<8>().0[0];
            // let mut c: [u8; 8] = [0; 8];
            // c.copy_from_slice(
            //     &r.commit.as_bytes()[..8],
            // );
            // panic!("{} {}",r.inner.result.is_object(),r.inner.result.is_array());
            for (i, r) in r.inner.result.as_array().unwrap().into_iter().enumerate() {
                if i > 2 {
                    break;
                }
                vals[i] = r.as_i64().unwrap() as i32;
            }

            results_per_commit.insert(
                &r.commit,
                // || {
                //     let text = crate::app::utils::join(vals.iter(), "\n").to_string();
                //     ui.painter()
                //         .layout_no_wrap(text, font_id.clone(), text_color)
                //         .into()
                // },
                r.inner.compute_time as f32,
                &[],
                &vals,
            );
        }
    }
    log::debug!("{:?}", results_per_commit);
}
