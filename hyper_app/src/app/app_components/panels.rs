use core::f32;
use std::{ops::Mul, usize};

use chrono::Duration;
use egui::Widget;
use re_ui::{DesignTokens, UiExt};

use crate::app::{
    commit,
    querying::{self, ComputeConfigQuery},
    show_projects_actions,
    types::{self, Commit, Config},
    LocalOrRemote,
};

use super::utils_results_batched::ComputeError;

impl crate::HyperApp {
    pub(crate) fn show_left_panel(&mut self, ctx: &egui::Context) {
        egui::SidePanel::left("left_panel")
            .default_width(ctx.screen_rect().width() / 6.0)
            .frame(egui::Frame {
                fill: ctx.style().visuals.panel_fill,
                ..Default::default()
            })
            .show_animated(ctx, self.show_left_panel, |ui| {
                self.show_left_panel_views_props(ui);
                // self.show_left_panel_custom_contents(ui);
            });
    }

    fn show_left_panel_views_props(&mut self, ui: &mut egui::Ui) {
        for tile_id in self.tree.active_tiles() {
            let Some(&pane) = self.tree.tiles.get_pane(&tile_id) else {
                // log::error!("{:?}", tile_id);
                continue;
            };
            let title = self.tabs[pane as usize].title();
            use super::re_ui_collapse::SectionCollapsingHeader;
            SectionCollapsingHeader::new(title)
                .default_open(false)
                .show(ui, |ui| {
                    if let super::Tab::ProjectSelection() = self.tabs[pane as usize] {
                        show_projects_actions(ui, &mut self.data);
                        ui.indent("proj_list", |ui| {
                            let mut span = ui.full_span();
                            span.min += 10.0;
                            ui.full_span_scope(span, |ui| self.show_repositories(ui))
                        });
                    } else if let super::Tab::QueryResults { id, format } =
                        &mut self.tabs[pane as usize]
                    {
                        egui::ComboBox::from_label("Commits")
                            .selected_text(format.as_ref())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(format, super::ResultFormat::List, "List");
                                ui.selectable_value(format, super::ResultFormat::Table, "Table");
                                ui.add_enabled_ui(false, |ui| {
                                    ui.selectable_value(format, super::ResultFormat::Json, "Json");
                                });
                            });
                    } else if let super::Tab::LocalQuery(id) = self.tabs[pane as usize] {
                        self.show_local_query_left_panel(ui, id);
                    }
                    match &self.tabs[pane as usize] {
                        _ => (),
                    };
                });
        }
    }
    fn show_local_query_left_panel(&mut self, ui: &mut egui::Ui, id: u16) {
        let query = &mut self.data.queries[id as usize];
        egui::Slider::new(&mut query.commits, 1..=100)
            .text("#commits")
            .clamp_to_range(false)
            .ui(ui)
            .on_hover_text("Maximum number of commits that will be processed.");
        egui::Slider::new(&mut query.max_matches, 1..=1000)
            .text("match limit")
            .clamp_to_range(false)
            .ui(ui)
            .on_hover_text("Maximum number of match per commit\n, for any of the patterns.");
        egui::Slider::new(&mut query.timeout, 1..=5000)
            .text("commit timeout")
            .clamp_to_range(false)
            .ui(ui)
            .on_hover_text("Maximum time to match each commit.");
        ui.add_enabled(
            false,
            egui::Slider::new(&mut query.timeout, 1..=5000)
                .text("commit timeout")
                .clamp_to_range(false),
        )
        .on_hover_text("Maximum time to match all commit.");
        let compute_button = ui.add_enabled(false, egui::Button::new("Compute All"));
        let q_res_ids = &mut query.results;
        if self.data.selected_code_data.len() != q_res_ids.len() {
            // TODO update on new commit
            // TODO update list instead of recreating it
            let mut l = self.data.selected_code_data.project_ids().filter_map(|i| {
                let (r, mut c) = self.data.selected_code_data.get_mut(i)?;
                let c = &mut c.iter_mut();
                c.next().map(|c| {
                    let commit = types::Commit {
                        repo: r.clone(),
                        id: c.clone(),
                    };
                    (i, commit)
                })
            });
            let mut it = q_res_ids.iter();
            let mut r = vec![];
            loop {
                match (l.next(), it.next()) {
                    (None, None) => break,
                    (Some((rid, c)), None) => {
                        let qrid: u16 = self.data.queries_results.len().try_into().unwrap();
                        r.push(qrid);
                        self.data.queries_results.push(super::QueryResults(
                            rid,
                            id,
                            Default::default(),
                            u16::MAX,
                        ));
                    }
                    (None, Some(&id)) => {
                        // nothing to do ProjectIds are valid for the duration of the session
                        // self.data.queries_results[id as usize].0 = u16::MAX;
                    }
                    (Some((i, _)), Some(&id)) => {
                        self.data.queries_results[id as usize].0 = i;
                        r.push(id);
                    }
                }
            }
            *q_res_ids = r;
        }
        let q_res_ids = &self.data.queries[id as usize].results;
        ui.style_mut().spacing.item_spacing = egui::vec2(3.0, 2.0);
        for q_res_id in q_res_ids {
            if *q_res_id == u16::MAX {
                continue;
            }
            let q_res = &mut self.data.queries_results[*q_res_id as usize];
            let Some((repo, mut c)) = self.data.selected_code_data.get_mut(q_res.0) else {
                continue;
            };
            let Some(c) = c.iter_mut().next() else {
                continue;
            };

            fn update_tiles(
                tabs: &mut Vec<crate::app::Tab>,
                tree: &mut egui_tiles::Tree<crate::app::TabId>,
                q_res: &mut super::QueryResults,
                q_res_id: &u16,
            ) {
                let tid = tabs.len() as u16;
                q_res.3 = tid;
                tabs.push(crate::app::Tab::QueryResults {
                    id: *q_res_id,
                    format: super::ResultFormat::Table,
                });
                let tid = tree.tiles.insert_new(egui_tiles::Tile::Pane(tid));
                tree.move_tile_to_container(tid, tree.root().unwrap(), usize::MAX, false);
            }
            let compute_button = ui
                .horizontal(|ui| {
                    ui.style_mut().spacing.button_padding = egui::vec2(3.0, 2.0);
                    let d = egui::Color32::DARK_GREEN;
                    let n = egui::Color32::GREEN;
                    let w = &mut ui.style_mut().visuals.widgets;
                    w.open.weak_bg_fill = d;
                    w.active.weak_bg_fill = d;
                    w.hovered.weak_bg_fill = n;
                    w.inactive.weak_bg_fill = d;
                    let q_res = &mut self.data.queries_results[*q_res_id as usize];
                    let compute_button;
                    if let Some(content) = q_res.2.get() {
                        match content {
                            Ok(content) => {
                                let rows = content.rows.lock().unwrap();
                                if rows.2 {
                                    if let [Err(err)] = rows.1.as_slice() {
                                        let d = egui::Color32::DARK_RED;
                                        let n = egui::Color32::RED;
                                        let w = &mut ui.style_mut().visuals.widgets;
                                        w.open.weak_bg_fill = d;
                                        w.active.weak_bg_fill = d;
                                        w.hovered.weak_bg_fill = n;
                                        w.inactive.weak_bg_fill = d;
                                        compute_button =
                                            ui.add(egui::Button::new("⚠")).on_hover_ui(|ui| {
                                                ui.label(format!(
                                                    "streamed results {}/{}",
                                                    rows.1.len(),
                                                    content.commits
                                                ));
                                                ui.label(format!("{:?}", err));
                                            });
                                    } else {
                                        compute_button =
                                            ui.add(egui::Button::new("↺")).on_hover_ui(|ui| {
                                                ui.label(format!(
                                                    "streamed results {}/{}",
                                                    rows.1.len(),
                                                    content.commits
                                                ));
                                                // crate::app::utils_results_batched::show_short_result_aux(
                                                //     content, ui,
                                                // )
                                            });
                                    }
                                } else {
                                    compute_button = ui.spinner().on_hover_text(format!(
                                        "waiting for the rest of the entries: {}/{}",
                                        rows.1.len(),
                                        content.commits
                                    ));
                                }
                            }
                            Err(err) => {
                                let d = egui::Color32::DARK_RED;
                                let n = egui::Color32::RED;
                                let w = &mut ui.style_mut().visuals.widgets;
                                w.open.weak_bg_fill = d;
                                w.active.weak_bg_fill = d;
                                w.hovered.weak_bg_fill = n;
                                w.inactive.weak_bg_fill = d;
                                compute_button = ui
                                    .add(egui::Button::new("⚠"))
                                    .on_hover_text(format!("{}\n{}", err.head(), err.content()));
                            }
                        }
                        ui.label(&format!(
                            "{}/{}/{}",
                            repo.user,
                            repo.name,
                            &c[..6.min(c.len())]
                        ));
                        if q_res.2.is_waiting() {
                            ui.spinner();
                            let synced = Self::sync_query_results(q_res);
                            if let Ok(true) = synced {
                                self.save_interval = std::time::Duration::ZERO;
                                if q_res.3 == u16::MAX {
                                    update_tiles(&mut self.tabs, &mut self.tree, q_res, q_res_id);
                                }
                            }
                        }
                    } else {
                        let q_res = &mut self.data.queries_results[*q_res_id as usize];
                        let synced = Self::sync_query_results(q_res);
                        if let Err(Some(err)) = &synced {
                            let d = egui::Color32::DARK_RED;
                            let n = egui::Color32::RED;
                            let w = &mut ui.style_mut().visuals.widgets;
                            w.open.weak_bg_fill = d;
                            w.active.weak_bg_fill = d;
                            w.hovered.weak_bg_fill = n;
                            w.inactive.weak_bg_fill = d;
                            compute_button = ui.add(egui::Button::new("⚠")).on_hover_text(err);
                        } else if let Err(None) = &synced {
                            compute_button = ui.spinner();
                        } else if let Ok(false) = &synced {
                            compute_button = ui.add(egui::Button::new("⏵"));
                        }
                        // finally, if unassigned pane then add it
                        else {
                            self.save_interval = std::time::Duration::ZERO;
                            if q_res.3 == u16::MAX {
                                update_tiles(&mut self.tabs, &mut self.tree, q_res, q_res_id);
                            }
                            compute_button = ui.add(egui::Button::new("⏵"));
                        }
                        ui.label(&format!(
                            "{}/{}/{}",
                            repo.user,
                            repo.name,
                            &c[..6.min(c.len())]
                        ));
                    };
                    compute_button
                })
                .inner;
            let q_res = &mut self.data.queries_results[*q_res_id as usize];

            if compute_button.clicked() {
                let (repo, mut c) = self.data.selected_code_data.get_mut(q_res.0).unwrap();
                let query = self.data.queries[q_res.1 as usize]
                    .query
                    .as_ref()
                    .to_string();
                wasm_rs_dbg::dbg!(&query);
                let config = Config::MavenJava;
                let language = config.language().to_string();
                let commits = self.data.queries[q_res.1 as usize].commits as usize;
                let commit = Commit {
                    repo: repo.clone(),
                    id: c.iter_mut().next().cloned().unwrap(),
                };
                let max_matches = self.data.queries[q_res.1 as usize].max_matches;
                let timeout = self.data.queries[q_res.1 as usize].timeout;
                let prom = querying::remote_compute_query_aux(
                    ui.ctx(),
                    &self.data.api_addr,
                    &ComputeConfigQuery {
                        commit,
                        config: Config::MavenJava,
                        len: commits,
                    },
                    querying::QueryContent {
                        language,
                        query,
                        commits,
                        max_matches,
                        timeout,
                    },
                );
                q_res.2.buffer(prom);
            }
        }
        if compute_button.clicked() {
            todo!()
            // *loc_rem = LocalOrRemote::Remote(querying::remote_compute_query_aux(
            //     ui.ctx(),
            //     &self.data.api_addr,
            //     &self.data.query,
            //     &mut self.data.querying_context,
            // ));
        }
    }

    /// returns Ok(true) if it is now local
    fn sync_query_results(
        q_res: &mut super::QueryResults,
        // results_per_commit: &mut super::ResultsPerCommit,
    ) -> Result<bool, Option<String>> {
        if q_res.2.is_waiting() {
            if q_res.2.try_poll_with(|x| {
                // x.map_err(|x| querying::QueryingError::NetworkError(x))?
                //     .content
                //     .ok_or(querying::QueryingError::NetworkError(
                //         "content was not deserialized or empty".to_string(),
                //     ))?
                //     .map(|x| x.into())
                x
            }) {
                Ok(true)
            } else {
                Err(None)
            }
        } else {
            Ok(false)
        }

        // let a = std::mem::take(&mut q_res.2);
        // match a {
        //     LocalOrRemote::Remote(prom) => {
        //         match prom.try_take() {
        //             Ok(Ok(r)) => {
        //                 if let Some(r) = r.content {
        //                     if let Ok(r) = &r {
        //                         // Self::update_results_per_commit(results_per_commit, r);
        //                     }
        //                     q_res.2 = LocalOrRemote::Local(r);
        //                     Ok(true)
        //                 } else {
        //                     Ok(false)
        //                 }
        //             }
        //             Ok(Err(err)) => {
        //                 log::error!("{}", err);
        //                 Err(Some(format!("error: {}", err)))
        //             }
        //             Err(prom) => {
        //                 q_res.2 = LocalOrRemote::Remote(prom);
        //                 Err(None)
        //             }
        //         }
        //     }
        //     LocalOrRemote::None => Ok(false),
        //     _ => Ok(false),
        // }
    }

    fn show_left_panel_custom_contents(&mut self, ui: &mut egui::Ui) {
        egui::TopBottomPanel::top("left_panel_top_bar")
            .min_height(3.0 * re_ui::DesignTokens::title_bar_height())
            .frame(egui::Frame {
                inner_margin: egui::Margin::symmetric(re_ui::DesignTokens::view_padding(), 0.0),
                ..Default::default()
            })
            .show_inside(ui, |ui| self.show_actions(ui));

        // list_item::list_item_scope(ui, "testing stuff", |ui| {
        //     ui.list_item().show_hierarchical(
        //         ui,
        //         list_item::PropertyContent::new("Text (editable)")
        //             .value_text_mut(&mut self.latest_cmd),
        //     );
        //     ui.list_item().show_hierarchical(
        //         ui,
        //         list_item::PropertyContent::new("Color")
        //             .with_icon(&re_ui::icons::SPACE_VIEW_TEXT)
        //             .action_button(&re_ui::icons::ADD, || {
        //                 // re_log::warn!("Add button clicked");
        //             })
        //             .value_color_mut(&mut egui::Color32::RED.to_array()),
        //     );
        //     ui.list_item().show_hierarchical(
        //         ui,
        //         list_item::PropertyContent::new("Bool (editable)")
        //             .value_bool_mut(&mut self.dummy_bool),
        //     );
        // });

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                egui::Frame {
                    inner_margin: egui::Margin::same(re_ui::DesignTokens::view_padding()),
                    ..Default::default()
                }
                .show(ui, |ui| self.left_panel_mid_section_ui(ui));
            });

        egui::ScrollArea::vertical()
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                egui::Frame {
                    inner_margin: egui::Margin::same(re_ui::DesignTokens::view_padding()),
                    ..Default::default()
                }
                .show(ui, |ui| self.left_panel_bottom_section_ui(ui));
            });
    }

    pub(crate) fn bottom_panel(&mut self, ctx: &egui::Context) {
        let mut frame_style = DesignTokens::bottom_panel_frame();
        if !ctx.style().visuals.dark_mode {
            frame_style.fill = egui::Visuals::light().window_fill;
            frame_style.stroke = egui::Visuals::light().window_stroke;
            frame_style.shadow.color = egui::Visuals::light().window_shadow.color;
        }
        egui::TopBottomPanel::bottom("bottom_panel")
            .resizable(true)
            .frame(frame_style)
            .show_animated(ctx, self.show_bottom_panel, |ui| {
                let view = &mut self.bottom_view;
                ui.horizontal(|ui| {
                    ui.strong("Bottom panel");
                    ui.add_space(20.0);
                    egui::ComboBox::from_label("View")
                        .selected_text(view.as_ref())
                        .show_ui(ui, |ui| {
                            ui.selectable_value(view, super::BottomPanelConfig::Commits, "Commits");
                            ui.selectable_value(
                                view,
                                super::BottomPanelConfig::CommitsTime,
                                "Commits Time",
                            );
                            ui.add_enabled_ui(false, |ui| {
                                ui.selectable_value(
                                    view,
                                    super::BottomPanelConfig::Temporal,
                                    "Temporal",
                                );
                                ui.selectable_value(
                                    view,
                                    super::BottomPanelConfig::Temporal,
                                    "Commit Metadata",
                                );
                            })
                        });
                    ui.add_space(20.0);
                    const MAX: i64 = 60 * 60 * 24 * 365;
                    const MIN: i64 = 60 * 60 * 24 * 1;
                    let resp =
                        &egui::widgets::Slider::new(&mut self.data.offset_fetch, 0..=MAX).ui(ui);
                    if resp.drag_stopped() {
                        self.save_interval = std::time::Duration::ZERO;
                    }
                    let resp = &egui::widgets::Slider::new(&mut self.data.max_fetch, MIN..=MAX)
                        .clamp_to_range(false)
                        .custom_formatter(|n, _| {
                            let n = n as i32;
                            let days = n / (60 * 60 * 24);
                            let hours = (n / (60 * 60)) % 24;
                            let mins = (n / 60) % 60;
                            let secs = n % 60;
                            format!("{days:02}:{hours:02}:{mins:02}:{secs:02}")
                        })
                        .custom_parser(|s| {
                            let parts: Vec<&str> = s.split(':').collect();
                            if parts.len() == 4 {
                                parts[0]
                                    .parse::<i32>()
                                    .and_then(|d| {
                                        parts[1].parse::<i32>().and_then(|h| {
                                            parts[2].parse::<i32>().and_then(|m| {
                                                parts[3].parse::<i32>().map(|s| {
                                                    ((d * 60 * 60 * 24)
                                                        + (h * 60 * 60)
                                                        + (m * 60)
                                                        + s)
                                                        as f64
                                                })
                                            })
                                        })
                                    })
                                    .ok()
                            } else {
                                None
                            }
                        })
                        .ui(ui);
                    if resp.drag_stopped() {
                        self.save_interval = std::time::Duration::ZERO;
                    }
                });
                if *view == super::BottomPanelConfig::Commits {
                    egui::Frame::menu(ui.style()).show(ui, |ui| {
                        egui::ScrollArea::both().show(ui, |ui| {
                            ui.add(egui::Label::new("---o--------".repeat(50)).extend());
                            ui.add(egui::Label::new("------------".repeat(50)).extend());
                            ui.add(egui::Label::new("-------o----".repeat(50)).extend());
                            ui.add(egui::Label::new("-o----------".repeat(50)).extend());
                        });
                    });
                }
                if *view == super::BottomPanelConfig::CommitsTime {
                    ui.painter().rect_filled(
                        ui.available_rect_before_wrap(),
                        ui.visuals().window_rounding,
                        ui.visuals().widgets.open.bg_fill,
                    );
                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            let timed = true;
                            if timed {
                                self.print_commit_graph_timed(ui);
                            } else {
                                self.print_commit_graph(ui, ctx);
                            }
                        });
                }
            });
    }
}
