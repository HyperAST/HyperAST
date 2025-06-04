use crate::app::*;
use commit::CommitSlice;
use re_ui::{DesignTokens, list_item};
use utils_egui::MyUiExt as _;
mod bars;
mod panels;

impl super::HyperApp {
    pub(crate) fn show_actions(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.strong("Actions");
            // TODO show list of groups of actions, ok to type erase, just mut borrow AppData and an ui
            trait AppActions {
                fn ui(self, data: &mut AppData, ui: &mut egui::Ui) -> egui::Response;
            }

            impl<F> AppActions for F
            where
                F: FnOnce(&mut AppData, &mut egui::Ui) -> egui::Response,
            {
                fn ui(self, data: &mut AppData, ui: &mut egui::Ui) -> egui::Response {
                    self(data, ui)
                }
            }

            ui.horizontal_wrapped(|ui| show_projects_actions(ui, &mut self.data))
        });
    }

    pub(crate) fn left_panel_mid_section_ui(&mut self, ui: &mut egui::Ui) {
        re_ui_collapse::SectionCollapsingHeader::new("Config").show(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            // ui.label("Some blueprint stuff here, that might be wide.");
            ui.re_checkbox(&mut self.dummy_bool, "Checkbox");

            // ui.collapsing_header("Collapsing header", true, |ui| {
            //     ui.label("Some data here");
            //     ui.re_checkbox(&mut self.dummy_bool, "Checkbox");
            // });
        });
    }

    pub(crate) fn left_panel_bottom_section_ui(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.label("Toggle switch:");
            ui.toggle_switch(8.0, &mut self.dummy_bool);
        });
        ui.label(format!("Latest command: {}", self.latest_cmd));

        // ---

        // if ui.button("Open modal").clicked() {
        //     self.modal_handler.open();
        // }

        // self.modal_handler.ui(
        //     ui.ctx(),
        //     || re_ui::modal::Modal::new("Modal window"),
        //     |ui, _| ui.label("This is a modal window."),
        // );

        // ---

        // if ui.button("Open full span modal").clicked() {
        //     self.full_span_modal_handler.open();
        // }

        // self.full_span_modal_handler.ui(
        //     ui.ctx(),
        //     || re_ui::modal::Modal::new("Modal window").full_span_content(true),
        //     |ui, _| {
        //         list_item::list_item_scope(ui, "modal demo", |ui| {
        //             for idx in 0..10 {
        //                 list_item::ListItem::new()
        //                     .show_flat(ui, list_item::LabelContent::new(format!("Item {idx}")));
        //             }
        //         });
        //     },
        // );

        ui.horizontal_wrapped(|ui| {
            if ui.button("Log info").clicked() {
                log::info!(
                    "A lot of text on info level.\nA lot of text in fact. So \
                                    much that we should ideally be auto-wrapping it at some point, much \
                                    earlier than this."
                );
            }
            if ui.button("Log warn").clicked() {
                log::warn!(
                    "A lot of text on warn level.\nA lot of text in fact. So \
                                much that we should ideally be auto-wrapping it at some point, much \
                                earlier than this."
                );
            }
            if ui.button("Log error").clicked() {
                log::error!(
                    "A lot of text on error level.\nA lot of text in fact. \
                                So much that we should ideally be auto-wrapping it at some point, much \
                                earlier than this."
                );
            }
        });

        // ---

        re_ui_collapse::SectionCollapsingHeader::new("Data")
            .button(list_item::ItemMenuButton::new(&re_ui::icons::ADD, |ui| {
                ui.weak("empty");
            }))
            .show(ui, |ui| {
                ui.label("Some data here");
            });
        re_ui_collapse::SectionCollapsingHeader::new("Blueprint").show(ui, |ui| {
            ui.style_mut().wrap_mode = Some(egui::TextWrapMode::Extend);
            // ui.label("Some blueprint stuff here, that might be wide.");
            ui.re_checkbox(&mut self.dummy_bool, "Checkbox");

            ui.collapsing_header("Collapsing header", true, |ui| {
                ui.label("Some data here");
                ui.re_checkbox(&mut self.dummy_bool, "Checkbox");
            });
        });

        self.show_repositories(ui);
    }

    fn show_repositories(&mut self, ui: &mut egui::Ui) {
        let label = "Repositories";

        let id = ui.make_persistent_id(label);
        let mut paste_trigered = false;
        let content = list_item::LabelContent::new(label)
            .truncate(true)
            .with_buttons(|ui| {
                let button = egui::Button::new("📋");
                if ui.add_enabled(true, button).clicked() {
                    paste_trigered = true;
                }
                let button = egui::ImageButton::new(
                    re_ui::icons::EXTERNAL_LINK
                        .as_image()
                        .fit_to_exact_size(egui::Vec2::splat(12.0))
                        .tint(ui.visuals().widgets.inactive.fg_stroke.color),
                );
                let resp = ui.add_enabled(true, button);
                if resp.clicked() {
                    self.modal_handler_projects.open();
                }
                resp
            })
            .always_show_buttons(true);

        let force_background = if ui.visuals().dark_mode {
            DesignTokens::load().section_collapsing_header_color()
        } else {
            ui.visuals().widgets.active.bg_fill
        };

        let resp = list_item::list_item_scope(ui, id, |ui| {
            list_item::ListItem::new()
                .interactive(true)
                .force_background(force_background)
                .show_hierarchical_with_children(ui, id, true, content, |ui| {
                    //TODO(ab): this space is not desirable when the content actually is list items
                    ui.add_space(4.0); // Add space only if there is a body to make minimized headers stick together.
                    for i in self.data.selected_code_data.project_ids() {
                        let Some((r, commits)) = self.data.selected_code_data.get_mut(i) else {
                            continue;
                        };
                        Self::show_repo_item(commits, r, ui, ui.id().with(i));
                    }
                    ui.add_space(4.0); // Same here
                })
        });

        if resp.item_response.clicked() {
            // `show_hierarchical_with_children_unindented` already toggles on double-click,
            // but we are _only_ a collapsing header, so we should also toggle on normal click:
            if let Some(mut state) = egui::collapsing_header::CollapsingState::load(ui.ctx(), id) {
                state.toggle(ui);
                state.store(ui.ctx());
            }
        };

        if let Some(paste) =
            utils::prepare_paste(ui, paste_trigered, &mut self.capture_clip_into_repos)
        {
            self.capture_clip_into_repos = false;
            if paste.contains("\n") {
                let mut acc = vec![];
                let mut bad = 0;
                for paste in paste.split("\n") {
                    match commit::validate_pasted_project_url(&paste) {
                        Ok(x) => {
                            acc.push(Ok(x));
                        }
                        Err(err) => {
                            bad += 1;
                            log::warn!("Wrong input from clipboard: {}:\n{}", err, paste);
                            acc.push(Err(format!(
                                "Wrong input from clipboard: {}:\n{}",
                                err, paste
                            )));
                        }
                    }
                }
                if bad == 0 {
                    for s in acc.chunks(5) {
                        let text: String = s
                            .into_iter()
                            .filter_map(|x| x.as_ref().ok())
                            .map(|(r, cs)| {
                                if cs.is_empty() {
                                    format!("github.com/{}/{}", r.user, r.name)
                                } else {
                                    cs.iter()
                                        .map(|c| {
                                            format!("\n\tgithub.com/{}/{}/{}", r.user, r.name, c)
                                        })
                                        .collect()
                                }
                            })
                            .collect();
                        self.notifs.success(format!("Succesfully added: {}", text));
                    }
                    for x in acc {
                        let (repo, commits) = x.unwrap();
                        self.data.selected_code_data.add(repo, commits);
                    }
                } else if bad == acc.len() {
                    self.notifs.add_log(re_log::LogMsg {
                        level: log::Level::Error,
                        target: format!("clipboard"),
                        msg: format!("Wrong input from clipboard:\n{}", paste),
                    });
                // } else if bad <= 2 && bad * 4 <= acc.len() { // TODO later if annoying
                } else {
                    let good: Vec<_> = acc
                        .into_iter()
                        .enumerate()
                        .filter_map(|(i, x)| x.ok().map(|_| i))
                        .collect();
                    self.notifs.add_log(re_log::LogMsg {
                        level: log::Level::Error,
                        target: format!("clipboard"),
                        msg: format!(
                            "{bad} Wrong inputs from clipboard but {:?} could be accepted:\n{}",
                            good, paste
                        ),
                    });
                }
            } else {
                let result = commit::validate_pasted_project_url(&paste);
                match result {
                    Ok((repo, commits)) => {
                        self.notifs.success(if commits.is_empty() {
                            format!("Successfully added github.com/{}/{}", repo.user, repo.name)
                        } else if commits.len() == 1 {
                            format!(
                                "Successfully added github.com/{}/{}/{}",
                                repo.user, repo.name, commits[0]
                            )
                        } else {
                            let commits: String =
                                commits.iter().map(|c| format!("\n{}", c)).collect();
                            format!(
                                "Successfully added github.com/{}/{}{}",
                                repo.user, repo.name, commits
                            )
                        });
                        self.data.selected_code_data.add(repo, commits);
                    }
                    Err(err) => {
                        log::warn!("Wrong input from clipboard: {}:\n{}", err, paste);
                        self.notifs.add_log(re_log::LogMsg {
                            level: log::Level::Warn,
                            target: format!("clipboard"),
                            msg: format!("Wrong input from clipboard: {}:\n{}", err, paste),
                        });
                    }
                }
            }
        }

        self.modal_handler_projects.ui(
            ui.ctx(),
            || re_ui::modal::ModalWrapper::new("Project Selection"),
            |ui, _| {
                ui.push_id(ui.id().with("modal projects"), |ui| {
                    show_project_selection(ui, &mut self.data)
                })
            },
        );
    }

    fn show_repo_item(
        mut commits: CommitSlice<'_>,
        r: &mut Repo,
        ui: &mut egui::Ui,
        id: egui::Id,
    ) -> list_item::ShowCollapsingResponse<()> {
        let button_menu = |ui: &mut egui::Ui| {
            let popup_id = ui.make_persistent_id("add_commit");
            let button = egui::Button::new("commit");
            let button = &ui.add_enabled(true, button);
            if button.clicked() {
                commits.push(Default::default());
                ui.memory_mut(|mem| mem.open_popup(popup_id))
            }
            let mut close_menu = false;
            egui::popup::popup_above_or_below_widget(
                ui,
                popup_id,
                button,
                egui::AboveOrBelow::Below,
                egui::popup::PopupCloseBehavior::CloseOnClickOutside,
                |ui| {
                    let text = commits.last_mut().unwrap();
                    let singleline = &ui.text_edit_singleline(text);
                    if button.clicked() {
                        singleline.request_focus()
                    }
                    if singleline.lost_focus() {
                        if text.is_empty() {
                            commits.pop();
                        }
                        ui.memory_mut(|mem| mem.close_popup());
                        close_menu = true;
                    }
                },
            );
            if close_menu {
                ui.close_menu();
            }

            let button = egui::Button::new("branch");
            if ui.add_enabled(false, button).clicked() {
                // TODO
                wasm_rs_dbg::dbg!("TODO add branch");
            }
            let button = egui::Button::new("commit range");
            if ui.add_enabled(false, button).clicked() {
                // TODO
                wasm_rs_dbg::dbg!("TODO add commit range");
            }
        };
        let label = format!("github.com/{}/{}", r.user, r.name);

        let label = egui::RichText::new(label)
            .size(10.0)
            .line_height(Some(10.0));

        let content = list_item::LabelContent::new(label)
            .with_buttons(|ui| {
                ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
                let button = egui::Button::image(
                    re_ui::icons::ADD
                        .as_image()
                        .fit_to_exact_size(egui::Vec2::splat(12.0)),
                );
                egui::menu::menu_custom_button(ui, button, button_menu).response
            })
            .always_show_buttons(true);

        let force_background = if ui.visuals().dark_mode {
            DesignTokens::load().section_collapsing_header_color()
        } else {
            ui.visuals().widgets.active.weak_bg_fill
        }
        .gamma_multiply(0.6);

        let default_open = true;

        let list = list_item::ListItem::new()
            .interactive(true)
            .force_background(force_background)
            .with_height(16.0);
        let response = list_item::list_item_scope(ui, id, |ui| {
            list.show_hierarchical_with_children(ui, id, default_open, content, |_| ())
        });
        let mut add_children = |ui: &mut egui::Ui| {
            ui.add_space(4.0);
            list_item::list_item_scope(ui, ui.id().with("commits"), |ui| {
                let mut rm = None;
                for (i, oid) in commits.iter_mut().enumerate() {
                    let text = egui::RichText::new(oid.as_str())
                        .size(8.0)
                        .line_height(Some(8.0));
                    let buttons = |ui: &mut egui::Ui| {
                        let resp = ui.add(
                            egui::ImageButton::new(
                                re_ui::icons::REMOVE
                                    .as_image()
                                    .fit_to_exact_size(egui::Vec2::splat(10.0)),
                            )
                            .tint(ui.visuals().widgets.inactive.fg_stroke.color),
                        );
                        if resp.clicked() {
                            rm = Some(i);
                        }
                        resp
                    };
                    let content = list_item::LabelContent::new(text).with_buttons(buttons);
                    let resp = list_item::ListItem::new()
                        .with_height(10.0)
                        .show_flat(ui, content);
                    let popup_id = ui.make_persistent_id(format!("change_commit {i}"));
                    if resp.clicked() {
                        ui.memory_mut(|mem| mem.open_popup(popup_id))
                    }

                    let mut close_menu = false;
                    egui::popup::popup_above_or_below_widget(
                        ui,
                        popup_id,
                        &resp,
                        egui::AboveOrBelow::Below,
                        egui::popup::PopupCloseBehavior::CloseOnClickOutside,
                        |ui| {
                            let singleline = &ui.text_edit_singleline(oid);
                            if resp.clicked() {
                                singleline.request_focus()
                            }
                            if singleline.lost_focus() {
                                ui.memory_mut(|mem| mem.close_popup());
                                close_menu = true;
                            }
                        },
                    );
                    if close_menu {
                        ui.close_menu();
                    }
                }
                if let Some(j) = rm {
                    ui.memory_mut(|mem| mem.close_popup());
                    commits.remove(j);
                }
            });
            ui.add_space(6.0);
        };
        let mut state = egui::collapsing_header::CollapsingState::load(ui.ctx(), id).unwrap();
        let indented = true;
        let mut span = ui.full_span().shrink(8.0);
        span.min = span.max.min(span.min + 10.0);
        let body_response = ui.full_span_scope(span, |ui| {
            if indented {
                ui.spacing_mut().indent =
                    DesignTokens::small_icon_size().x + DesignTokens::text_to_icon_padding();
                state.show_body_indented(&response.item_response, ui, |ui| add_children(ui))
            } else {
                state.show_body_unindented(ui, |ui| add_children(ui))
            }
        });

        if response.item_response.clicked() {
            // `show_hierarchical_with_children_unindented` already toggles on double-click,
            // but we are _only_ a collapsing header, so we should also toggle on normal click:
            if let Some(mut state) = egui::collapsing_header::CollapsingState::load(ui.ctx(), id) {
                state.toggle(ui);
                state.store(ui.ctx());
            }
        }
        response
    }
}

