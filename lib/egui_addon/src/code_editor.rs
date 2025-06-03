use crate::Languages;

use self::generic_text_buffer::TextBuffer;
use super::Lang;
pub use editor_content::EditAwareString;
use egui::{Response, WidgetText};
use egui_demo_lib::easy_mark::easy_mark;
use generic_text_edit::TextEdit;
use serde::Deserialize;
use std::fmt::Debug;

#[cfg(feature = "ts_highlight")]
const TREE_SITTER: bool = false;
#[cfg(not(feature = "ts_highlight"))]
const TREE_SITTER: bool = false;

pub(crate) mod editor_content;

mod generic_state;
pub mod generic_text_buffer;
pub mod generic_text_edit;

pub trait CodeHolder {
    fn set_lang(&mut self, lang: &str);
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct CodeEditor<L, C = EditAwareString> {
    #[serde(default = "default_info")]
    pub info: EditorInfo<String>,
    pub lang_name: String,
    // code: String,
    pub code: C,
    #[cfg(feature = "ts_highlight")]
    #[serde(skip)]
    #[serde(default = "default_parser")]
    pub parser: tree_sitter::Parser,
    #[serde(skip)]
    pub languages: L,
    #[serde(skip)]
    pub lang: Option<Lang>,
}

impl<L: Languages, C> CodeHolder for CodeEditor<L, C> {
    fn set_lang(&mut self, lang: &str) {
        self.lang = self.languages.get(lang);
        self.lang_name = lang.into();
    }
}

impl<L, C: Debug> Debug for CodeEditor<L, C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodeEditor")
            .field("code", &self.code)
            .finish()
    }
}

impl<C: Clone> Clone for CodeEditor<C> {
    fn clone(&self) -> Self {
        Self {
            info: self.info.clone(),
            lang_name: self.lang_name.clone(),
            code: self.code.clone(),
            lang: self.lang.clone(),
            #[cfg(feature = "ts_highlight")]
            parser: default_parser(),
            languages: self.languages.clone(),
        }
    }
}

impl<L: Default + Languages, C: From<String>> From<(EditorInfo<String>, String)>
    for CodeEditor<L, C>
{
    fn from((info, code): (EditorInfo<String>, String)) -> Self {
        let code = code.into();
        Self {
            info,
            code,
            ..Default::default()
        }
    }
}

impl<L: Default + Languages, C: From<String>> CodeEditor<L, C> {
    pub fn new(info: EditorInfo<String>, code: String) -> Self {
        let code = code.into();
        Self {
            info,
            code,
            ..Default::default()
        }
    }
}

#[derive(Deserialize, serde::Serialize, Clone)]
pub struct EditorInfo<T> {
    pub title: T,
    pub short: T,
    pub long: T,
}

impl Default for EditorInfo<&'static str> {
    fn default() -> Self {
        Self {
            title: "Editor",
            short: "a code editor",
            long: "this is a code editor, you should probably make a custom description on its purpose",
        }
    }
}
impl EditorInfo<&'static str> {
    pub fn copied(&self) -> EditorInfo<String> {
        EditorInfo {
            title: self.title.to_string(),
            short: self.short.to_string(),
            long: self.long.to_string(),
        }
    }
}
pub fn default_info() -> EditorInfo<String> {
    EditorInfo::default().copied()
}

#[cfg(feature = "ts_highlight")]
#[cfg(not(target_arch = "wasm32"))]
pub fn default_parser() -> tree_sitter::Parser {
    tree_sitter::Parser::new()
}

#[cfg(feature = "ts_highlight")]
#[cfg(target_arch = "wasm32")]
pub fn default_parser() -> tree_sitter::Parser {
    tree_sitter::Parser::new().unwrap()
}

impl<L: Default + Languages, C: From<String>> Default for CodeEditor<L, C> {
    fn default() -> Self {
        let languages = L::default();
        let lang = languages.get("JavaScript");
        Self {
            lang_name: "JavaScript".into(),
            code: r#"function  f() { return 0; }
function f() { return 1; }

function f() { return 2; }
// class Test {
//     int double(int x) {
//         return x * 2;
//     }
// }
            "#
            .to_string()
            .into(),
            #[cfg(feature = "ts_highlight")]
            parser: default_parser(),
            languages: Default::default(),
            lang,
            info: EditorInfo::default().copied(),
        }
    }
}

impl<L: Default + Languages> From<&str> for CodeEditor<L> {
    fn from(value: &str) -> Self {
        Self {
            code: value.to_string().into(),
            ..Default::default()
        }
    }
}

impl<L: Default + Languages> AsRef<str> for CodeEditor<L> {
    fn as_ref(&self) -> &str {
        self.code()
    }
}

