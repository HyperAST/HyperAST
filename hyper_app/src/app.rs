use std::{
    collections::{hash_map, HashMap},
    fmt::Debug,
    sync::Arc,
};

use serde::{Deserialize, Serialize};

use self::{
    egui_utils::{radio_collapsing, show_wip},
    single_repo::ComputeConfigSingle,
    types::{Lang, Languages, Repo},
};

mod code_editor;
mod code_tracking;
mod single_repo;
mod syntax_highlighting;
mod ts_highlight;
pub(crate) mod types;
// mod split_from_side_panel;
mod egui_utils;
mod interactive_split;
mod long_tracking;
mod multi_split;
mod split;
mod utils;

// const API_URL: &str = "http://131.254.13.72:8080";
const API_URL: &str = "http://0.0.0.0:8080";

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct HyperApp {
    // Example stuff:
    project_name: String,

    code_editors: types::CodeEditors,

    #[serde(skip)]
    languages: Arc<HashMap<String, Lang>>,

    selected: types::SelectedConfig,
    single: ComputeConfigSingle,
    multi: types::ComputeConfigMulti,
    diff: types::ComputeConfigDiff,
    tracking: types::ComputeConfigTracking,
    aspects: types::ComputeConfigAspectViews,

    #[serde(skip)]
    compute_single_result: Option<single_repo::RemoteResult>,

    #[serde(skip)]
    fetched_files: HashMap<types::FileIdentifier, code_tracking::RemoteFile>,
    #[serde(skip)]
    tracking_result: Buffered<code_tracking::RemoteResult>,
    #[serde(skip)]
    aspects_result: Option<code_aspects::RemoteView>,

    long_tracking: long_tracking::LongTacking,
}

#[derive(Default, Serialize, Deserialize)]
pub enum Buffered<T: std::marker::Send + 'static> {
    #[default]
    Empty,
    #[serde(skip)]
    Init(poll_promise::Promise<T>),
    Single(T),
    #[serde(skip)]
    Waiting {
        content: T,
        waiting: poll_promise::Promise<T>,
    },
}

impl<T: Debug + std::marker::Send + 'static> Debug for Buffered<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => write!(f, "Empty"),
            Self::Init(waiting) => f
                .debug_tuple("Init")
                .field(&waiting.ready().is_none())
                .finish(),
            Self::Single(content) => f.debug_tuple("Single").field(content).finish(),
            Self::Waiting { content, waiting } => f
                .debug_struct("Waiting")
                .field("content", content)
                .field("waiting", &waiting.ready().is_none())
                .finish(),
        }
    }
}

impl<T: std::marker::Send + 'static> Buffered<T> {
    pub fn try_poll(&mut self) -> bool {
        let this = std::mem::take(self);
        let (changed, new) = match this {
            Buffered::Init(waiting) => match waiting.try_take() {
                Ok(ready) => (true, Buffered::Single(ready)),
                Err(waiting) => (false, Buffered::Init(waiting)),
            },
            Buffered::Waiting { waiting, content } => match waiting.try_take() {
                Ok(ready) => (true, Buffered::Single(ready)),
                Err(waiting) => (false, Buffered::Waiting { content, waiting }),
            },
            Buffered::Empty => (false, Buffered::Empty),
            Buffered::Single(content) => (false, Buffered::Single(content)),
        };
        *self = new;
        changed
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        match self {
            Buffered::Empty | Buffered::Init(_) => None,
            Buffered::Single(content) | Buffered::Waiting { content, .. } => Some(content),
        }
    }

    pub fn is_waiting(&self) -> bool {
        match self {
            Buffered::Init(_) | Buffered::Waiting { .. } => true,
            _ => false,
        }
    }

    pub fn buffer(&mut self, waiting: poll_promise::Promise<T>) {
        let this = std::mem::take(self);
        *self = match this {
            Buffered::Empty => Buffered::Init(waiting),
            Buffered::Init(waiting) => Buffered::Init(waiting),
            Buffered::Single(content) => Buffered::Waiting { content, waiting },
            Buffered::Waiting {
                content,
                waiting: _,
            } => {
                // cancel old promise ?
                Buffered::Waiting { content, waiting }
            }
        };
    }

    pub fn take(&mut self) -> Option<T> {
        let this = std::mem::take(self);
        let (content, rest) = match this {
            Buffered::Waiting { waiting, content } => (Some(content), Buffered::Init(waiting)),
            Buffered::Single(content) => (Some(content), Buffered::Empty),
            x => (None, x),
        };
        *self = rest;
        content
    }
}

