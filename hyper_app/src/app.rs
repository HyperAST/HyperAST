use self::{
    single_repo::ComputeConfigSingle,
    tree_view::FetchedHyperAST,
    types::{Commit, Languages, Repo},
};
use egui_addon::{
    code_editor::{self, generic_text_buffer::byte_index_from_char_index},
    egui_utils::{radio_collapsing, show_wip},
    interactive_split::interactive_splitter::InteractiveSplitter,
    syntax_highlighting::{self, syntax_highlighting_async},
    Lang,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{hash_map, HashMap, VecDeque},
    fmt::Debug,
    ops::Range,
    sync::Arc,
};

mod code_editor_automerge;
mod code_tracking;
mod commit;
pub(crate) mod crdt_over_ws;
mod long_tracking;
mod single_repo;
mod tree_view;
mod ts_highlight;
pub(crate) mod types;
mod utils;
mod code_editor_automerge;

// const API_URL: &str = "http://131.254.13.72:8080";
const API_URL: &str = "http://0.0.0.0:8080";

/// We derive Deserialize/Serialize so we can persist app state on shutdown.
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct HyperApp {
    // Example stuff:
    project_name: String,

    // code_editors: Arc<std::sync::Mutex<types::CodeEditors<code_editor_automerge::CodeEditor>>>,
    scripting_context: ScriptingContext<
        self::types::CodeEditors,
        types::CodeEditors<code_editor_automerge::CodeEditor>,
    >,

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
    #[serde(skip)]
    store: Arc<FetchedHyperAST>,

    long_tracking: long_tracking::LongTacking,
}

#[derive(Default, Serialize, Deserialize)]
struct ScriptingContext<L, S> {
    current: EditStatus<L, S>,
    local_scripts: HashMap<String, L>,
    // shared_script: Option<Arc<std::sync::Mutex<S>>>,
    // shared_script: Arc<std::sync::RwLock<Vec<Option<Arc<std::sync::Mutex<S>>>>>>,
    // shared_scripts: DashMap<String, Arc<std::sync::Mutex<S>>>,
}

#[derive(Serialize, Deserialize)]
enum EditStatus<L, S> {
    Sharing(Arc<std::sync::Mutex<S>>), //(Id)
    Shared(usize, Arc<std::sync::Mutex<S>>), //(Id)
    Local { name: String, content: L },
    Example { i: usize, content: L },
}
impl<L: Default, S> Default for EditStatus<L, S> {
    fn default() -> Self {
        Self::Example {
            i: 0,
            content: Default::default(),
        }
    }
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

#[derive(Serialize, Deserialize)]
pub struct MultiBuffered<T, U: std::marker::Send + 'static> {
    content: Option<T>,
    #[serde(skip)]
    waiting: VecDeque<poll_promise::Promise<U>>,
}
impl<T, U: std::marker::Send + 'static> Default for MultiBuffered<T, U> {
    fn default() -> Self {
        Self {
            content: Default::default(),
            waiting: Default::default(),
        }
    }
}
pub trait Accumulable<Rhs = Self> {
    fn acc(&mut self, rhs: Rhs) -> bool;
}
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AccumulableResult<T, E> {
    content: T,
    errors: E,
}
impl<T: Accumulable<U>, U, E: Accumulable<F>, F> Accumulable<Result<U, F>>
    for AccumulableResult<T, E>
{
    fn acc(&mut self, rhs: Result<U, F>) -> bool {
        match rhs {
            Ok(rhs) => self.content.acc(rhs),
            Err(err) => self.errors.acc(err),
        }
    }
}
impl Accumulable<String> for Vec<String> {
    fn acc(&mut self, rhs: String) -> bool {
        self.push(rhs);
        true
    }
}
impl<T: Default, U: std::marker::Send + 'static> MultiBuffered<T, U> {
    pub fn try_poll(&mut self) -> bool
    where
        T: Accumulable<U>,
    {
        if let Some(front) = self.waiting.pop_front() {
            match front.try_take() {
                Ok(content) => {
                    if self.content.is_none() {
                        self.content = Some(Default::default())
                    }
                    let Some(c) = &mut self.content  else {
                        unreachable!()
                    };
                    c.acc(content)
                }
                Err(front) => {
                    self.waiting.push_front(front);
                    false
                }
            }
        } else {
            false
        }
    }

    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.content.as_mut()
    }

    pub fn is_waiting(&self) -> bool {
        !self.waiting.is_empty()
    }

    pub fn buffer(&mut self, waiting: poll_promise::Promise<U>) {
        self.waiting.push_back(waiting)
    }

    pub fn take(&mut self) -> Option<T> {
        self.content.take()
    }
}

