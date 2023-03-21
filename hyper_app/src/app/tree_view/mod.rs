use egui::TextFormat;
use epaint::text::LayoutSection;
pub use hyper_ast::{types::Type, store::nodes::fetched::{NodeStore, FetchedLabels, NodeIdentifier}};
use lazy_static::__Deref;
use std::{
    fmt::Debug,
    ops::Mul,
    sync::{
        atomic::{AtomicBool, AtomicUsize},
        Arc,
    },
    time::Duration,
};

use crate::app::{
    syntax_highlighting_async::{self, async_exec},
    syntax_highlighting_ts::{self, CodeTheme},
};

// mod store;
// pub use self::store::{FetchedHyperAST, NodeId};
mod cache;

#[derive(Debug)]
pub struct PrefillCache {
    head: f32,
    children: Vec<f32>,
    next: Option<Box<PrefillCache>>,
}

impl PrefillCache {
    fn height(&self) -> f32 {
        self.head
            + self.children.iter().sum::<f32>()
            + self.next.as_ref().map_or(0.0, |x| x.height())
    }
}
#[derive(serde::Deserialize, serde::Serialize, Clone, Debug)]
pub(crate) enum Action {
    Keep,
    Focused(f32),
    Clicked(Vec<usize>),
    Delete,
}
pub(crate) struct FetchedViewImpl<'a> {
    store: Arc<NodeStore>,
    aspects: &'a super::types::ComputeConfigAspectViews,
    pub(super) prefill_cache: Option<PrefillCache>,
    min_before_count: usize,
    draw_count: usize,
    hightlights: Vec<(&'a [usize], &'a egui::Color32, &'a mut Option<egui::Pos2>)>,
    focus: Option<&'a [usize]>,
    path: Vec<usize>,
    root_ui_id: egui::Id,
}

impl<'a> Debug for FetchedViewImpl<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FetchedViewImpl")
            .field("prefill_cache", &self.prefill_cache)
            .field("min_before_count", &self.min_before_count)
            .field("draw_count", &self.draw_count)
            .finish()
    }
}
struct FoldRet<U, V> {
    toggle_response: egui::Response,
    header_response: egui::Response,
    header_returned: U,
    body_response: Option<egui::Response>,
    body_returned: Option<V>,
}

impl<U, V>
    From<(
        egui::Response,
        egui::InnerResponse<U>,
        Option<egui::InnerResponse<V>>,
    )> for FoldRet<U, V>
{
    fn from(
        value: (
            egui::Response,
            egui::InnerResponse<U>,
            Option<egui::InnerResponse<V>>,
        ),
    ) -> Self {
        let (resp, ret) = value
            .2
            .map_or((None, None), |x| (Some(x.response), Some(x.inner)));
        Self {
            toggle_response: value.0,
            header_response: value.1.response,
            header_returned: value.1.inner,
            body_response: resp,
            body_returned: ret,
        }
    }
}