impl Default for HyperApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            project_name: "Simple Computation".to_owned(),
            code_editors: Default::default(),
            languages: Default::default(),
            single: Default::default(),
            selected: Default::default(),
            diff: Default::default(),
            multi: Default::default(),
            tracking: Default::default(),
            compute_single_result: Default::default(),
            fetched_files: Default::default(),
            tracking_result: Default::default(),
            aspects: Default::default(),
            aspects_result: Default::default(),
            long_tracking: Default::default(),
        }
    }
}

impl HyperApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, languages: Languages) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        dbg!();

        // // Load previous app state (if any).
        // // Note that you must enable the `persistence` feature for this to work.
        // if let Some(storage) = cc.storage {
        //     let mut r: TemplateApp = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        //     if r.code_editor.lang.is_none() {
        //         r.code_editor.lang = languages.get("JavaScript").cloned();
        //     }
        //     if r.languages.is_empty() {
        //         r.languages = languages.into();
        //     }
        //     return r;
        // }

        // parsed.walk().node().kind();

        let mut r = HyperApp::default();
        r.code_editors.init.lang = languages.get("JavaScript").cloned();
        r.code_editors.filter.lang = languages.get("JavaScript").cloned();
        r.code_editors.accumulate.lang = languages.get("JavaScript").cloned();
        // dbg!(&r.code_editors.lang);
        // assert!(r.code_editors.lang.is_some());
        r.languages = languages.into();
        // r.code_editor.parser
        //     .set_language(&lang.into())
        //     .expect("Error loading Java grammar");
        // let parsed = r.code_editor.parser.parse(code, None).unwrap();
        // r.code_editor.parsed = parsed;
        r
    }
}

