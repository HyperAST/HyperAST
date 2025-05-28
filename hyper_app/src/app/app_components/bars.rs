use re_ui::{ContextExt as _, DesignTokens, UiExt as _};

use crate::{app::types, command::UICommand};

use super::CommandSender;

impl crate::HyperApp {
    // #[cfg(target_arch = "wasm32")]
    // fn top_bar(&mut self, egui_ctx: &egui::Context) {}

    // #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn top_bar(&mut self, egui_ctx: &egui::Context) {
        let top_bar_style = egui_ctx.top_bar_style(cfg!(not(target_arch = "wasm32")));

        let mut frame_style = DesignTokens::top_panel_frame();
        if !egui_ctx.style().visuals.dark_mode {
            frame_style.fill = egui::Color32::WHITE; //egui::Visuals::light().window_fill;
            frame_style.stroke = egui::Visuals::light().window_stroke;
            frame_style.stroke.color = egui::Color32::WHITE;
            frame_style.shadow.color = egui::Color32::WHITE; //egui::Visuals::light().window_shadow.color;
        }
        egui::TopBottomPanel::top("top_bar")
            .frame(frame_style)
            .exact_height(top_bar_style.height)
            .show(egui_ctx, |ui| {
                #[cfg(not(target_arch = "wasm32"))]
                if !re_ui::NATIVE_WINDOW_BAR {
                    // Interact with background first, so that buttons in the top bar gets input priority
                    // (last added widget has priority for input).
                    let title_bar_response = ui.interact(
                        ui.max_rect(),
                        ui.id().with("background"),
                        egui::Sense::click(),
                    );
                    if title_bar_response.double_clicked() {
                        let maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::Maximized(!maximized));
                    } else if title_bar_response.is_pointer_button_down_on() {
                        // TODO(emilk): This should probably only run on `title_bar_response.drag_started_by(PointerButton::Primary)`,
                        // see https://github.com/emilk/egui/pull/4656
                        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
                    }
                }

                egui::menu::bar(ui, |ui| {
                    // ui.set_height(top_bar_style.height);
                    ui.add_space(top_bar_style.indent);
                    let rect = ui.available_rect_before_wrap();

                    #[cfg(target_os = "macos")]
                    ui.add_space(60.0);

                    let title = egui::RichText::new("HyperAST").heading().size(15.0);
                    if ui
                        .add(egui::Label::new(title).sense(egui::Sense::click()))
                        .clicked()
                    {
                        if let Some(tid) = self
                            .tabs
                            .iter()
                            .position(|x| x == &super::Tab::MarkdownStatic(0))
                        {
                            let tid = tid as u16;
                            if let Some(child) = self.tree.tiles.find_pane(&tid) {
                                if !self.tree.is_visible(child)
                                    || !self.tree.active_tiles().contains(&child)
                                {
                                    self.tree.set_visible(child, true);
                                    self.tree.move_tile_to_container(
                                        child,
                                        self.tree.root.unwrap(),
                                        0,
                                        true,
                                    );
                                }
                            } else {
                                let child = self.tree.tiles.insert_pane(tid);
                                match self.tree.tiles.get_mut(self.tree.root.unwrap()) {
                                    Some(egui_tiles::Tile::Container(c)) => c.add_child(child),
                                    _ => todo!(),
                                };
                            }
                        } else if self.maximized.is_none() {
                            let tid = self.tabs.len() as u16;
                            self.tabs.push(super::Tab::MarkdownStatic(0));
                            if self.maximized.is_none() {
                                let child = self.tree.tiles.insert_pane(tid);
                                match self.tree.tiles.get_mut(self.tree.root.unwrap()) {
                                    Some(egui_tiles::Tile::Container(c)) => c.add_child(child),
                                    _ => todo!(),
                                };
                            }
                        }
                    }
                    ui.menu_button("File", |ui| file_menu(ui, &self.data.command_sender));
                    egui::warn_if_debug_build(ui);

                    let desired_size = egui::Vec2::new(rect.width() / 3.0, rect.height() * 1.6);
                    let max_rect = egui::Rect::from_center_size(rect.center(), desired_size);
                    let add_contents = |ui: &mut egui::Ui, _rect: egui::Rect| {
                        ui.add_space(50.0);
                        ui.visuals_mut().selection.bg_fill = ui.visuals().widgets.active.bg_fill;
                        ui.visuals_mut().selection.stroke.color =
                            ui.visuals().widgets.active.bg_fill;
                        ui.visuals_mut().selection.stroke.width *= 4.0;
                        for s in <types::SelectedConfig as strum::IntoEnumIterator>::iter() {
                            let text = s.title();
                            let button =
                                egui::Button::new(text.as_ref()).selected(s == self.selected);
                            if ui
                                .add_enabled(s.enabled(), button)
                                .on_disabled_hover_text("WIP layout")
                                .on_hover_ui(|ui| s.on_hover_show(ui))
                                .clicked()
                            {
                                if self.selected != s {
                                    let (tabs, tree) = self
                                        .layouts
                                        .remove(s.title().as_ref())
                                        .unwrap_or_else(|| {
                                            let tabs = s.default_layout();
                                            let tree = egui_tiles::Tree::new_grid(
                                                "my_tree",
                                                (0..tabs.len() as u16).collect(),
                                            );
                                            (tabs, tree)
                                        });
                                    let tabs = std::mem::replace(&mut self.tabs, tabs);
                                    let tree = std::mem::replace(&mut self.tree, tree);
                                    self.layouts
                                        .insert(self.selected.title().into(), (tabs, tree));
                                    self.selected = s;
                                }
                            }
                        }
                        ui.add_space(50.0);
                    };
                    ui.visuals_mut().clip_rect_margin = 0.0;
                    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(max_rect), |ui| {
                        egui::ScrollArea::horizontal()
                            // .horizontal_scroll_offset(max_rect.left() - rect.left() + 50.0)
                            .auto_shrink(false)
                            .hscroll(true)
                            .scroll_bar_visibility(
                                egui::scroll_area::ScrollBarVisibility::AlwaysHidden,
                            )
                            .show_viewport(ui, add_contents);
                    });
                    // let max_rect = max_rect.expand2((5.0, 0.0).into());
                    let (rect, _) = max_rect.split_left_right_at_fraction(0.15);
                    let mut mesh = egui::Mesh::default();
                    mesh.colored_vertex(rect.left_bottom(), frame_style.fill);
                    mesh.colored_vertex(rect.left_top(), frame_style.fill);
                    mesh.colored_vertex(rect.right_bottom(), egui::Color32::TRANSPARENT);
                    mesh.colored_vertex(rect.right_top(), egui::Color32::TRANSPARENT);
                    mesh.add_triangle(0, 1, 2);
                    mesh.add_triangle(1, 2, 3);
                    ui.painter().add(mesh);
                    let rect = egui::Rect::from_min_size(
                        max_rect.right_bottom() - rect.size(),
                        rect.size(),
                    );
                    let mut mesh = egui::Mesh::default();
                    mesh.colored_vertex(rect.left_bottom(), egui::Color32::TRANSPARENT);
                    mesh.colored_vertex(rect.left_top(), egui::Color32::TRANSPARENT);
                    mesh.colored_vertex(rect.right_bottom(), frame_style.fill);
                    mesh.colored_vertex(rect.right_top(), frame_style.fill);
                    mesh.add_triangle(0, 1, 2);
                    mesh.add_triangle(1, 2, 3);
                    ui.painter().add(mesh);

                    ui.add_space(10.0);
                    use crate::command::UICommandSender;
                    // #[cfg(hyperast_experimental)]
                    // if ui
                    //     .add(ui.small_icon_button_widget(&re_ui::icons::ADD))
                    //     .on_hover_text("new blank layout")
                    //     .clicked()
                    // {
                    //     // TODO
                    // }
                    ui.add_space(10.0);
                    if ui.button("💾").on_hover_text("save layout").clicked() {
                        self.data.command_sender.send_ui(UICommand::SaveLayout);
                    }
                    ui.add_space(10.0);
                    if ui.button("↺").on_hover_text("reset layout").clicked() {
                        self.data.command_sender.send_ui(UICommand::ResetLayout);
                    }
                    top_bar_ui(self, ui);
                });
            });
    }

    // #[cfg(not(target_arch = "wasm32"))]
}

