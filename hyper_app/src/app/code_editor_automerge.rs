//! NOTE Pretty adoc impl, would benefit from being merged with impl in egui_addon

use super::Lang;
use eframe::epaint::ahash::HashMap;
use egui::{Response, WidgetText};
use egui_addon::code_editor::EditorInfo;
use egui_demo_lib::easy_mark::easy_mark;
use std::sync::Arc;

const TREE_SITTER: bool = false;

use crate::app::crdt_over_ws::Quote;

#[derive(serde::Deserialize, serde::Serialize)]
pub(crate) struct CodeEditor<C = Quote> {
    #[serde(default = "default_info")]
    pub info: EditorInfo<String>,
    pub language: String,
    // code: String,
    pub code: C,
    #[serde(skip)]
    #[serde(default = "default_parser")]
    pub parser: tree_sitter::Parser,
    #[serde(skip)]
    pub languages: Arc<HashMap<String, Lang>>,
    #[serde(skip)]
    pub lang: Option<Lang>,
}
impl<C> egui_addon::code_editor::CodeHolder for CodeEditor<C> {
    fn set_lang(&mut self, lang: String) {
        self.language = lang;
    }
}
impl From<egui_addon::code_editor::CodeEditor> for CodeEditor {
    fn from(value: egui_addon::code_editor::CodeEditor) -> Self {
        let code = value.code;
        let code = code.into();
        Self {
            info: value.info,
            language: value.language,
            code,
            parser: value.parser,
            languages: value.languages,
            lang: value.lang,
        }
    }
}

impl<C: From<String>> From<(EditorInfo<String>, String)> for CodeEditor<C> {
    fn from((info, code): (EditorInfo<String>, String)) -> Self {
        let code = code.into();
        Self {
            info,
            code,
            ..Default::default()
        }
    }
}

impl autosurgeon::Reconcile for CodeEditor {
    type Key<'a> = std::borrow::Cow<'a, String>;

    fn reconcile<R: autosurgeon::Reconciler>(&self, mut reconciler: R) -> Result<(), R::Error> {
        use autosurgeon::reconcile::MapReconciler;
        let mut m = reconciler.map()?;
        // m.put("id", &self.language)?;
        m.put("info.title", &self.info.title)?;
        m.put("info.short", &self.info.short)?;
        m.put("info.long", &self.info.long)?;
        m.put("code", &self.code)?;
        Ok(())
    }

    // fn hydrate_key<'a, D: autosurgeon::ReadDoc>(
    //     doc: &D,
    //     obj: &automerge::ObjId,
    //     prop: autosurgeon::Prop<'_>,
    // ) -> Result<autosurgeon::reconcile::LoadKey<Self::Key<'a>>, autosurgeon::ReconcileError> {
    //     autosurgeon::hydrate_key(doc, obj, prop, "id".into())
    // }

    // fn key(&self) -> autosurgeon::reconcile::LoadKey<std::borrow::Cow<'_, String>> {
    //     autosurgeon::reconcile::LoadKey::Found(std::borrow::Cow::Borrowed(&self.language))
    // }
}

impl autosurgeon::Hydrate for CodeEditor {
    fn hydrate_map<D: autosurgeon::ReadDoc>(
        doc: &D,
        obj: &automerge::ObjId,
    ) -> Result<Self, autosurgeon::HydrateError> {
        use automerge::ObjType;
        use autosurgeon::HydrateError;
        let Some(obj_type) = doc.object_type(obj) else {
            return Err(HydrateError::unexpected("a map", "a scalar value".to_string()))
        };
        match obj_type {
            ObjType::Map | ObjType::Table => {
                Ok(Self {
                    // language: String::hydrate(doc, obj, "id".into())?,
                    info: EditorInfo {
                        title: String::hydrate(doc, obj, "info.title".into())?,
                        short: String::hydrate(doc, obj, "info.short".into())?,
                        long: String::hydrate(doc, obj, "info.long".into())?,
                    },
                    code: crate::app::crdt_over_ws::Quote::hydrate(doc, obj, "code".into())?,
                    ..Default::default()
                })
            }
            ObjType::Text => Err(HydrateError::unexpected(
                "a map",
                "a text object".to_string(),
            )),
            ObjType::List => Err(HydrateError::unexpected(
                "a map",
                "a list object".to_string(),
            )),
        }
    }
}