impl eframe::App for HyperApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        // eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // self.frame_history
        //     .on_new_frame(ctx.input(|i| i.time), frame.info().cpu_usage);
        ctx.request_repaint_after(std::time::Duration::from_secs_f32(5.0));
        let Self {
            project_name,
            code_editors,
            languages,
            selected,
            single,
            multi,
            diff,
            tracking,
            aspects,
            compute_single_result,
            fetched_files,
            tracking_result,
            aspects_result,
            long_tracking,
        } = self;
        let mut trigger_compute = false;

        #[cfg(not(target_arch = "wasm32"))] // no File->Quit on web pages!
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        _frame.close();
                    }
                });
            });
        });

        egui::SidePanel::left("side_panel")
            .width_range(ctx.available_rect().width() * 0.1..=ctx.available_rect().width() * 0.9)
            .show(ctx, |ui| {
                ui.vertical_centered(|ui| {
                    ui.label(
                        egui::RichText::new("Processing API for HyperAST")
                            .heading()
                            .size(25.0),
                    );
                });
                egui::widgets::global_dark_light_mode_switch(ui);

                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add_space(20.0);
                    single_repo::show_single_repo_menu(ui, selected, single);
                    ui.separator();

                    ui.add_enabled_ui(false, |ui| {
                        show_multi_repo(ui, selected, multi);
                        show_wip(ui, Some(" soon available"));
                    });
                    ui.separator();
                    ui.add_enabled_ui(false, |ui| {
                        show_diff(ui, selected, diff);
                        show_wip(ui, Some(" soon available"));
                    });
                    ui.separator();
                    // ui.add_enabled_ui(false, |ui| {
                    code_tracking::show_code_tracking_menu(ui, selected, tracking, tracking_result);
                    // show_wip(ui, Some(" soon available"));
                    // });
                    ui.separator();
                    long_tracking::show_menu(ui, selected, long_tracking);
                    ui.separator();
                    code_aspects::show_aspects_views_menu(ui, selected, aspects, aspects_result);
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
                let is_portrait = ui.available_rect_before_wrap().aspect_ratio() < 1.0;
                if is_portrait {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        egui::warn_if_debug_build(ui);
                        code_editors.init.ui(ui);
                        code_editors.filter.ui(ui);
                        code_editors.accumulate.ui(ui);
                        ui.horizontal(|ui| {
                            if ui.add(egui::Button::new("Compute")).clicked() {
                                trigger_compute |= true;
                            };
                            single_repo::show_short_result(&*compute_single_result, ui);
                        });
                        single_repo::show_long_result(&*compute_single_result, ui);
                    });
                } else {
                    interactive_split::Splitter::vertical()
                        .ratio(0.7)
                        .show(ui, |ui1, ui2| {
                            ui1.push_id(ui1.id().with("input"), |ui| {
                                egui::warn_if_debug_build(ui);
                                code_editors.init.ui(ui);
                                code_editors.filter.ui(ui);
                                code_editors.accumulate.ui(ui);
                            });
                            let ui = ui2;
                            ui.horizontal(|ui| {
                                if ui.add(egui::Button::new("Compute")).clicked() {
                                    trigger_compute |= true;
                                };
                                single_repo::show_short_result(&*compute_single_result, ui);
                            });
                            single_repo::show_long_result(&*compute_single_result, ui);
                        });
                }
            });
        } else if *selected == types::SelectedConfig::Tracking {
            egui::CentralPanel::default().show(ctx, |ui| {
                code_tracking::show_code_tracking_results(
                    ui,
                    tracking,
                    tracking_result,
                    fetched_files,
                    ctx,
                );
            });
        } else if *selected == types::SelectedConfig::LongTracking {
            egui::CentralPanel::default().show(ctx, |ui| {
                long_tracking::show_results(ui, long_tracking, fetched_files);
            });
        } else if *selected == types::SelectedConfig::Aspects {
            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(aspects_result) = aspects_result {
                    if let Some(aspects_result) = aspects_result.ready() {
                        match aspects_result {
                            Ok(aspects_result) => {
                                egui::ScrollArea::both().show(ui, |ui| {
                                    if let Some(content) = &aspects_result.content {
                                        content.show(ui);
                                    }
                                });
                                // egui::CollapsingHeader::new("Tree")
                                //     .default_open(false)
                                //     .show(ui, |ui| {
                                //         // aspects_result.ui(ui)
                                //         if let Some(content) = &aspects_result.content {
                                //             content.show(ui);
                                //         }
                                //     });
                            }
                            Err(err) => {
                                wasm_rs_dbg::dbg!(err);
                            }
                        }
                    }
                } else {
                    *aspects_result = Some(code_aspects::remote_fetch_tree(
                        ctx,
                        &aspects.commit,
                        &aspects.path,
                    ));
                }
            });
        }

        if false {
            egui::Window::new("Window").show(ctx, |ui| {
                ui.label("Windows can be moved by dragging them.");
                ui.label("They are automatically sized based on contents.");
                ui.label("You can turn on resizing and scrolling if you like.");
                ui.label("You would normally choose either panels OR windows.");
            });
        }

        if trigger_compute {
            self.compute_single_result = Some(single_repo::remote_compute_single(
                ctx,
                single,
                code_editors,
            ));
        }
    }
}

fn show_remote_code(
    ui: &mut egui::Ui,
    commit: &mut types::Commit,
    file_path: &mut String,
    file_result: hash_map::Entry<'_, types::FileIdentifier, code_tracking::RemoteFile>,
    // ctx: &egui::Context,
) -> (
    egui::Response,
    egui::InnerResponse<()>,
    std::option::Option<
        egui::InnerResponse<
            Option<egui::scroll_area::ScrollAreaOutput<egui::text_edit::TextEditOutput>>,
        >,
    >,
) {
    egui::ScrollArea::horizontal()
        .show(ui, |ui| {
            show_remote_code2(ui, commit, file_path, file_result, f32::INFINITY, false)
        })
        .inner
}