impl<'a> FetchedViewImpl<'a> {
    pub(crate) fn new(
        store: Arc<FetchedHyperAST>,
        aspects: &'a super::types::ComputeConfigAspectViews,
        take: Option<PrefillCache>,
        hightlights: Vec<(&'a [usize], &'a epaint::Color32, &'a mut Option<egui::Pos2>)>,
        focus: Option<&'a [usize]>,
        path: Vec<usize>,
        root_ui_id: egui::Id,
    ) -> Self {
        Self {
            store,
            aspects,
            prefill_cache: take,
            draw_count: 0,
            min_before_count: 0,
            hightlights,
            focus,
            path,
            root_ui_id,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, root: &NodeIdentifier) -> Action {
        ui.style_mut().spacing.button_padding.y = 0.0;
        ui.style_mut().spacing.item_spacing.y = 0.0;

        todo!()
        // let r = if let Some(c) = self.store.both.ids.iter().position(|x| x == root) {
        //     self.ui_both_impl(ui, 0, c)
        // } else if let Some(c) = self.store.labeled.ids.iter().position(|x| x == root) {
        //     self.ui_labeled_impl(ui, 0, c)
        // } else if let Some(c) = self.store.children.ids.iter().position(|x| x == root) {
        //     self.ui_children_impl(ui, 0, c)
        // } else if let Some(c) = self.store.typed.ids.iter().position(|x| x == root) {
        //     self.ui_typed_impl(ui, 0, c)
        // } else {
        //     panic!();
        // };
        // r
    }

    pub(crate) fn ui_both_impl(&mut self, ui: &mut egui::Ui, depth: usize, nid: usize) -> Action {
        let kind = &self.store.type_sys.0[self.store.both.kinds[nid] as usize];
        let label = self.store.both.labels[nid];
        let label = &self.store.label_list[label as usize];
        let min = ui.available_rect_before_wrap().min;
        if min.y < 0.0 {
            self.min_before_count += 1;
            // wasm_rs_dbg::dbg!(min.y);
        }
        // ui.painter().debug_rect(
        //     ui.available_rect_before_wrap(),
        //     egui::Color32::GREEN,
        //     format!("{:?}", ""),
        // );
        // egui::CollapsingHeader::new(format!("{}: {}\t{}", kind, label, nid))

        self.draw_count += 1;
        let id = ui.id().with(&self.path);
        // let id = ui.make_persistent_id("my_collapsing_header");
        let mut load_with_default_open =
            egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                id,
                depth < 1,
            );
        if self.focus.is_some() {
            wasm_rs_dbg::dbg!(self.focus);
            load_with_default_open.set_open(true)
        }
        let show: FoldRet<_, _> = load_with_default_open
            .show_header(ui, |ui| {
                // ui.label(format!("{}: {}", kind, label));
                let ret = ui.monospace(format!("{}: ", kind)).context_menu(|ui| {
                    ui.label(format!("{:?}", self.path));
                });
                ui.label(format!("{}", label));
                ret
            })
            .body(|ui| {
                let o = self.store.both.cs_ofs[nid] as usize;
                let cs = &self.store.both.children[o..o + self.store.both.cs_lens[nid] as usize]
                    .to_vec();
                self.children_ui(ui, depth, cs)
            })
            .into();
        // let show = egui::CollapsingHeader::new(format!("{}: {}", kind, label))
        //     .id_source(id)
        //     .default_open(depth < 1)
        //     .show(ui, |ui| {
        //         if egui::collapsing_header::CollapsingState::load(ui.ctx(), id).map_or(false, |x|x.is_open()) {
        //             let o = self.store.both.cs_ofs[nid] as usize;
        //             let cs = &self.store.both.children[o..o + self.store.both.cs_lens[nid] as usize]
        //                 .to_vec();
        //             self.children_ui(ui, depth, cs)
        //         } else {
        //             Action::Keep
        //         }
        //     });

        let mut prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
            prefill_cache
        } else {
            PrefillCache {
                head: 0.0,
                children: vec![],
                next: None,
            }
        };
        prefill.head = show.header_response.rect.height();
        if DEBUG_LAYOUT {
            ui.painter().debug_rect(
                show.header_response.rect.union(
                    show.body_response
                        .as_ref()
                        .map(|x| x.rect)
                        .unwrap_or(egui::Rect::NOTHING),
                ),
                egui::Color32::BLUE,
                format!("\t\t\t\t\t\t\t\t{:?}", show.header_response.rect),
            );
        }
        let mut rect = show.header_response.rect.union(
            show.body_response
                .as_ref()
                .map(|x| x.rect)
                .unwrap_or(egui::Rect::NOTHING),
        );
        rect.max.x += 10.0;

        for (p, c, ret_pos) in &mut self.hightlights {
            selection_highlight(ui, *p, *c, min, rect, self.root_ui_id, *ret_pos);
        }
        // ui.label(format!("{:?}", show.body_response.map(|x| x.rect)));
        self.prefill_cache = Some(prefill);

        if show
            .header_returned
            .interact(egui::Sense::click())
            .clicked()
        {
            Action::Clicked(self.path.to_vec())
        } else if let Some(&[]) = self.focus {
            Action::Focused(min.y)
        } else {
            show.body_returned.unwrap_or(Action::Keep)
        }
    }
    pub(crate) fn ui_children_impl(
        &mut self,
        ui: &mut egui::Ui,
        depth: usize,
        nid: usize,
    ) -> Action {
        // egui::CollapsingHeader::new(format!("{}\t{}", kind, nid))
        let min = ui.available_rect_before_wrap().min;
        if min.y < 0.0 {
            self.min_before_count += 1;
        }
        // ui.painter().debug_rect(
        //     ui.available_rect_before_wrap(),
        //     egui::Color32::GREEN,
        //     format!("{:?}", ""),
        // );
        let kind = &self.store.type_sys.0[self.store.children.kinds[nid] as usize];
        self.draw_count += 1;
        let id = ui.id().with(&self.path);
        // let id = ui.make_persistent_id("my_collapsing_header");

        if kind == "import_declaration"
            || kind == "expression"
            || kind == "formal_parameters"
            || kind == "expression_statement"
        {
            let mut prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
                prefill_cache
            } else {
                PrefillCache {
                    head: 0.0,
                    children: vec![],
                    next: None,
                }
            };
            let min = ui.available_rect_before_wrap().min;
            let theme = syntax_highlighting_ts::CodeTheme::from_memory(ui.ctx());
            let layout_job = make_pp_code(self.store.clone(), ui.ctx(), nid, theme);
            let galley = ui.fonts(|f| f.layout_job(layout_job));

            let size = galley.size();
            let resp = ui.allocate_exact_size(size, egui::Sense::click());

            let rect = egui::Rect::from_min_size(min, size);
            let rect = rect;//.expand(3.0);
            ui.painter_at(rect.expand(1.0)).galley(min, galley);
            // rect.max.x += 10.0;

            prefill.head = ui.available_rect_before_wrap().min.y - min.y;

            for (p, c, ret_pos) in &mut self.hightlights {
                // egui::Rect::from_min_size(min, (ui.available_width(), height).into()),
                selection_highlight(ui, *p, *c, min, rect, self.root_ui_id, *ret_pos);
            }
            self.prefill_cache = Some(prefill);

            return if resp.1.clicked() {
                Action::Clicked(self.path.to_vec())
            } else if let Some(&[]) = self.focus {
                Action::Focused(min.y)
            } else {
                Action::Keep
            };
        }