impl Default for HyperApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            project_name: "Simple Computation".to_owned(),
            // code_editors: Default::default(),
            scripting_context: Default::default(),
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
            store: Default::default(),
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
        // let mut arc = r.scripting_context.lock().unwrap();
        // arc.init.lang = languages.get("JavaScript").cloned();
        // arc.filter.lang = languages.get("JavaScript").cloned();
        // arc.accumulate.lang = languages.get("JavaScript").cloned();
        // dbg!(&r.code_editors.lang);
        // assert!(r.code_editors.lang.is_some());
        r.languages = languages.into();
        // drop(arc);
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
            // code_editors,
            scripting_context,
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
            store,
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
                    code_aspects::show_aspects_views_menu(
                        ui,
                        selected,
                        aspects,
                        store.clone(),
                        aspects_result,
                    );
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
                    single,
                    scripting_context,
                    &mut trigger_compute,
                    compute_single_result,
                );
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
            egui::CentralPanel::default()
                .frame(egui::Frame::central_panel(&ctx.style()).inner_margin(2.0))
                .show(ctx, |ui| {
                    long_tracking::show_results(
                        ui,
                        aspects,
                        store.clone(),
                        long_tracking,
                        fetched_files,
                    );
                });
        } else if *selected == types::SelectedConfig::Aspects {
            egui::CentralPanel::default().show(ctx, |ui| {
                if let Some(aspects_result) = aspects_result {
                    code_aspects::show(aspects_result, ui, aspects);
                } else {
                    // *aspects_result = Some(code_aspects::remote_fetch_tree(
                    //     ctx,
                    //     &aspects.commit,
                    //     &aspects.path,
                    // ));
                    *aspects_result = Some(code_aspects::remote_fetch_node(
                        ctx,
                        store.clone(),
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
                scripting_context,
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
            show_remote_code1(ui, commit, file_path, file_result, f32::INFINITY, false)
        })
        .inner
}

fn show_remote_code1(
    ui: &mut egui::Ui,
    commit: &mut types::Commit,
    mut file_path: &mut String,
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
        ui.id().with("file view"),
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
                            // show_code_scrolled(ui, language, wrap, code, desired_width)
                            show_code_scrolled(ui, language, wrap, code, desired_width)
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

fn show_code_scrolled(
    ui: &mut egui::Ui,
    language: &str,
    wrap: bool,
    code: &str,
    desired_width: f32,
) -> Option<egui::scroll_area::ScrollAreaOutput<egui::text_edit::TextEditOutput>> {
    // use egui_demo_lib::syntax_highlighting;
    use syntax_highlighting::syntax_highlighting_async as syntax_highlighter;
    let theme = syntax_highlighting::syntect::CodeTheme::from_memory(ui.ctx());

    let mut layouter = |ui: &egui::Ui, code: &str, wrap_width: f32| {
        let mut layout_job =
            // egui_demo_lib::syntax_highlighting::highlight(ui.ctx(), &theme, code, language);
            syntax_highlighter::highlight(ui.ctx(), &theme, code, language);
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
}

type SkipedBytes = usize;

fn show_remote_code2(
    ui: &mut egui::Ui,
    commit: &mut types::Commit,
    mut file_path: &mut String,
    file_result: hash_map::Entry<'_, types::FileIdentifier, code_tracking::RemoteFile>,
    desired_width: f32,
    wrap: bool,
) -> (
    egui::Response,
    egui::InnerResponse<()>,
    std::option::Option<
        egui::InnerResponse<
            Option<
                egui::scroll_area::ScrollAreaOutput<(SkipedBytes, egui::text_edit::TextEditOutput)>,
            >,
        >,
    >,
) {
    let mut upd_src = false;
    egui::collapsing_header::CollapsingState::load_with_default_open(
        ui.ctx(),
        ui.id().with("file view"),
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
                            // show_code_scrolled(ui, language, wrap, code, desired_width)
                            show_code_scrolled2(
                                ui,
                                language,
                                wrap,
                                code,
                                &text.line_breaks,
                                desired_width,
                            )
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

fn show_code_scrolled2(
    ui: &mut egui::Ui,
    language: &str,
    wrap: bool,
    code: &str,
    line_breaks: &[usize],
    desired_width: f32,
) -> Option<egui::scroll_area::ScrollAreaOutput<(SkipedBytes, egui::text_edit::TextEditOutput)>> {
    let theme = egui_demo_lib::syntax_highlighting::CodeTheme::from_memory(ui.ctx());
    let mut layouter = |ui: &egui::Ui, code: &str, wrap_width: f32| {
        let mut layout_job =
            egui_demo_lib::syntax_highlighting::highlight(ui.ctx(), &theme, code, language);
        if wrap {
            panic!();
            layout_job.wrap.max_width = wrap_width;
        }
        ui.fonts(|f| f.layout_job(layout_job))
    };
    let font_id = egui::FontId::new(10.0, egui::FontFamily::Monospace);

    let total_rows = line_breaks.len();
    Some(if false {
        // by rows highlight
        let row_height_sans_spacing = ui.fonts(|f| f.row_height(&font_id)) - 0.9; //text_style_height(&text_style);
        egui::ScrollArea::vertical().show_rows(
            ui,
            row_height_sans_spacing,
            total_rows,
            |ui, rows| {
                let start = if rows.start == 0 {
                    0
                } else {
                    line_breaks[rows.start - 1]
                };
                ui.painter()
                    .debug_rect(ui.max_rect(), egui::Color32::RED, "text");
                let desired_height_rows = ui.available_height() / row_height_sans_spacing
                    * (rows.end - rows.start) as f32;
                let mut te = egui::TextEdit::multiline(
                    &mut code[start..line_breaks[(rows.end).min(total_rows - 1)]].to_string(),
                )
                .font(font_id) // for cursor height
                .code_editor()
                // .desired_rows(desired_height_rows as usize)
                // .desired_width(100.0)
                .desired_width(desired_width)
                .clip_text(true)
                .lock_focus(true)
                .layouter(&mut layouter)
                .show(ui);
                (start, te)
            },
        )
    } else {
        egui::ScrollArea::vertical().show(ui, |ui| {
            let mut te = egui::TextEdit::multiline(&mut code.to_string())
                .font(font_id) // for cursor height
                .code_editor()
                // .desired_rows(desired_height_rows as usize)
                // .desired_width(100.0)
                .desired_width(desired_width)
                .clip_text(true)
                .lock_focus(true)
                .layouter(&mut layouter)
                .show(ui);
            (0, te)
        })
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

    radio_collapsing(ui, id, title, selected, &wanted, add_body);
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

    radio_collapsing(ui, id, title, selected, &wanted, add_body);
}

mod code_aspects;

use lazy_static::lazy_static;

lazy_static! {
    static ref COMMIT_STRS: Arc<std::sync::Mutex<BorrowFrameCache<String, ComputeCommitStr>>> = {
        // let mut map = HashMap::new();
        // map.insert("James", vec!["user", "admin"]);
        // map.insert("Jim", vec!["user"]);
        // map
        Default::default()
    };
}
pub struct BorrowFrameCache<Value, Computer> {
    generation: u32,
    computer: Computer,
    cache: nohash_hasher::IntMap<u64, (u32, Value)>,
}

impl<Value, Computer> Default for BorrowFrameCache<Value, Computer>
where
    Computer: Default,
{
    fn default() -> Self {
        Self::new(Computer::default())
    }
}

impl<Value, Computer> BorrowFrameCache<Value, Computer> {
    pub fn new(computer: Computer) -> Self {
        Self {
            generation: 0,
            computer,
            cache: Default::default(),
        }
    }

    /// Must be called once per frame to clear the cache.
    pub fn evice_cache(&mut self) {
        let current_generation = self.generation;
        self.cache.retain(|_key, cached| {
            cached.0 == current_generation // only keep those that were used this frame
        });
        self.generation = self.generation.wrapping_add(1);
    }
}

impl<Value: 'static + Send + Sync, Computer: 'static + Send + Sync> egui::util::cache::CacheTrait
    for BorrowFrameCache<Value, Computer>
{
    fn update(&mut self) {
        self.evice_cache();
    }

    fn len(&self) -> usize {
        self.cache.len()
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl<Value, Computer> BorrowFrameCache<Value, Computer> {
    /// Get from cache (if the same key was used last frame)
    /// or recompute and store in the cache.
    pub fn get<Key>(&mut self, key: Key) -> &Value
    where
        Key: Copy + std::hash::Hash,
        Computer: egui::util::cache::ComputerMut<Key, Value>,
    {
        let hash = egui::util::hash(key);

        match self.cache.entry(hash) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let cached = entry.into_mut();
                cached.0 = self.generation;
                &cached.1
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let value = self.computer.compute(key);
                &entry.insert((self.generation, value)).1
            }
        }
    }
    /// WARN panic if absent value
    pub fn access<Key>(&self, key: Key) -> &Value
    where
        Key: std::hash::Hash,
        Computer: egui::util::cache::ComputerMut<Key, Value>,
    {
        let hash = egui::util::hash(&key);
        &self.cache.get(&hash).unwrap().1
    }
}

#[derive(Default)]
struct ComputeCommitStr {
    // map:
}

impl egui::util::cache::ComputerMut<(&str, &Commit), String> for ComputeCommitStr {
    fn compute(&mut self, (forge, commit): (&str, &Commit)) -> String {
        format!(
            "{}/{}/{}/{}",
            forge, commit.repo.user, commit.repo.name, commit.id
        )
    }
}

struct CommitTextBuffer<'a, 'b, 'c> {
    commit: &'a mut Commit,
    forge: String,
    str: &'b mut std::sync::MutexGuard<'c, BorrowFrameCache<String, ComputeCommitStr>>,
}

impl<'a, 'b, 'c> CommitTextBuffer<'a, 'b, 'c> {
    fn new(
        commit: &'a mut Commit,
        forge: String,
        str: &'b mut std::sync::MutexGuard<'c, BorrowFrameCache<String, ComputeCommitStr>>,
    ) -> Self {
        str.get((&forge, commit));
        Self { commit, forge, str }
    }
}

impl<'a, 'b, 'c> code_editor::generic_text_buffer::TextBuffer for CommitTextBuffer<'a, 'b, 'c> {
    type Ref = String;
    fn is_mutable(&self) -> bool {
        true
    }
    fn as_reference(&self) -> &Self::Ref {
        self.str.access((&self.forge, &self.commit))
    }

    fn insert_text(&mut self, text: &str, char_index: usize) -> usize {
        // Get the byte index from the character index
        let byte_idx = self.byte_index_from_char_index(char_index);
        if text.starts_with("https://") {
            let text = &text["https://".len()..];
            let split: Vec<_> = text.split("/").collect();
            if split[0] != "github.com" {
                // TODO launch an alert
                // wasm_rs_dbg::dbg!("only github.com is allowed");
                return 0;
            }
            if split.len() == 5 {
                self.commit.repo.user = split[1].to_string();
                self.commit.repo.name = split[2].to_string();
                assert_eq!("commit", split[3].to_string());
                self.commit.id = split[4].to_string();
            }
            // wasm_rs_dbg::dbg!(&self.commit);
            self.str.get((&self.forge, &self.commit));
            return text.chars().count();
        }

        let mut t = self.str.get((&self.forge, &self.commit)).to_string();

        t.insert_str(byte_idx, text);
        let split: Vec<_> = t.split("/").collect();
        if split[0] != "github.com" {
            // TODO launch an alert
            // wasm_rs_dbg::dbg!("only github.com is allowed");
            return 0;
        }
        self.commit.repo.user = split[1].to_string();
        self.commit.repo.name = split[2].to_string();
        self.commit.id = split[3].to_string();

        self.str.get((&self.forge, &self.commit));

        text.chars().count()
    }

    fn delete_char_range(&mut self, char_range: Range<usize>) {
        // assert!(char_range.start <= char_range.end);

        // // Get both byte indices
        // let byte_start = self.byte_index_from_char_index(char_range.start);
        // let byte_end = self.byte_index_from_char_index(char_range.end);

        // // Then drain all characters within this range
        // self.drain(byte_start..byte_end);
        // todo!()
        // WARN could produce unexpected functional results for the user
    }

    fn replace_range(&mut self, text: &str, char_range: Range<usize>) -> usize {
        // Get the byte index from the character index
        let byte_idx = self.byte_index_from_char_index(char_range.start);
        if text.starts_with("https://") {
            let text = &text["https://".len()..];
            let split: Vec<_> = text.split("/").collect();
            if split[0] != "github.com" {
                // TODO launch an alert
                // wasm_rs_dbg::dbg!(&split[0]);
                // wasm_rs_dbg::dbg!("only github.com is allowed");
                return 0;
            }
            if split.len() == 5 {
                self.commit.repo.user = split[1].to_string();
                self.commit.repo.name = split[2].to_string();
                assert_eq!("commit", split[3].to_string());
                self.commit.id = split[4].to_string();
            }
            // wasm_rs_dbg::dbg!(&split, &self.commit);
            self.str.get((&self.forge, &self.commit));
            return text.chars().count();
        }

        let mut t = self.str.get((&self.forge, &self.commit)).to_string();
        {
            let byte_start = byte_index_from_char_index(&t, char_range.start);
            let byte_end = byte_index_from_char_index(&t, char_range.end);
            t.drain(byte_start..byte_end);
        }
        t.insert_str(byte_idx, text);
        let split: Vec<_> = text.split("/").collect();
        if split[0] != "github.com" {
            // TODO launch an alert
            // wasm_rs_dbg::dbg!("only github.com is allowed");
            return 0;
        }
        self.commit.repo.user = split[1].to_string();
        self.commit.repo.name = split[2].to_string();
        self.commit.id = split[3].to_string();

        self.str.get((&self.forge, &self.commit));

        text.chars().count()
    }

    fn clear(&mut self) {
        // self.clear()
    }

    fn replace(&mut self, text: &str) {
        // *self = text.to_owned();
    }

    fn take(&mut self) -> String {
        self.str.get((&self.forge, &self.commit)).to_string()
    }
}

pub(crate) fn show_commit_menu(ui: &mut egui::Ui, commit: &mut Commit) -> bool {
    let mut mutex_guard = COMMIT_STRS.lock().unwrap();
    let mut c = CommitTextBuffer::new(commit, "github.com".to_string(), &mut mutex_guard);
    let ml = code_editor::generic_text_edit::TextEdit::multiline(&mut c)
        // .margin(egui::Vec2::new(0.0, 0.0))
        // .desired_width(40.0)
        .id(ui.id().with("commit entry"))
        .show(ui);

    ml.response.changed()
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