fn top_bar_ui(app: &mut crate::HyperApp, ui: &mut egui::Ui) {
    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
        // From right-to-left:

        if re_ui::CUSTOM_WINDOW_DECORATIONS {
            ui.add_space(8.0);
            #[cfg(not(target_arch = "wasm32"))]
            if true {
                ui.native_window_buttons_ui();
            }
            ui.separator();
        } else {
            ui.add_space(16.0);
        }

        re_ui::notifications::notification_toggle_button(ui, &mut app.notifs);
        ui.medium_icon_toggle_button(&re_ui::icons::RIGHT_PANEL_TOGGLE, &mut app.show_right_panel);
        ui.medium_icon_toggle_button(
            &re_ui::icons::BOTTOM_PANEL_TOGGLE,
            &mut app.show_bottom_panel,
        );
        ui.medium_icon_toggle_button(&re_ui::icons::LEFT_PANEL_TOGGLE, &mut app.show_left_panel);
        ui.add_enabled_ui(false, |ui| {
            let mut grid = false;
            ui.medium_icon_toggle_button(&re_ui::icons::CONTAINER_GRID, &mut grid);
            ui.medium_icon_toggle_button(&re_ui::icons::CONTAINER_TABS, &mut grid);
            ui.medium_icon_toggle_button(&re_ui::icons::CONTAINER_HORIZONTAL, &mut grid);
            ui.medium_icon_toggle_button(&re_ui::icons::CONTAINER_VERTICAL, &mut grid);
        });
        egui::global_theme_preference_switch(ui);

        let resp = ui.toggle_value(&mut app.persistance, "persistance");
        if resp.changed() {
            app.save_interval = std::time::Duration::from_secs(0);
        }
    });
}

fn file_menu(ui: &mut egui::Ui, command_sender: &CommandSender) {
    UICommand::SaveResults.menu_button_ui(ui, command_sender);
    UICommand::SaveLayout.menu_button_ui(ui, command_sender);
    #[cfg(not(target_arch = "wasm32"))]
    UICommand::Open.menu_button_ui(ui, command_sender);
    #[cfg(not(target_arch = "wasm32"))]
    UICommand::Quit.menu_button_ui(ui, command_sender);
}