        let mut load_with_default_open =
            egui::collapsing_header::CollapsingState::load_with_default_open(
                ui.ctx(),
                id,
                depth < 1,
            );
        if self.focus.is_some() {
            load_with_default_open.set_open(true)
        }
        let show: FoldRet<_, _> = load_with_default_open
            .show_header(ui, |ui| {
                // ui.label(format!("{}: {}", kind, label));
                ui.monospace(format!("{}", kind)).context_menu(|ui| {
                    ui.label(format!("{:?}", self.path));
                })
            })
            .body(|ui| {
                let o = self.store.children.cs_ofs[nid] as usize;
                let cs = &self.store.children.children
                    [o..o + self.store.children.cs_lens[nid] as usize]
                    .to_vec();
                self.children_ui(ui, depth, cs)
            })
            .into();
        // let show = egui::CollapsingHeader::new(format!("{}", kind))
        //     .id_source(id)
        //     .default_open(depth < 1)
        //     .show(ui, |ui| {
        //         if egui::collapsing_header::CollapsingState::load(ui.ctx(), id)
        //             .map_or(false, |x| x.is_open())
        //         {
        //             let o = self.store.children.cs_ofs[nid] as usize;
        //             let cs = &self.store.children.children
        //                 [o..o + self.store.children.cs_lens[nid] as usize]
        //                 .to_vec();
        //             self.children_ui(ui, depth, cs)
        //         } else {
        //             Action::Keep
        //         }
        //     });