pub(crate) fn default_info() -> EditorInfo<String> {
    EditorInfo::default().copied()
}

pub(crate) fn default_parser() -> tree_sitter::Parser {
    tree_sitter::Parser::new().unwrap()
}

impl<C: From<String>> Default for CodeEditor<C> {
    fn default() -> Self {
        let code = &r#"function  f() { return 0; }
function f() { return 1; }

function f() { return 2; }
// class Test {
//     int double(int x) {
//         return x * 2;
//     }
// }
//             "#;
        Self {
            language: "JavaScript".into(),
            code: code.to_string().into(),
            parser: default_parser(),
            languages: Default::default(),
            lang: Default::default(),
            info: EditorInfo::default().copied(),
        }
    }
}

impl From<&str> for CodeEditor {
    fn from(value: &str) -> Self {
        Self {
            code: value.to_string().into(),
            ..Default::default()
        }
    }
}

// impl super::Demo for CodeEditor {
//     fn name(&self) -> &'static str {
//         "🖮 Code Editor"
//     }

//     fn show(&mut self, ctx: &egui::Context, open: &mut bool) {
//         use super::View as _;
//         egui::Window::new(self.name())
//             .open(open)
//             .default_height(500.0)
//             .show(ctx, |ui| self.ui(ui));
//     }
// }

impl CodeEditor {
    pub(crate) fn title(&mut self, _title: &str) -> &mut Self {
        // self.
        self
    }
    // pub(crate) fn set_info(&mut self, info: EditorInfo<String>) -> &mut Self {
    //     self.info = info;
    //     self
    // }
    pub fn code(&self) -> &str {
        use egui_addon::code_editor::generic_text_buffer::TextBuffer;
        self.code.as_str()
    }
    pub fn ui(&mut self, ui: &mut egui::Ui) -> Option<Response> {
        let Self {
            code, lang, info, ..
        } = self;

        // ui.horizontal(|ui| {
        //     ui.set_height(0.0);
        //     ui.label("init:");
        // });

        // if cfg!(feature = "syntect") {
        //     ui.horizontal(|ui| {
        //         ui.label("Language:");
        //         if ui.text_edit_singleline(language).changed() {
        //             todo!()
        //         }
        //     });
        //     ui.horizontal_wrapped(|ui| {
        //         ui.spacing_mut().item_spacing.x = 0.0;
        //         ui.label("Syntax highlighting powered by ");
        //         ui.hyperlink_to("syntect", "https://github.com/trishume/syntect");
        //         ui.label(".");
        //     });
        // } else {
        //     ui.horizontal_wrapped(|ui| {
        //         ui.spacing_mut().item_spacing.x = 0.0;
        //         ui.label("Compile the demo with the ");
        //         ui.code("syntax_highlighting");
        //         ui.label(" feature to enable more accurate syntax highlighting using ");
        //         ui.hyperlink_to("syntect", "https://github.com/trishume/syntect");
        //         ui.label(".");
        //     });
        // }

        let theme = egui_addon::syntax_highlighting::simple::CodeTheme::from_memory(ui.ctx());
        // ui.collapsing("Theme", |ui| {
        //     ui.group(|ui| {
        //         theme.ui(ui);
        //         theme.clone().store_in_memory(ui.ctx());
        //     });
        // });

        let id = ui.make_persistent_id(&info.title);
        let mut col =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);
        // col.show_header(ui, |ui| {
        //     ui.label(egui::RichText::new(&info.title).heading());
        //     ui.label(egui::RichText::new(&info.short).italics())
        //         .on_hover_ui(|ui| {
        //             easy_mark(ui, &info.long);
        //         });
        //     // ui.toggle_value(&mut self.selected, "Filter");
        //     // ui.radio_value(&mut self.radio_value, false, "");
        //     // ui.radio_value(&mut self.radio_value, true, "");
        // })