fn show_remote_code2(
    ui: &mut egui::Ui,
    commit: &mut types::Commit,
    file_path: &mut String,
    file_result: hash_map::Entry<'_, types::FileIdentifier, code_tracking::RemoteFile>,
    desired_width: f32,
    wrap: bool,
) -> (
    egui::Response,
    egui::InnerResponse<()>,
    std::option::Option<
        egui::InnerResponse<
            Option<egui::scroll_area::ScrollAreaOutput<egui::text_edit::TextEditOutput>>,
        >,
    >,
) {
    let mut upd_src = false;
    egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        ui.next_auto_id(),
        true,
    )
    .show_header(ui, |ui| {
        upd_src = ui.text_edit_singleline(file_path).lost_focus();
        let url = format!(
            "{}/{}/{}/blob/{}/{}",
            "https://github.com", &commit.repo.user, &commit.repo.name, &commit.id, &file_path,
        );
        ui.hyperlink_to("see in github", url);
    })
    .body_unindented(|ui| {
        ui.add_space(4.0);
        if let hash_map::Entry::Occupied(promise) = &file_result {
            let promise = promise.get();
            let resp = if let Some(result) = promise.ready() {
                match result {
                    Ok(resource) => {
                        // ui_resource(ui, resource);
                        if let Some(text) = &resource.content {
                            let code: &str = &text.content;
                            let language = "java";
                            let theme = egui_demo_lib::syntax_highlighting::CodeTheme::from_memory(
                                ui.ctx(),
                            );

                            let mut layouter = |ui: &egui::Ui, string: &str, wrap_width: f32| {
                                let mut layout_job = egui_demo_lib::syntax_highlighting::highlight(
                                    ui.ctx(),
                                    &theme,
                                    string,
                                    language,
                                );
                                if wrap {
                                    layout_job.wrap.max_width = wrap_width;
                                }
                                ui.fonts(|f| f.layout_job(layout_job))
                            };
                            Some(egui::ScrollArea::both().show(ui, |ui| {
                                egui::TextEdit::multiline(&mut code.to_string())
                                    .font(egui::FontId::new(10.0, egui::FontFamily::Monospace)) // for cursor height
                                    .code_editor()
                                    .desired_rows(10)
                                    // .desired_width(100.0)
                                    .desired_width(desired_width)
                                    .clip_text(true)
                                    .lock_focus(true)
                                    .layouter(&mut layouter)
                                    .show(ui)
                            }))
                        } else {
                            None
                        }
                    }
                    Err(error) => {
                        // This should only happen if the fetch API isn't available or something similar.
                        ui.colored_label(
                            ui.visuals().error_fg_color,
                            if error.is_empty() { "Error" } else { error },
                        );
                        None
                    }
                }
            } else {
                ui.spinner();
                None
            };
            if upd_src {
                file_result.insert_entry(code_tracking::remote_fetch_file(
                    ui.ctx(),
                    commit,
                    file_path,
                ));
            }
            resp
        } else {
            file_result.insert_entry(code_tracking::remote_fetch_file(
                ui.ctx(),
                commit,
                file_path,
            ));
            None
        }
    })
}

fn show_multi_repo(
    ui: &mut egui::Ui,
    selected: &mut types::SelectedConfig,
    multi: &mut types::ComputeConfigMulti,
) {
    let title = "Multi Repo";
    let wanted = types::SelectedConfig::Multi;
    let id = ui.make_persistent_id(title);
    let add_body = |ui: &mut egui::Ui| {
        ui.text_edit_singleline(&mut "github.com/INRIA/spoon");
    };

    radio_collapsing(ui, id, title, selected, wanted, add_body);
}

fn show_diff(
    ui: &mut egui::Ui,
    selected: &mut types::SelectedConfig,
    diff: &mut types::ComputeConfigDiff,
) {
    let title = "Semantic Diff";
    let wanted = types::SelectedConfig::Diff;
    let id = ui.make_persistent_id(title);
    let add_body = |ui: &mut egui::Ui| {
        ui.text_edit_singleline(&mut "github.com/INRIA/spoon");
    };

    radio_collapsing(ui, id, title, selected, wanted, add_body);
}

mod code_aspects;

pub(crate) fn show_repo(ui: &mut egui::Ui, repo: &mut Repo) -> bool {
    let mut changed = false;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        let user_id = ui.next_auto_id().with("user");
        let name_id = ui.next_auto_id().with("name");
        ui.push_id("user", |ui| {
            ui.label("github.com/"); // efwserfwefwe/fewefwse
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