        let mut prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
            prefill_cache
        } else {
            PrefillCache {
                head: 0.0,
                children: vec![],
                next: None,
            }
        };
        let mut rect = show.header_response.rect.union(
            show.body_response
                .as_ref()
                .map(|x| x.rect)
                .unwrap_or(egui::Rect::NOTHING),
        );
        rect.max.x += 10.0;
        prefill.head = show.header_response.rect.height();
        if DEBUG_LAYOUT {
            ui.painter().debug_rect(
                rect,
                egui::Color32::BLUE,
                format!("\t\t\t\t\t\t\t\t{:?}", show.header_response.rect),
            );
        }

        for (p, c, ret_pos) in &mut self.hightlights {
            selection_highlight(ui, *p, *c, min, rect, self.root_ui_id, *ret_pos);
        }

        // ui.label(format!("{:?}", show.body_response.map(|x| x.rect)));
        self.prefill_cache = Some(prefill);
        if show
            .header_returned
            .interact(egui::Sense::click())
            .clicked()
        {
            Action::Clicked(self.path.to_vec())
        } else if let Some(&[]) = self.focus {
            Action::Focused(min.y)
        } else {
            show.body_returned.unwrap_or(Action::Keep)
        }
    }

    pub(crate) fn ui_labeled_impl(
        &mut self,
        ui: &mut egui::Ui,
        _depth: usize,
        nid: usize,
    ) -> Action {
        let min = ui.available_rect_before_wrap().min;
        let kind = &self.store.type_sys.0[self.store.labeled.kinds[nid] as usize];
        let label = self.store.labeled.labels[nid];
        let label = &self.store.label_list[label as usize];
        let label = label
            .replace("\n", "\\n")
            .replace("\t", "\\t")
            .replace(" ", "·");
        self.draw_count += 1;
        let id = ui.id().with(&self.path);
        let action;
        let rect = if label.len() > 50 {
            if kind == "spaces" {
                let monospace = ui.monospace(format!("{}: ", kind)).context_menu(|ui| {
                    ui.label(format!("{:?}", self.path));
                });
                action = if monospace.interact(egui::Sense::click()).clicked() {
                    Action::Clicked(self.path.to_vec())
                } else {
                    Action::Keep
                };
                let rect1 = monospace.rect;
                let rect2 = ui.label(format!("{}", label)).rect;
                rect1.union(rect2)
            } else {
                let monospace = ui.monospace(format!("{}: ", kind)).context_menu(|ui| {
                    ui.label(format!("{:?}", self.path));
                });
                let rect1 = monospace.rect;
                let indent = ui.indent(id, |ui| {
                    ui.label(format!("{}", label)).context_menu(|ui| {
                        ui.label(format!("{:?}", self.path));
                    })
                });

                action = if monospace.interact(egui::Sense::click()).clicked()
                    || indent.inner.interact(egui::Sense::click()).clicked()
                {
                    Action::Clicked(self.path.to_vec())
                } else {
                    Action::Keep
                };
                let rect2 = indent.response.rect;
                rect1.union(rect2)
            }
        } else {
            action = Action::Keep;
            let add_contents = |ui: &mut egui::Ui| {
                ui.monospace(format!("{}: ", kind));
                ui.label(format!("{}", label));
            };
            if kind == "spaces" {
                if self.aspects.syntax {
                    ui.horizontal(add_contents).response.rect
                } else {
                    egui::Rect::from_min_max(min, min)
                }
            } else {
                ui.horizontal(add_contents).response.rect
            }
            
            
        };
        let mut prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
            prefill_cache
        } else {
            PrefillCache {
                head: 0.0,
                children: vec![],
                next: None,
            }
        };
        prefill.head = ui.available_rect_before_wrap().min.y - min.y;

        for (p, c, ret_pos) in &mut self.hightlights {
            if p.is_empty() {
                selection_highlight(ui, *p, *c, min, rect, self.root_ui_id, *ret_pos);
                // ui.painter().debug_rect(rect, **c, "");
            }
        }
        self.prefill_cache = Some(prefill);
        action
    }
    pub(crate) fn ui_typed_impl(&mut self, ui: &mut egui::Ui, _depth: usize, nid: usize) -> Action {
        let min = ui.available_rect_before_wrap().min;
        let kind = &self.store.type_sys.0[self.store.typed.kinds[nid] as usize];
        // ui.label(format!("k {}\t{}", kind, nid));
        self.draw_count += 1;
        ui.monospace(format!("{}", kind));
        let mut prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
            prefill_cache
        } else {
            PrefillCache {
                head: 0.0,
                children: vec![],
                next: None,
            }
        };
        prefill.head = ui.available_rect_before_wrap().min.y - min.y;
        self.prefill_cache = Some(prefill);
        Action::Keep
    }

    pub(crate) fn children_ui(&mut self, ui: &mut egui::Ui, depth: usize, cs: &[u64]) -> Action {
        let mut action = Action::Keep;
        // if depth > 5 {
        //     for c in cs {
        //         ui.label(c.to_string());
        //     }
        //     return Action::Keep;
        // }

        let mut prefill_old = if let Some(prefill_cache) = self.prefill_cache.take() {
            // wasm_rs_dbg::dbg!(
            //     &prefill_cache.head,
            //     &prefill_cache.children,
            //     &prefill_cache.next.is_some()
            // );
            prefill_cache
        } else {
            PrefillCache {
                head: 0.0,
                children: vec![],
                next: None,
            }
        };

        let mut prefill = PrefillCache {
            head: prefill_old.head,
            children: vec![],
            next: None,
        };
        for (i, c) in cs.iter().enumerate() {
            let mut path = self.path.clone();
            path.push(i);
            let rect = ui.available_rect_before_wrap();
            let focus = self.focus.as_ref().and_then(|x| {
                if x.is_empty() {
                    None
                } else if x[0] == i {
                    Some(&x[1..])
                } else {
                    None
                }
            });
            if self.focus.is_none()
                && rect.min.y > 0.0
                && ui.ctx().screen_rect().height() - CLIP_LEN < rect.min.y
            {
                break;
            }
            let hightlights: Vec<_> = self
                .hightlights
                .drain_filter(|(x, c, p)| {
                    !x.is_empty() && x[0] == i
                    // if x.is_empty() {
                    //     None
                    // } else if x[0] == i {
                    //     Some((&x[1..], *c, *p))
                    // } else {
                    //     None
                    // }
                })
                .map(|(x, c, p)| (&x[1..], c, p))
                .collect();
            let mut ignore = None;
            let mut imp = if let Some(child) = prefill_old.children.get(i) {
                let exact_max_y = rect.min.y + *child;
                if focus.is_none() && exact_max_y < CLIP_LEN {
                    ignore = Some(exact_max_y);
                    // FetchedViewImpl {
                    //     store: self.store,
                    //     prefill_cache: None,
                    //     min_before_count: 0,
                    //     draw_count: 0,
                    // }
                    prefill.children.push(*child);
                    if DEBUG_LAYOUT {
                        ui.painter().debug_rect(
                            egui::Rect::from_min_max(rect.min, (rect.max.x, exact_max_y).into()),
                            egui::Color32::RED,
                            format!(
                                "\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t{:?}\t{:?}\t{:?}",
                                exact_max_y, i, child
                            ),
                        );
                    }
                    ui.allocate_space((ui.min_size().x, *child).into());
                    continue;
                } else {
                    if DEBUG_LAYOUT {
                        ui.painter().debug_rect(
                            egui::Rect::from_min_max(rect.min, (rect.max.x, exact_max_y).into()),
                            egui::Color32::BLUE,
                            format!(
                                "\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t{:?}\t{:?}\t{:?}",
                                exact_max_y, i, child
                            ),
                        );
                    }
                    FetchedViewImpl {
                        store: self.store.clone(), // TODO perfs, might be better to pass cloned store between children
                        aspects: self.aspects,
                        prefill_cache: None,
                        min_before_count: 0,
                        draw_count: 0,
                        hightlights,
                        focus,
                        path,
                        root_ui_id: self.root_ui_id,
                    }
                }
            } else if i == prefill_old.children.len() {
                if DEBUG_LAYOUT {
                    ui.painter().debug_rect(
                        egui::Rect::from_min_max(rect.min, (rect.max.x, 200.0).into()),
                        egui::Color32::LIGHT_RED,
                        format!("\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t{:?}", i),
                    );
                }
                FetchedViewImpl {
                    store: self.store.clone(),
                    aspects: self.aspects,
                    prefill_cache: prefill_old.next.take().map(|b| *b),
                    min_before_count: 0,
                    draw_count: 0,
                    hightlights,
                    focus,
                    path,
                    root_ui_id: self.root_ui_id,
                }
            } else {
                FetchedViewImpl {
                    store: self.store.clone(),
                    aspects: self.aspects,
                    prefill_cache: None,
                    min_before_count: 0,
                    draw_count: 0,
                    hightlights,
                    focus,
                    path,
                    root_ui_id: self.root_ui_id,
                }
            };
            let ret = if let Some(c) = self.store.both.ids.iter().position(|x| x == c) {
                imp.ui_both_impl(ui, depth + 1, c)
            } else if let Some(c) = self.store.labeled.ids.iter().position(|x| x == c) {
                imp.ui_labeled_impl(ui, depth + 1, c)
            } else if let Some(c) = self.store.children.ids.iter().position(|x| x == c) {
                imp.ui_children_impl(ui, depth + 1, c)
            } else if let Some(c) = self.store.typed.ids.iter().position(|x| x == c) {
                imp.ui_typed_impl(ui, depth + 1, c)
            } else {
                let min = ui.available_rect_before_wrap().min;
                ui.label(format!("f {c}"));
                let head = ui.available_rect_before_wrap().min.y - min.y;
                imp.prefill_cache = Some(PrefillCache {
                    head: head,
                    children: vec![],
                    next: None,
                });
                Action::Keep
            };
            if let Action::Clicked(_) = ret {
                action = ret;
            } else if let Action::Focused(_) = ret {
                action = ret;
            };
            let c_cache = imp.prefill_cache.unwrap();
            let h = c_cache.height();
            if let Some(e_m_y) = ignore {
                prefill.children.push(h);
                if prefill_old.children.len() == i {
                    prefill.next = Some(Box::new(c_cache));
                }
                if DEBUG_LAYOUT {
                    ui.painter().debug_rect(
                egui::Rect::from_min_size(rect.min, (500.0-i as f32 * 3.0, e_m_y-rect.min.y).into()),
                egui::Color32::RED,
                format!(
                    "\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t{}\t{}\t{}\t{}",
                    h,rect.min.y, rect.min.y + h, e_m_y
                ),
            );
                }
                continue;
            }

            self.min_before_count += imp.min_before_count;
            self.draw_count += imp.draw_count;
            let mut color = egui::Color32::GOLD;
            if rect.min.y < CLIP_LEN && rect.min.y + h > CLIP_LEN {
                if c_cache.next.is_some() || !c_cache.children.is_empty() {
                    color = egui::Color32::BROWN;
                    prefill.next = Some(Box::new(c_cache))
                } else {
                    color = egui::Color32::DARK_RED;
                    prefill.children.push(h);
                }
            } else if prefill.next.is_none() {
                if rect.min.y > CLIP_LEN {
                    color = egui::Color32::LIGHT_GREEN;
                }
                prefill.children.push(h);
            }
            if DEBUG_LAYOUT {
                ui.painter().debug_rect(
                egui::Rect::from_min_size(rect.min, (500.0-i as f32 * 3.0, h).into()),
                color,
                format!(
                    "\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t{}\t{}\t{}",
                    h,rect.min.y, rect.min.y + h
                ),
            );
            }
        }
        self.prefill_cache = Some(prefill);
        action
    }
}