        let title = egui::RichText::new(&info.title).heading();
        let header_res = ui.horizontal(|ui| {
            col.show_toggle_button(ui, checkbox_heading(title));
            // ui.label("Header");
            ui.add_space(100.);
            ui.label(egui::RichText::new(&info.short).italics())
                .on_hover_ui(|ui| {
                    easy_mark(ui, &info.long);
                });
        });
        col.show_body_indented(&header_res.response, ui, |ui| {
            // .body(|ui| {
            egui::ScrollArea::vertical()
                .show(ui, |ui| {
                    if TREE_SITTER {
                        // let layouter = |ui: &egui::Ui, code: &EditAwareString, wrap_width: f32| {
                        //     dbg!(&lang);
                        //     let mut layout_job =
                        //         egui_addon::syntax_highlighting::syntax_highlighting_ts::highlight(
                        //             ui.ctx(),
                        //             &theme,
                        //             code,
                        //             &lang.as_ref().unwrap(),
                        //         );
                        //     layout_job.wrap.max_width = wrap_width;
                        //     ui.fonts(|f| f.layout_job(layout_job))
                        // };

                        // let out = generic_text_edit::TextEdit::multiline(code)
                        //     .font(egui::TextStyle::Monospace) // for cursor height
                        //     .code_editor()
                        //     .desired_rows(5)
                        //     .lock_focus(true)
                        //     .desired_width(f32::INFINITY)
                        //     .layouter(&mut layouter)
                        //     .show(ui);
                        panic!()
                    } else {
                        let language = "rs";
                        let theme =
                            egui_demo_lib::syntax_highlighting::CodeTheme::from_memory(ui.ctx());

                        let mut layouter =
                            |ui: &egui::Ui,
                             string: &super::crdt_over_ws::Quote,
                             _wrap_width: f32| {
                                let layout_job = egui_demo_lib::syntax_highlighting::highlight(
                                    ui.ctx(),
                                    &theme,
                                    string.text.as_str(),
                                    language,
                                );
                                // layout_job.wrap.max_width = wrap_width; // no wrapping
                                ui.fonts(|f| f.layout_job(layout_job))
                            };
                        ui.add(
                            egui_addon::code_editor::generic_text_edit::TextEdit::multiline(code)
                                .font(egui::TextStyle::Monospace) // for cursor height
                                .code_editor()
                                .desired_rows(1)
                                .lock_focus(true)
                                .layouter(&mut layouter),
                        )
                    }
                })
                .inner
        })
        .map(|x| x.inner)
    }
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
        let text = text.into();

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
        // let (rect, mut response) = ui.allocate_exact_size(desired_size, Sense::click());

        if response.clicked() {
            // *checked = !*checked;
            // response.mark_changed();
        }

        let checked = openness > 0.8;
        let checked = &checked;

        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::WidgetType::Checkbox,
                *checked,
                text.as_ref().map_or("", |x| x.text()),
            )
        });

        if ui.is_rect_visible(rect) {
            // let visuals = ui.style().interact_selectable(&response, *checked); // too colorful
            let visuals = ui.style().interact(&response);
            let (small_icon_rect, big_icon_rect) = ui.spacing().icon_rectangles(rect);
            ui.painter().add(epaint::RectShape {
                rect: big_icon_rect.expand(visuals.expansion),
                rounding: visuals.rounding,
                fill: visuals.bg_fill,
                stroke: visuals.bg_stroke,
            });

            if *checked {
                // Check mark:
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
                text.paint_with_visuals(ui.painter(), text_pos, visuals);
            }
        }

        // ui.checkbox(&mut false, title);
        // let stroke = ui.style().interact(&response).fg_stroke;
        // let radius = egui::lerp(2.0..=3.0, openness);
        // ui.painter()
        //     .circle_filled(response.rect.center(), radius, stroke.color)
    }
}
