use egui::Widget;
use re_ui::{DesignTokens, UiExt};

use crate::app::{
    querying::{self, ComputeConfigQuery},
    show_projects_actions,
    types::{self, Commit, Config, QueriedLang},
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
                egui::ScrollArea::vertical().show(ui, |ui| {
                    self.show_left_panel_views_props(ui);
                    // self.show_left_panel_custom_contents(ui);
                })
            });
    }

    fn show_left_panel_views_props(&mut self, ui: &mut egui::Ui) {
        for tile_id in self.tree.active_tiles() {
            let Some(&pane) = self.tree.tiles.get_pane(&tile_id) else {
                // log::error!("{:?}", tile_id);
                continue;
            };
            let title = self.tabs[pane as usize].title(&self.data);
            use super::re_ui_collapse::SectionCollapsingHeader;
            SectionCollapsingHeader::with_id(ui.id().with(pane), title)
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
                    } else if let super::Tab::TreeAspect = self.tabs[pane as usize] {
                        crate::app::code_aspects::show_config(
                            ui,
                            &mut self.data.aspects,
                            &mut self.data.aspects_result,
                            &self.data.api_addr,
                            self.data.store.clone(),
                        );
                    } else if let super::Tab::TSG = self.tabs[pane as usize] {
                        crate::app::tsg::show_config(ui, &mut self.data.tsg);
                    } else if let super::Tab::Smells = self.tabs[pane as usize] {
                        crate::app::smells::show_config(ui, &mut self.data.smells);
                    } else if let super::Tab::LongTracking = self.tabs[pane as usize] {
                        crate::app::long_tracking::show_config(ui, &mut self.data.long_tracking);
                    }

                    match &self.tabs[pane as usize] {
                        _ => (),
                    };
                });
        }
    }
    fn show_local_query_left_panel(&mut self, ui: &mut egui::Ui, id: u16) {
        let query = &mut self.data.queries[id as usize];
        ui.horizontal(|ui| {
            ui.label("name: ");
            ui.text_edit_singleline(&mut query.name);
        });
        egui::ComboBox::new((ui.id(), "Lang", id), "Lang")
            .selected_text(query.lang.as_str())
            .show_ui(ui, |ui| {
                let v = "Cpp";
                if ui.selectable_label(v == query.lang, v).clicked() {
                    if query.lang != v {
                        query.lang = v.to_string()
                    }
                }
                let v = "Java";
                if ui.selectable_label(v == query.lang, v).clicked() {
                    if query.lang != v {
                        query.lang = v.to_string()
                    }
                }
            });

        let sel_precomp = if let Some(id) = query.precomp.clone() {
            self.data.queries[id as usize].name.to_string()
        } else {
            "<none>".to_string()
        };
        let mut create_q = false;
        egui::ComboBox::new((ui.id(), "Precomp", id), "Precomp")
            .selected_text(sel_precomp)
            .show_ui(ui, |ui| {
                create_q = ui.button("new").clicked();
                let mut precomp = None;
                for (i, q) in self.data.queries.iter().enumerate() {
                    let v = &q.name;
                    if ui.selectable_label(i == id as usize, v).clicked() {
                        if i == id as usize {
                            precomp = Some(i);
                        }
                    }
                }
                let query = &mut self.data.queries[id as usize];
                if let Some(precomp) = precomp {
                    query.precomp = Some(precomp as u16);
                }
                if ui
                    .selectable_label(query.precomp.is_none(), "<none>")
                    .clicked()
                {
                    query.precomp = None;
                }
            });
        if create_q {
                self.data.queries.push(crate::app::QueryData {
                    name: "precomp".to_string(),
                    lang: self.data.queries[id as usize].lang.to_string(),
                    query: egui_addon::code_editor::CodeEditor::new(
                        egui_addon::code_editor::EditorInfo::default().copied(),
                        r#"translation_unit"#.to_string(),
                    ),
                    ..Default::default()
                });
                let qid = self.data.queries.len() as u16 - 1;
                let query = &mut self.data.queries[id as usize];
                query.precomp = Some(qid);
                let tid = self.tabs.len() as u16;
                self.tabs.push(crate::app::Tab::LocalQuery(qid));
                let child = self.tree.tiles.insert_pane(tid);
                match self.tree.tiles.get_mut(self.tree.root.unwrap()) {
                    Some(egui_tiles::Tile::Container(c)) => c.add_child(child),
                    _ => todo!(),
                };
            }
            let query = &mut self.data.queries[id as usize];
            egui::Slider::new(&mut query.commits, 1..=100)
            .text("#commits")
            .clamping(egui::SliderClamping::Never)
            .ui(ui)
            .on_hover_text("Maximum number of commits that will be processed.");
        egui::Slider::new(&mut query.max_matches, 1..=1000)
            .text("match limit")
            .clamping(egui::SliderClamping::Never)
            .ui(ui)
            .on_hover_text("Maximum number of match per commit\n, for any of the patterns.");
        egui::Slider::new(&mut query.timeout, 1..=5000)
            .text("commit timeout")
            .clamping(egui::SliderClamping::Never)
            .ui(ui)
            .on_hover_text("Maximum time to match each commit.");
        ui.add_enabled(
            false,
            egui::Slider::new(&mut query.timeout, 1..=5000)
                .text("commit timeout")
                .clamping(egui::SliderClamping::Never),
        )
        .on_hover_text("Maximum time to match all commit.");
        let compute_button = ui.add_enabled(false, egui::Button::new("Compute All"));
        let q_res_ids = &mut query.results;
        if self.data.selected_code_data.commit_count() != q_res_ids.len() {
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
                        self.data.queries_results.push(super::QueryResults {
                            project: rid,
                            query: id,
                            content: Default::default(),
                            tab: u16::MAX,
                        });
                    }
                    (None, Some(&_id)) => {
                        // nothing to do ProjectIds are valid for the duration of the session
                        // self.data.queries_results[id as usize].0 = u16::MAX;
                    }
                    (Some((i, _)), Some(&id)) => {
                        self.data.queries_results[id as usize].project = i;
                        r.push(id);
                    }
                }
            }
            *q_res_ids = r;
        }
        let query_data = &self.data.queries[id as usize];
        let q_res_ids = &query_data.results;
        ui.style_mut().spacing.item_spacing = egui::vec2(3.0, 2.0);
        for q_res_id in q_res_ids {
            if *q_res_id == u16::MAX {
                continue;
            }
            let q_res = &mut self.data.queries_results[*q_res_id as usize];
            let Some((repo, mut c)) = self.data.selected_code_data.get_mut(q_res.project) else {
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
                q_res.tab = tid;
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
                    let w = &mut ui.style_mut().visuals.widgets;
                    let d = egui::Color32::DARK_GREEN;
                    let _n = w.hovered.weak_bg_fill;
                    let n = egui::Color32::GREEN;
                    w.open.weak_bg_fill = d;
                    w.active.weak_bg_fill = d;
                    w.hovered.weak_bg_fill = n;
                    w.inactive.weak_bg_fill = d;
                    let q_res = &mut self.data.queries_results[*q_res_id as usize];
                    let compute_button;
                    if let Some(content) = q_res.content.get() {
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
                                    } else if rows.1.len() == content.commits {
                                        compute_button =
                                            ui.add(egui::Button::new("↺")).on_hover_ui(|ui| {
                                                ui.label(format!(
                                                    "streamed results {}/{}",
                                                    rows.1.len(),
                                                    content.commits
                                                ));
                                            });
                                    } else {
                                        let rect = ui.spinner().rect;
                                        compute_button = ui
                                            .interact(
                                                rect,
                                                ui.id().with(q_res.project),
                                                egui::Sense::click(),
                                            )
                                            .on_hover_text(format!(
                                                "waiting for the rest of the entries: {}/{}",
                                                rows.1.len(),
                                                content.commits
                                            ));
                                    }
                                } else if rows.1.len() == content.commits {
                                    compute_button =
                                        ui.add(egui::Button::new("↺")).on_hover_ui(|ui| {
                                            ui.label(format!(
                                                "streamed results {}/{}",
                                                rows.1.len(),
                                                content.commits
                                            ));
                                        });
                                } else {
                                    let d = egui::Color32::DARK_GRAY;
                                    let n = egui::Color32::GRAY;
                                    let w = &mut ui.style_mut().visuals.widgets;
                                    w.open.weak_bg_fill = d;
                                    w.active.weak_bg_fill = d;
                                    w.hovered.weak_bg_fill = n;
                                    w.inactive.weak_bg_fill = d;
                                    compute_button =
                                        ui.add(egui::Button::new("⚠")).on_hover_ui(|ui| {
                                            ui.label(format!(
                                                "interupted at {}/{}",
                                                rows.1.len(),
                                                content.commits
                                            ));
                                        });
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
                        if q_res.content.is_waiting() {
                            ui.spinner();
                            let synced = Self::sync_query_results(q_res);
                            if let Ok(true) = synced {
                                self.save_interval = std::time::Duration::ZERO;
                                if q_res.tab == u16::MAX {
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
                            if q_res.tab == u16::MAX {
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
                        let query_data = &self.data.queries[q_res.query as usize];
                        // let current_lang = &query_data.lang;
                        let w = &mut ui.style_mut().visuals.widgets;
                        w.hovered.weak_bg_fill = _n;
                        ui.selectable_label(false, "..").on_hover_ui(|ui| {
                            ui.horizontal(|ui| {
                                ui.label("name:");
                                ui.label(&query_data.name)
                            });
                            ui.horizontal(|ui| {
                                ui.label("lang:");
                                ui.label(&query_data.lang)
                            });
                        });
                        // egui::ComboBox::new(("Lang", q_res.query), "Lang")
                        //     .selected_text(query_data.lang.as_str())
                        //     .show_ui(ui, |ui| {
                        //         lang_selection(ui, current_lang, "Cpp", &mut new_lang, q_res.query);
                        //         lang_selection(
                        //             ui,
                        //             current_lang,
                        //             "Java",
                        //             &mut new_lang,
                        //             q_res.query,
                        //         );
                        //     });
                    };
                    compute_button
                })
                .inner;
            let q_res = &mut self.data.queries_results[*q_res_id as usize];

            if compute_button.clicked() {
                let (repo, mut commit_slice) =
                    self.data.selected_code_data.get_mut(q_res.project).unwrap();
                let query_data = &self.data.queries[q_res.query as usize];
                let language = query_data.lang.to_string();
                let query = query_data.query.as_ref().to_string();
                wasm_rs_dbg::dbg!(&query);
                let commits = query_data.commits as usize;
                let commit = Commit {
                    repo: repo.clone(),
                    id: commit_slice.iter_mut().next().cloned().unwrap(),
                };
                let max_matches = query_data.max_matches;
                let timeout = query_data.timeout;
                let precomp = query_data.precomp.clone().map(|id| &self.data.queries[id as usize]);
                let precomp = precomp.map(|p| p.query.as_ref().to_string());
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
                        precomp,
                    },
                    commit_slice.iter_mut().skip(1).map(|x| x.to_string()),
                );
                q_res.content.buffer(prom);
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
        if q_res.content.is_waiting() {
            if q_res.content.try_poll_with(|x| {
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
                            let n = n as i64;
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
                                    .parse::<i64>()
                                    .and_then(|d| {
                                        parts[1].parse::<i64>().and_then(|h| {
                                            parts[2].parse::<i64>().and_then(|m| {
                                                parts[3].parse::<i64>().map(|s| {
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
                } else if *view == super::BottomPanelConfig::CommitsTime {
                    egui::ScrollArea::both()
                        .auto_shrink([false; 2])
                        .show(ui, |ui| {
                            egui::Frame::menu(ui.style()).show(ui, |ui| {
                                let timed = true;
                                if timed {
                                    self.print_commit_graph_timed(ui);
                                } else {
                                    self.print_commit_graph(ui, ctx);
                                }
                            });
                        });
                }
            });
    }
}

fn lang_selection<'a, T>(
    ui: &mut egui::Ui,
    current_lang: &str,
    selected_value: &'a str,
    new_lang: &mut Option<(&'a str, T)>,
    payload: T,
) {
    let same = current_lang == selected_value;
    if ui.selectable_label(same, selected_value).clicked() {
        if !same {
            *new_lang = Some((selected_value, payload));
        }
    }
}