fn show_port(ui: &mut egui::Ui, id: egui::Id, pos: epaint::Pos2) {
    let area = egui::Area::new(id)
        .order(egui::Order::Middle)
        .constrain(true)
        .fixed_pos(pos)
        .interactable(false);
    area.show(ui.ctx(), |ui| {
        ui.add(egui_cable::prelude::Port::new(id));
    });
}

const DEBUG_LAYOUT: bool = false;
/// increase to debug and see culling in action
const CLIP_LEN: f32 = 0.0; //250.0;

fn subtree_to_layout(
    store: &FetchedHyperAST,
    theme: &syntax_highlighting_ts::CodeTheme,
    nid: u64,
) -> (usize, Vec<LayoutSection>) {
    use hyper_ast::nodes::CompressedNode;
    use std::borrow::Borrow;
    pub fn compute_layout_sections<
        IdN,
        IdL,
        T: Borrow<CompressedNode<IdN, IdL>>,
        F: Copy + Fn(&IdN) -> T,
        G: Copy + Fn(&IdL) -> usize,
    >(
        f: F,
        g: G,
        theme: &syntax_highlighting_ts::CodeTheme,
        id: &IdN,
        out: &mut Vec<LayoutSection>,
        offset: usize,
    ) -> usize {
        match f(id).borrow() {
            CompressedNode::Type(kind) => {
                let len = kind.to_string().len();
                let end = offset + len;
                let format = syntax_highlighting_ts::TokenType::Keyword;
                let mut format = theme.formats[format].clone();
                format.font_id = egui::FontId::monospace(12.0);
                out.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: offset..end.clone(),
                    format,
                });
                end
            }
            CompressedNode::Label { kind: _, label } => {
                let len = g(label);
                let end = offset + len;
                let format = syntax_highlighting_ts::TokenType::Punctuation;
                let mut format = theme.formats[format].clone();
                format.font_id = egui::FontId::monospace(12.0);
                out.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: offset..end.clone(),
                    format,
                });
                end
            }
            CompressedNode::Children2 { kind: _, children } => {
                unreachable!()
            }
            CompressedNode::Children { kind: _, children } => {
                let it = children.iter();
                let mut offset = offset;
                for id in it {
                    offset = compute_layout_sections(f, g, theme, &id, out, offset);
                }
                offset
            }
            CompressedNode::Spaces(s) => {
                let len = g(s);
                let end = offset + len;
                let format = syntax_highlighting_ts::TokenType::Punctuation;
                let mut format = theme.formats[format].clone();
                format.font_id = egui::FontId::monospace(12.0);
                out.push(LayoutSection {
                    leading_space: 0.0,
                    byte_range: offset..end.clone(),
                    format,
                });
                end
            }
        }
    }

    let mut layout = vec![];

    let len = compute_layout_sections(
        |id| -> _ {
            use hyper_ast::nodes::CompressedNode;
            let c = &(*id as u64);
            if let Some(c) = store.both.ids.iter().position(|x| x == c) {
                // imp.ui_both_impl(ui, depth + 1, c);
                panic!()
            } else if let Some(c) = store.labeled.ids.iter().position(|x| x == c) {
                // imp.ui_labeled_impl(ui, depth + 1, c);
                let s = &store.type_sys.0[store.labeled.kinds[c] as usize];
                let kind = if s.starts_with("xml") {
                    Type::parse_xml(s)
                } else {
                    Type::parse(s).unwrap()
                };
                let l = store.labeled.labels[c] as usize;
                CompressedNode::Label { label: l, kind }
            } else if let Some(c) = store.children.ids.iter().position(|x| x == c) {
                // imp.ui_children_impl(ui, depth + 1, c);
                let s = &store.type_sys.0[store.children.kinds[c] as usize];
                let kind = if s.starts_with("xml") {
                    Type::parse_xml(s)
                } else {
                    Type::parse(s).unwrap()
                };
                let o = store.children.cs_ofs[c] as usize;
                let children = &store.children.children[o..o + store.children.cs_lens[c] as usize];
                CompressedNode::Children {
                    children: children.iter().map(|x| *x as u64).collect(),
                    kind,
                }
            } else if let Some(c) = store.typed.ids.iter().position(|x| x == c) {
                // imp.ui_typed_impl(ui, depth + 1, c);
                let s = &store.type_sys.0[store.typed.kinds[c] as usize];
                if s.starts_with("xml") {
                    CompressedNode::Type(Type::parse_xml(s))
                } else {
                    CompressedNode::Type(Type::parse(s).unwrap())
                }
            } else {
                CompressedNode::Type(Type::Error)
            }
        },
        |id| -> _ { store.label_list[*id].len() },
        theme,
        &nid,
        &mut layout,
        0,
    );
    (len, layout)
}