impl super::HyperApp {
    /// Show recent text log messages to the user as toast notifications.
    pub fn show_text_logs_as_notifications(&mut self) {
        // while let Ok(re_log::LogMsg {
        //     level,
        //     target: _,
        //     msg,
        // }) = self.text_log_rx.try_recv()
        // {
        //     let kind = match level {
        //         re_log::Level::Error => toasts::ToastKind::Error,
        //         re_log::Level::Warn => toasts::ToastKind::Warning,
        //         re_log::Level::Info => toasts::ToastKind::Info,
        //         re_log::Level::Debug | re_log::Level::Trace => {
        //             continue; // too spammy
        //         }
        //     };

        //     self.toasts.add(re_log::LogMsg {
        //         kind,
        //         text: msg,
        //         options: toasts::ToastOptions::with_ttl_in_seconds(4.0),
        //     });
        // }
    }
}

use super::AppData;

impl super::HyperApp {
    pub(crate) fn old_ui(&mut self, ctx: &egui::Context) {
        if false {
            let mut trigger_compute = false;
            let Self {
                selected,
                data:
                    AppData {
                        api_addr,
                        scripting_context,
                        querying_context,
                        tsg_context,
                        smells_context,
                        languages: _,
                        single,
                        query,
                        tsg,
                        smells,
                        multi,
                        diff,
                        tracking,
                        aspects,
                        compute_single_result,
                        querying_result,
                        tsg_result,
                        smells_result,
                        smells_diffs_result,
                        fetched_files,
                        tracking_result,
                        aspects_result,
                        store,
                        long_tracking,
                        ..
                    },
                ..
            } = self;
            egui::SidePanel::left("side_panel")
                .width_range(
                    ctx.available_rect().width() * 0.1..=ctx.available_rect().width() * 0.9,
                )
                .show(ctx, |ui| {
                    ui.vertical_centered(|ui| {
                        ui.label(
                            egui::RichText::new("Processing API for HyperAST")
                                .heading()
                                .size(25.0),
                        );
                    });
                    egui::widgets::global_theme_preference_switch(ui);

                    egui::ScrollArea::vertical().show(ui, |ui| {
                        ui.add_space(20.0);
                        show_menu(ui, selected, &single_repo::WANTED, |ui| {
                            single_repo::show_config(ui, single)
                        });
                        ui.separator();
                        show_menu(ui, selected, &querying::WANTED, |ui| {
                            querying::show_config(ui, query)
                        });
                        ui.separator();
                        show_menu(ui, selected, &tsg::WANTED, |ui| tsg::show_config(ui, tsg));
                        ui.separator();
                        show_menu(ui, selected, &smells::WANTED, |ui| {
                            smells::show_config(ui, smells)
                        });

                        ui.separator();
                        ui.add_enabled_ui(false, |ui| {
                            show_multi_repo(ui, selected, multi);
                            ui.wip(Some(" soon available"));
                        });
                        ui.separator();
                        ui.add_enabled_ui(false, |ui| {
                            show_diff_menu(ui, selected, diff);
                            ui.wip(Some(" soon available"));
                        });
                        ui.separator();
                        // ui.add_enabled_ui(false, |ui| {
                        show_menu(ui, selected, &code_tracking::WANTED, |ui| {
                            code_tracking::show_config(ui, tracking, tracking_result)
                        });
                        // ui.wip(Some(" soon available"));
                        // });
                        ui.separator();
                        show_menu(ui, selected, &long_tracking::WANTED, |ui| {
                            long_tracking::show_config(ui, long_tracking)
                        });
                        ui.separator();
                        show_menu(ui, selected, &code_aspects::WANTED, |ui| {
                            code_aspects::show_config(
                                ui,
                                aspects,
                                aspects_result,
                                &api_addr,
                                store.clone(),
                            )
                        });
                    });

                    ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                        ui.horizontal(|ui| {
                            ui.spacing_mut().item_spacing.x = 0.0;
                            ui.label("powered by ");
                            ui.hyperlink_to("egui", "https://github.com/emilk/egui");
                            ui.label(" and ");
                            ui.hyperlink_to(
                                "eframe",
                                "https://github.com/emilk/egui/tree/master/crates/eframe",
                            );
                            ui.label(".");
                        });
                    });
                });
            // TODO change layout with window ratio
            // ui.ctx().screen_rect()
            if *selected == types::SelectedConfig::Single {
                egui::CentralPanel::default().show(ctx, |ui| {
                    single_repo::show_single_repo(
                        ui,
                        api_addr,
                        single,
                        scripting_context,
                        &mut trigger_compute,
                        compute_single_result,
                    );
                });
            } else if *selected == types::SelectedConfig::Querying {
                egui::CentralPanel::default().show(ctx, |ui| {
                    querying::show_querying(
                        ui,
                        api_addr,
                        query,
                        querying_context,
                        &mut trigger_compute,
                        querying_result,
                    );
                });
            } else if *selected == types::SelectedConfig::Tsg {
                egui::CentralPanel::default().show(ctx, |ui| {
                    tsg::show_querying(
                        ui,
                        api_addr,
                        tsg,
                        tsg_context,
                        &mut trigger_compute,
                        tsg_result,
                    );
                });
            } else if *selected == types::SelectedConfig::Smells {
                egui::CentralPanel::default().show(ctx, |ui| {
                    smells::show_central_panel(
                        ui,
                        api_addr,
                        smells,
                        smells_context,
                        &mut trigger_compute,
                        smells_result,
                        smells_diffs_result,
                        fetched_files,
                    );
                });
            } else if *selected == types::SelectedConfig::Tracking {
                egui::CentralPanel::default().show(ctx, |ui| {
                    code_tracking::show_code_tracking_results(
                        ui,
                        &api_addr,
                        tracking,
                        tracking_result,
                        fetched_files,
                        ctx,
                    );
                });
            } else if *selected == types::SelectedConfig::LongTracking {
                egui::CentralPanel::default()
                    .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(2.0))
                    .show(ctx, |ui| {
                        long_tracking::show_results(
                            ui,
                            &api_addr,
                            aspects,
                            store.clone(),
                            long_tracking,
                            fetched_files,
                        );
                    });
            } else if *selected == types::SelectedConfig::Aspects {
                egui::CentralPanel::default().show(ctx, |ui| {
                    if let Some(aspects_result) = aspects_result {
                        code_aspects::show(aspects_result, ui, api_addr, aspects);
                    } else {
                        // *aspects_result = Some(code_aspects::remote_fetch_tree(
                        //     ctx,
                        //     &aspects.commit,
                        //     &aspects.path,
                        // ));
                        *aspects_result = Some(code_aspects::remote_fetch_node_old(
                            ctx,
                            &api_addr,
                            store.clone(),
                            &aspects.commit,
                            &aspects.path,
                        ));
                    }
                });
            }

            if trigger_compute {
                if *selected == types::SelectedConfig::Single {
                    self.data.compute_single_result = Some(single_repo::remote_compute_single(
                        ctx,
                        api_addr,
                        &mut single.content,
                        scripting_context,
                    ));
                } else if *selected == types::SelectedConfig::Querying {
                    self.data.querying_result = Some(querying::remote_compute_query(
                        ctx,
                        api_addr,
                        query,
                        querying_context,
                    ));
                } else if *selected == types::SelectedConfig::Tsg {
                    self.data.tsg_result =
                        Some(tsg::remote_compute_query(ctx, api_addr, tsg, tsg_context));
                }
            }
        }
    }
}