impl<L: Default> CodeEditor<L> {
    pub fn code(&self) -> &str {
        self.code.as_str()
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) {
        let Self {
            code, lang, info, ..
        } = self;

        let theme = crate::syntax_highlighting::simple::CodeTheme::from_memory(ui.ctx());

        let id = ui.make_persistent_id(&info.title);
        let mut col =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);

        let title = egui::RichText::new(&info.title).heading();
        let header_res = ui.horizontal(|ui| {
            col.show_toggle_button(ui, checkbox_heading(title));
            ui.add_space(100.);
            ui.label(egui::RichText::new(&info.short).italics())
                .on_hover_ui(|ui| {
                    easy_mark(ui, &info.long);
                });
        });
        col.show_body_indented(&header_res.response, ui, |ui| {
            egui::ScrollArea::vertical().show(ui, |ui| {
                if TREE_SITTER {
                    #[cfg(feature = "ts_highlight")]
                    let _layouter = |ui: &egui::Ui, code: &EditAwareString, wrap_width: f32| {
                        dbg!(&lang);
                        let mut layout_job =
                            crate::syntax_highlighting::syntax_highlighting_ts::highlight(
                                ui.ctx(),
                                &theme,
                                code,
                                &lang.as_ref().unwrap(),
                            );
                        layout_job.wrap.max_width = wrap_width;
                        ui.fonts(|f| f.layout_job(layout_job))
                    };
                } else {
                    show_edit_syntect(ui, code);
                }
            });
        });
    }
}

pub fn show_edit_syntect(ui: &mut egui::Ui, code: &mut EditAwareString) {
    let language = "rs";
    let theme = egui_extras::syntax_highlighting::CodeTheme::from_memory(ui.ctx(), ui.style());

    let mut layouter = |ui: &egui::Ui, string: &EditAwareString, _wrap_width: f32| {
        let layout_job = egui_extras::syntax_highlighting::highlight(
            ui.ctx(),
            ui.style(),
            &theme,
            string.as_str(),
            language,
        );
        ui.fonts(|f| f.layout_job(layout_job))
    };

    ui.add(
        TextEdit::multiline(code)
            .font(egui::TextStyle::Monospace) // for cursor height
            .code_editor()
            .desired_rows(1)
            .lock_focus(true)
            .layouter(&mut layouter),
    );
}

fn checkbox_heading(
    text: impl Into<WidgetText> + 'static,
) -> impl FnOnce(&mut egui::Ui, f32, &Response) + 'static {
    |ui: &mut egui::Ui, openness: f32, response: &Response| {
        // let Checkbox { checked, text } = self;
        use egui::NumExt;

        let spacing = &ui.spacing();
        let icon_width = spacing.icon_width;
        let icon_spacing = spacing.icon_spacing;
        let text: WidgetText = text.into();

        let (text, mut desired_size) = if text.is_empty() {
            (None, epaint::vec2(icon_width, 0.0))
        } else {
            let total_extra = epaint::vec2(icon_width + icon_spacing, 0.0);

            let wrap_width = ui.available_width() - total_extra.x;
            let text = text.into_galley(ui, None, wrap_width, egui::TextStyle::Button);

            let mut desired_size = total_extra + text.size();
            desired_size = desired_size.at_least(spacing.interact_size);

            (Some(text), desired_size)
        };

        desired_size = desired_size.at_least(epaint::Vec2::splat(spacing.interact_size.y));
        desired_size.y = desired_size.y.max(icon_width);
        let rect = response.rect;

        let checked = openness > 0.8;
        let checked = &checked;

        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::Checkbox,
                true,
                *checked,
                text.as_ref().map_or("", |x| x.text()),
            )
        });

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().interact(&response);
            let (small_icon_rect, big_icon_rect) = ui.spacing().icon_rectangles(rect);
            ui.painter().add(epaint::RectShape::new(
                big_icon_rect.expand(visuals.expansion),
                visuals.corner_radius,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            ));

            if *checked {
                ui.painter().add(egui::Shape::line(
                    vec![
                        epaint::pos2(small_icon_rect.left(), small_icon_rect.center().y),
                        epaint::pos2(small_icon_rect.center().x, small_icon_rect.bottom()),
                        epaint::pos2(small_icon_rect.right(), small_icon_rect.top()),
                    ],
                    visuals.fg_stroke,
                ));
            }
            if let Some(text) = text {
                let text_pos = epaint::pos2(
                    rect.min.x + icon_width + icon_spacing,
                    rect.center().y - 0.5 * text.size().y,
                );
                ui.painter().galley(text_pos, text, visuals.text_color());
                // text.paint_with_visuals(ui.painter(), text_pos, visuals);
            }
        }
    }
}