fn subtree_to_string(store: &FetchedHyperAST, nid: u64) -> String {
    let mut out = BuffOut::default();

    #[derive(Default)]
    struct BuffOut {
        buff: String,
    }

    impl std::fmt::Write for BuffOut {
        fn write_str(&mut self, s: &str) -> std::fmt::Result {
            Ok(self.buff.extend(s.chars()))
        }
    }

    impl From<BuffOut> for String {
        fn from(value: BuffOut) -> Self {
            value.buff
        }
    }

    hyper_ast::nodes::serialize(
        |id| -> _ {
            use hyper_ast::nodes::CompressedNode;
            let c = &(*id as u64);
            if let Some(c) = store.both.ids.iter().position(|x| x == c) {
                // imp.ui_both_impl(ui, depth + 1, c, pid.with(i));
                panic!()
            } else if let Some(c) = store.labeled.ids.iter().position(|x| x == c) {
                // imp.ui_labeled_impl(ui, depth + 1, c, pid.with(i));
                let s = &store.type_sys.0[store.labeled.kinds[c] as usize];
                let kind = if s.starts_with("xml") {
                    Type::parse_xml(s)
                } else {
                    Type::parse(s).unwrap()
                };
                let l = store.labeled.labels[c] as usize;
                CompressedNode::Label { label: l, kind }
            } else if let Some(c) = store.children.ids.iter().position(|x| x == c) {
                // imp.ui_children_impl(ui, depth + 1, c, pid.with(i));
                let s = &store.type_sys.0[store.children.kinds[c] as usize];
                let kind = if s.starts_with("xml") {
                    Type::parse_xml(s)
                } else {
                    Type::parse(s).unwrap()
                };
                let o = store.children.cs_ofs[c] as usize;
                let children = &store.children.children[o..o + store.children.cs_lens[c] as usize];
                CompressedNode::Children {
                    children: children.iter().map(|x| *x as u64).collect(),
                    kind,
                }
            } else if let Some(c) = store.typed.ids.iter().position(|x| x == c) {
                // imp.ui_typed_impl(ui, depth + 1, c);
                let s = &store.type_sys.0[store.typed.kinds[c] as usize];
                if s.starts_with("xml") {
                    CompressedNode::Type(Type::parse_xml(s))
                } else {
                    CompressedNode::Type(Type::parse(s).unwrap())
                }
            } else {
                CompressedNode::Type(Type::Error)
            }
            // node_store
            //     .resolve(id.clone())
            //     .into_compressed_node()
            //     .unwrap()
        },
        |id| -> _ { store.label_list[*id].clone() },
        &nid,
        &mut out,
        "\n",
    );
    out.buff
}