fn show_multi_repo(
    ui: &mut egui::Ui,
    selected: &mut types::SelectedConfig,
    _multi: &mut types::ComputeConfigMulti,
) {
    let wanted = types::SelectedConfig::Multi;
    let add_body = |ui: &mut egui::Ui| {
        ui.text_edit_singleline(&mut "github.com/INRIA/spoon");
    };
    show_menu(ui, selected, &wanted, add_body);
}

fn show_diff_menu(
    ui: &mut egui::Ui,
    selected: &mut types::SelectedConfig,
    _diff: &mut types::ComputeConfigDiff,
) {
    let wanted = types::SelectedConfig::Diff;
    let add_body = |ui: &mut egui::Ui| {
        ui.text_edit_singleline(&mut "github.com/INRIA/spoon");
    };
    show_menu(ui, selected, &wanted, add_body);
}

pub(crate) fn show_repo_menu(ui: &mut egui::Ui, repo: &mut Repo) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let user_id = ui.next_auto_id().with("user");
        let name_id = ui.next_auto_id().with("name");
        ui.push_id("user", |ui| {
            ui.label("github.com/"); // efwserfwefwe/fewefwse
            let events = ui.input(|i| i.events.clone()); // avoid dead-lock by cloning. TODO(emilk): optimize
            for event in &events {
                match event {
                    egui::Event::Paste(text_to_insert) => {
                        if !text_to_insert.is_empty() {
                            // let mut ccursor = delete_selected(text, &cursor_range);
                            // insert_text(&mut ccursor, text, text_to_insert);
                            // Some(CCursorRange::one(ccursor))
                        }
                    }
                    _ => (),
                };
            }
            if egui::TextEdit::singleline(&mut repo.user)
                .margin(egui::Vec2::new(0.0, 0.0))
                .desired_width(40.0)
                .id(user_id)
                .show(ui)
                .response
                .changed()
            {
                let mut user = None;
                let mut name = None;
                match repo.user.split_once("/") {
                    Some((a, "")) => {
                        user = Some(a.to_string());
                    }
                    Some((a, b)) => {
                        user = Some(a.to_string());
                        name = Some(b.to_string());
                    }
                    None => (),
                }
                if let Some(user) = user {
                    changed |= repo.user != user;
                    repo.user = user;
                    ui.memory_mut(|mem| {
                        mem.surrender_focus(user_id);
                        mem.request_focus(name_id)
                    });
                }
                if let Some(name) = name {
                    changed |= repo.name != name;
                    repo.name = name;
                }
            }
        });
        // 62a2b556c26f0f42a2ae791a86dc39dd36d35392
        if ui
            .push_id("name", |ui| {
                ui.label("/");
                egui::TextEdit::singleline(&mut repo.name)
                    .clip_text(true)
                    .desired_width(40.0)
                    .desired_rows(1)
                    .hint_text("name")
                    .id(name_id)
                    .interactive(true)
                    .show(ui)
            })
            .inner
            .response
            .changed()
        {
            changed |= true;
        };
    });
    changed
}

pub fn show_menu<R>(
    ui: &mut egui::Ui,
    selected: &mut types::SelectedConfig,
    wanted: &types::SelectedConfig,
    add_body: impl FnOnce(&mut egui::Ui) -> R,
) -> egui::CollapsingResponse<R> {
    let title = selected.title();
    let id = ui.make_persistent_id(title.as_ref());
    radio_collapsing(ui, id, title, selected, &wanted, add_body)
}