fn make_pp_code(
    store: Arc<FetchedHyperAST>,
    ctx: &egui::Context,
    nid: usize,
    theme: syntax_highlighting_ts::CodeTheme,
) -> epaint::text::LayoutJob {
    #[derive(Default)]
    struct PrettyPrinter {}
    impl cache::ComputerMut<(&FetchedHyperAST, u64), String> for PrettyPrinter {
        fn compute(&mut self, (store, id): (&FetchedHyperAST, u64)) -> String {
            subtree_to_string(store, id)
        }
    }

    type PPCache = cache::FrameCache<String, PrettyPrinter>;

    let code = ctx.memory_mut(|mem| {
        mem.caches
            .cache::<PPCache>()
            .get((store.as_ref(), store.children.ids[nid]))
    });
    #[derive(Default)]
    struct Spawner {}
    impl
        syntax_highlighting_async::cache::Spawner<
            (
                Arc<FetchedHyperAST>,
                &syntax_highlighting_ts::CodeTheme,
                u64,
                usize,
            ),
            Layouter,
        > for Spawner
    {
        fn spawn(
            &self,
            ctx: &egui::Context,
            (store, theme, id, len): (
                Arc<FetchedHyperAST>,
                &syntax_highlighting_ts::CodeTheme,
                u64,
                usize,
            ),
        ) -> Layouter {
            Layouter {
                ctx: ctx.clone(),
                total_str_len: len,
                ..Default::default()
            }
        }
    }
    use std::sync::atomic::Ordering;
    use std::sync::Mutex;
    #[derive(Default)]
    struct Layouter {
        ctx: egui::Context,
        mt: Vec<Arc<Mutex<async_exec::TimeoutHandle>>>,
        sections: Vec<LayoutSection>,
        /// remaining, queue
        queued: Arc<(AtomicUsize, crossbeam_queue::SegQueue<Vec<LayoutSection>>)>,
        i: usize,
        total_str_len: usize,
    }
    impl
        syntax_highlighting_async::cache::IncrementalComputer<
            Spawner,
            (
                Arc<FetchedHyperAST>,
                &syntax_highlighting_ts::CodeTheme,
                u64,
                usize,
            ),
            Vec<LayoutSection>,
        > for Layouter
    {
        fn increment(
            &mut self,
            spawner: &Spawner,
            (store, theme, id, len): (
                Arc<FetchedHyperAST>,
                &syntax_highlighting_ts::CodeTheme,
                u64,
                usize,
            ),
        ) -> Vec<LayoutSection> {
            let theme = theme.clone();
            assert_eq!(len, self.total_str_len);
            if self.mt.is_empty() && self.i < self.total_str_len {
                let h = self.queued.clone();
                let ctx = self.ctx.clone();
                let fut = move || {
                    let (len, sections) = subtree_to_layout(store.as_ref(), &theme, id);
                    h.1.push(sections);
                    h.0.store(len, Ordering::Relaxed);
                    ctx.request_repaint_after(Duration::from_millis(10));
                };
                self.mt
                    .push(Arc::new(Mutex::new(async_exec::spawn_macrotask(Box::new(
                        fut,
                    )))));
                vec![LayoutSection {
                    leading_space: 0.0,
                    byte_range: 0..self.total_str_len,
                    format: TextFormat {
                        font_id: egui::FontId::monospace(12.0),
                        ..Default::default()
                    },
                }]
            } else if self.i < self.total_str_len {
                self.i = self.queued.as_ref().0.load(Ordering::Relaxed);
                for _ in 0..self.queued.as_ref().1.len() {
                    let sections = self.queued.as_ref().1.pop();
                    if let Some(sections) = sections {
                        self.sections.extend_from_slice(&sections);
                    }
                }
                let mut sections = self.sections.clone();

                if self.i < self.total_str_len {
                    sections.push(LayoutSection {
                        leading_space: 0.0,
                        byte_range: self.i..self.total_str_len,
                        format: TextFormat {
                            font_id: egui::FontId::monospace(12.0),
                            ..Default::default()
                        },
                    })
                }

                sections
            } else {
                self.mt.clear();
                self.sections.clone()
            }
        }
    }

    type HCache = syntax_highlighting_async::cache::IncrementalCache<Layouter, Spawner>;

    let sections = ctx.memory_mut(|mem| {
        mem.caches.cache::<HCache>().get(
            ctx,
            (store.clone(), &theme, store.children.ids[nid], code.len()),
        )
    });

    let layout_job = epaint::text::LayoutJob {
        text: code,
        sections,
        ..epaint::text::LayoutJob::default()
    };
    layout_job
}

fn selection_highlight(
    ui: &mut egui::Ui,
    path: &[usize],
    color: &epaint::Color32,
    min: epaint::Pos2,
    rect: epaint::Rect,
    root_ui_id: egui::Id,
    ret_pos: &mut Option<egui::Pos2>,
) {
    if path.is_empty() {
        let clip = ui.clip_rect();
        let min_elem = clip.size().min_elem();
        if clip.intersects(rect) {
            ui.painter().debug_rect(rect, *color, "");
        }
        let clip = if min_elem < 1.0 {
            clip
        } else {
            let mut clip = clip.shrink((min_elem / 2.0).min(4.0));
            clip.set_width((clip.width() - 14.0).max(0.0));
            clip
        };

        if clip.intersects(rect) {
            if color == &egui::Color32::BLUE {
                let id = root_ui_id.with("blue_highlight");
                wasm_rs_dbg::dbg!("green", id);
                let pos = egui::pos2(min.x - 15.0, min.y - 10.0);
                let pos = clip.clamp(pos);
                if ui.clip_rect().contains(pos) {
                    show_port(ui, id, pos);
                    *ret_pos = Some(pos);
                }
            } else if color == &egui::Color32::GREEN {
                let id = root_ui_id.with("green_highlight");
                let pos = egui::pos2(rect.max.x - 10.0, rect.min.y - 10.0);
                let pos = clip.clamp(pos);
                if ui.clip_rect().contains(pos) {
                    show_port(ui, id, pos);
                    *ret_pos = Some(pos);
                }
            }

            ui.painter().debug_rect(rect, *color, "");
        }
    }
}
