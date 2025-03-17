use super::{
    code_aspects::{remote_fetch_labels, remote_fetch_nodes_by_ids, HightLightHandle},
    long_tracking::TARGET_COLOR,
};
use crate::app::syntax_highlighting::{self as syntax_highlighter, syntax_highlighting_async};
use egui::TextFormat;
use epaint::text::LayoutSection;
pub use hyperast::store::nodes::fetched::NodeIdentifier;
use hyperast::{
    nodes::IndentedAlt,
    store::nodes::fetched::LabelIdentifier,
    types::{AnyType, HyperType, Labeled, WithChildren, WithStats},
};
use std::{
    fmt::Debug,
    num::NonZeroU32,
    ops::ControlFlow,
    sync::{atomic::AtomicUsize, Arc},
    time::Duration,
};
mod cache;

pub(crate) mod store {
    pub use hyperast::store::nodes::fetched::NodeIdentifier;
    use hyperast::store::nodes::fetched::NodeStore;
    use hyperast::types::Lang;
    use hyperast::{
        store::nodes::fetched::{FetchedLabels, HashedNodeRef, LabelIdentifier},
        types::AnyType,
    };
    use std::borrow::Borrow;
    use std::collections::HashSet;
    use std::collections::VecDeque;

    #[derive(Default)]
    pub(crate) struct TStore;

    impl<'a> hyperast::types::TypeStore for TStore {
        type Ty = AnyType;
    }

    #[derive(Default)]
    pub struct FetchedHyperAST {
        pub(crate) label_store: std::sync::RwLock<FetchedLabels>,
        pub(crate) node_store: std::sync::RwLock<NodeStore>,
        // /// each set is fetched sequentially, non blocking
        // /// pushed ids are tested against all pending sets because they might not have entered the store
        // /// new set every 100 elements, due to id serialized size in url
        // /// TODO split by arch
        // /// TODO maybe use a crossbeam queue while putting a dummy value in nodestore or use dashmap
        // nodes_waiting: std::sync::Mutex<VecDeque<HashSet<NodeIdentifier>>>,
        // /// each set is fetched sequentially, non blocking
        // /// pushed ids are tested against all pending sets because they might not have entered the store
        // /// new set every 200 elements, due to id serialized size in url
        // labels_waiting: std::sync::Mutex<VecDeque<HashSet<LabelIdentifier>>>,
        /// pending ie. nodes in flight
        pub(crate) nodes_pending: std::sync::Mutex<VecDeque<HashSet<NodeIdentifier>>>,
        pub(crate) nodes_waiting: std::sync::Mutex<Option<HashSet<NodeIdentifier>>>,
        pub(crate) labels_pending: std::sync::Mutex<VecDeque<HashSet<LabelIdentifier>>>,
        pub(crate) labels_waiting: std::sync::Mutex<Option<HashSet<LabelIdentifier>>>,
        /// timer to avoid flooding
        pub(crate) timer: std::sync::Mutex<Option<f32>>,
    }

    impl FetchedHyperAST {
        pub(crate) fn read(&self) -> AcessibleFetchedHyperAST<'_> {
            AcessibleFetchedHyperAST {
                label_store: self.label_store.read().unwrap(),
                node_store: self.node_store.read().unwrap(),
                nodes_pending: self.nodes_pending.lock().unwrap(),
                nodes_waiting: std::cell::RefCell::new(self.nodes_waiting.lock().unwrap()),
                labels_pending: self.labels_pending.lock().unwrap(),
                labels_waiting: std::cell::RefCell::new(self.labels_waiting.lock().unwrap()),
            }
        }
        pub fn resolve_type(&self, n: &NodeIdentifier) -> AnyType {
            let ns = self.node_store.read().unwrap();
            let n: HashedNodeRef<'_, NodeIdentifier> = ns.try_resolve(*n).unwrap();
            let lang = n.get_lang();
            let raw = n.get_raw_type();
            match lang {
                "hyperast_gen_ts_java::types::Lang" => {
                    let t: &'static dyn hyperast::types::HyperType =
                        hyperast_gen_ts_java::types::Lang::make(raw);
                    t.into()
                }
                "hyperast_gen_ts_cpp::types::Lang" => {
                    let t: &'static dyn hyperast::types::HyperType =
                        hyperast_gen_ts_cpp::types::Lang::make(raw);
                    t.into()
                }
                "hyperast_gen_ts_xml::types::Lang" => {
                    let t: &'static dyn hyperast::types::HyperType =
                        hyperast_gen_ts_xml::types::Lang::make(raw);
                    t.into()
                }
                l => unreachable!("{}", l),
            }
        }
    }

    pub(crate) struct AcessibleFetchedHyperAST<'a> {
        pub(crate) label_store: std::sync::RwLockReadGuard<'a, FetchedLabels>,
        pub(crate) node_store: std::sync::RwLockReadGuard<'a, NodeStore>,
        pub(crate) nodes_pending: std::sync::MutexGuard<'a, VecDeque<HashSet<NodeIdentifier>>>,
        pub(crate) nodes_waiting:
            std::cell::RefCell<std::sync::MutexGuard<'a, Option<HashSet<NodeIdentifier>>>>,
        pub(crate) labels_pending: std::sync::MutexGuard<'a, VecDeque<HashSet<LabelIdentifier>>>,
        pub(crate) labels_waiting:
            std::cell::RefCell<std::sync::MutexGuard<'a, Option<HashSet<LabelIdentifier>>>>,
    }

    // impl<'b> hyperast::types::NodStore<NodeIdentifier> for AcessibleFetchedHyperAST<'b> {
    //     type R<'a> = HashedNodeRef<'a, NodeIdentifier>;
    // }
    impl<'a, 'b> hyperast::types::NLending<'a, NodeIdentifier> for AcessibleFetchedHyperAST<'b> {
        type N = HashedNodeRef<'a, NodeIdentifier>;
    }

    impl<'b> hyperast::types::NodeStore<NodeIdentifier> for AcessibleFetchedHyperAST<'b> {
        fn resolve(&self, id: &NodeIdentifier) -> <Self as hyperast::types::NLending<'_, NodeIdentifier>>::N {
            if let Some(r) = self.node_store.try_resolve(*id) {
                r
            } else {
                // TODO use a recursive fetch
                // TODO need an additional queue for such recursive fetch
                // TODO use additional nodes that are not fetched but where fetched to avoid transfering more than necessary
                if !self.nodes_pending.iter().any(|x| x.contains(id)) {
                    self.nodes_waiting
                        .borrow_mut()
                        .get_or_insert(Default::default())
                        .insert(*id);
                }
                // unimplemented!()
                self.node_store.unavailable_node()
            }
        }
    }

    impl<'b> hyperast::types::inner_ref::NodeStore<NodeIdentifier> for AcessibleFetchedHyperAST<'b> {
        type Ref = HashedNodeRef<'static, NodeIdentifier>;
        fn scoped<R>(&self, id: &NodeIdentifier, f: impl Fn(&Self::Ref) -> R) -> R {
            let t = &hyperast::types::NodeStore::resolve(self, id);
            // SAFETY: safe as long as Self::Ref does not exposes its fake &'static fields
            let t = unsafe { std::mem::transmute(t) };
            f(t)
        }
        fn scoped_mut<R>(&self, id: &NodeIdentifier, mut f: impl FnMut(&Self::Ref) -> R) -> R {
            let t = &hyperast::types::NodeStore::resolve(self, id);
            // SAFETY: safe as long as Self::Ref does not exposes its fake &'static fields
            let t = unsafe { std::mem::transmute(t) };
            f(t)
        }
        fn multi<R, const N: usize>(
            &self,
            id: &[NodeIdentifier; N],
            f: impl Fn(&[Self::Ref; N]) -> R,
        ) -> R {
            todo!()
        }
    }

    impl<'b> hyperast::types::LabelStore<str> for AcessibleFetchedHyperAST<'b> {
        type I = LabelIdentifier;

        fn get_or_insert<U: Borrow<str>>(&mut self, _node: U) -> Self::I {
            todo!("TODO remove this method from trait as it cannot be implemented on immutable/append_only label stores")
        }

        fn get<U: Borrow<str>>(&self, _node: U) -> Option<Self::I> {
            todo!("TODO remove this method from trait as it cannot be implemented efficiently for all stores")
        }

        fn resolve(&self, id: &Self::I) -> &str {
            if let Some(get) = self.label_store.try_resolve(id) {
                get
            } else {
                if !self.labels_pending.iter().any(|x| x.contains(id)) {
                    self.labels_waiting
                        .borrow_mut()
                        .get_or_insert(Default::default())
                        .insert(*id);
                }
                "."
            }
        }
    }

    impl<'a, 'b> hyperast::types::TypeStore for AcessibleFetchedHyperAST<'b> {
        type Ty = AnyType;
    }

    impl<'a, 'b: 'a> hyperast::types::HyperASTShared for AcessibleFetchedHyperAST<'a>
    where
        Self: 'b,
    {
        type IdN = NodeIdentifier;

        type Idx = u16;

        type Label = LabelIdentifier;
        // type T<'store> = HashedNodeRef<'store, NodeIdentifier>;
        // type RT = HashedNodeRef<'static, NodeIdentifier>;
    }

    // impl<'a> hyperast::types::HyperASTAsso for AcessibleFetchedHyperAST<'a> {
    //     type NS<'store>
    //         = Self
    //     where
    //         Self: 'store;

    //     fn node_store<'c>(&'c self) -> &'c Self::NS<'c> {
    //         self
    //     }

    //     type LS = Self;

    //     fn label_store(&self) -> &Self::LS {
    //         self
    //     }

    //     type TS<'store>
    //         = Self
    //     where
    //         Self: 'store;

    //     fn resolve_type(&self, id: &Self::IdN) -> <Self::TS<'_> as hyperast::types::TypeStore>::Ty {
    //         let ns = &self.node_store;
    //         let Some(n) = ns.try_resolve::<NodeIdentifier>(*id) else {
    //             use hyperast::types::HyperType;
    //             return hyperast_gen_ts_java::types::Type::Dot.as_static().into();
    //         };
    //         let lang = n.get_lang();
    //         let raw = n.get_raw_type();
    //         match lang {
    //             "hyperast_gen_ts_java::types::Lang" => {
    //                 let t: &'static dyn hyperast::types::HyperType =
    //                     hyperast_gen_ts_java::types::Lang::make(raw);
    //                 t.into()
    //             }
    //             "hyperast_gen_ts_cpp::types::Lang" => {
    //                 let t: &'static dyn hyperast::types::HyperType =
    //                     hyperast_gen_ts_cpp::types::Lang::make(raw);
    //                 t.into()
    //             }
    //             "hyperast_gen_ts_xml::types::Lang" => {
    //                 let t: &'static dyn hyperast::types::HyperType =
    //                     hyperast_gen_ts_xml::types::Lang::make(raw);
    //                 t.into()
    //             }
    //             l => unreachable!("{}", l),
    //         }
    //     }
    // }

    impl<'a, 'b> hyperast::types::AstLending<'a> for AcessibleFetchedHyperAST<'b> {
        type RT = HashedNodeRef<'a, NodeIdentifier>;
    }

    impl<'a> hyperast::types::HyperAST for AcessibleFetchedHyperAST<'a> {
        type NS = Self;

        fn node_store(&self) -> &Self::NS {
            self
        }

        type LS = Self;

        fn label_store(&self) -> &Self::LS {
            self
        }

        type TS = Self;

        fn resolve_type(&self, id: &Self::IdN) -> <Self::TS as hyperast::types::TypeStore>::Ty {
            let ns = &self.node_store;
            let Some(n) = ns.try_resolve::<NodeIdentifier>(*id) else {
                use hyperast::types::HyperType;
                return hyperast_gen_ts_java::types::Type::Dot.as_static().into();
            };
            let lang = n.get_lang();
            let raw = n.get_raw_type();
            match lang {
                "hyperast_gen_ts_java::types::Lang" => {
                    let t: &'static dyn hyperast::types::HyperType =
                        hyperast_gen_ts_java::types::Lang::make(raw);
                    t.into()
                }
                "hyperast_gen_ts_cpp::types::Lang" => {
                    let t: &'static dyn hyperast::types::HyperType =
                        hyperast_gen_ts_cpp::types::Lang::make(raw);
                    t.into()
                }
                "hyperast_gen_ts_xml::types::Lang" => {
                    let t: &'static dyn hyperast::types::HyperType =
                        hyperast_gen_ts_xml::types::Lang::make(raw);
                    t.into()
                }
                l => unreachable!("{}", l),
            }
            // self.resolve_type(id)
            // let ns = self.node_store();
            // let n = ns.resolve(id);
            // Self::TS::decompress_type(&n, std::any::TypeId::of::<<Self::TS as TypeStore>::Ty>())
        }
    }

    impl std::hash::Hash for FetchedHyperAST {
        fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
            self.label_store.read().unwrap().len().hash(state);
            self.node_store.read().unwrap().len().hash(state);
        }
    }
}

#[derive(Debug)]
pub struct PrefillCache {
    head: f32,
    children: Vec<f32>,
    children_sizes: Vec<Option<NonZeroU32>>,
    next: Option<Box<PrefillCache>>,
}

impl PrefillCache {
    fn height(&self) -> f32 {
        self.head
            + self.children.iter().sum::<f32>()
            + self.next.as_ref().map_or(0.0, |x| x.height())
    }
}
#[derive(Clone, Debug, Default)]
pub(crate) enum Action {
    #[default]
    Keep,
    SerializeKind(AnyType),
    HideKind(AnyType),
    PartialFocused(f32),
    Focused(f32),
    Clicked(Vec<usize>),
}

pub(crate) struct FetchedViewImpl<'a> {
    store: Arc<store::FetchedHyperAST>,
    aspects: &'a super::types::ComputeConfigAspectViews,
    pub(super) prefill_cache: Option<PrefillCache>,
    min_before_count: usize,
    draw_count: usize,
    hightlights: Vec<HightLightHandle<'a>>,
    focus: Option<(&'a [usize], &'a [NodeIdentifier])>,
    path: Vec<usize>,
    root_ui_id: egui::Id,
    additions: Option<&'a [u32]>,
    deletions: Option<&'a [u32]>,
    global_pos: Option<u32>,
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
    #[allow(unused)]
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
        store: Arc<store::FetchedHyperAST>,
        aspects: &'a super::types::ComputeConfigAspectViews,
        take: Option<PrefillCache>,
        hightlights: Vec<HightLightHandle<'a>>,
        focus: Option<(&'a [usize], &'a [NodeIdentifier])>,
        path: Vec<usize>,
        root_ui_id: egui::Id,
        additions: Option<&'a [u32]>,
        deletions: Option<&'a [u32]>,
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
            additions,
            deletions,
            global_pos: None,
        }
    }

    pub fn show(&mut self, ui: &mut egui::Ui, api_addr: &str, root: &NodeIdentifier) -> Action {
        ui.style_mut().spacing.button_padding.y = 0.0;
        ui.style_mut().spacing.item_spacing.y = 0.0;

        let node_store = self.store.node_store.read().unwrap();
        // wasm_rs_dbg::dbg!(root);
        let action = if let Some(r) = node_store.try_resolve::<NodeIdentifier>(*root) {
            // let lang = r.get_lang();
            // gen::types::Type::Lang;
            let kind = self.store.resolve_type(&root);
            let l = r.try_get_label().copied();
            let cs = r.children();
            let size = r.size();
            self.global_pos = Some(size as u32);
            if let Some(cs) = cs {
                if let Some(label) = l {
                    let cs = cs.0.to_vec();
                    // NOTE: Why would it be an issue ?
                    // if let Some(label) = self.store.label_store.read().unwrap().try_resolve(&label) {
                    //     assert_eq!("", label, "{:?} {:?} {:?}", root, cs.len(), node_store);
                    // }
                    drop(node_store);
                    self.ui_both_impl(ui, kind, size as u32, label, cs.as_ref())
                } else {
                    let cs = cs.0.to_vec();
                    drop(node_store);
                    self.ui_children_impl2(ui, kind, size as u32, *root, cs.as_ref())
                }
            } else if let Some(label) = l {
                drop(node_store);
                self.ui_labeled_impl2(ui, kind, size as u32, *root, label)
            } else {
                drop(node_store);
                self.ui_typed_impl2(ui, kind, size as u32)
            }
        } else {
            if !self
                .store
                .nodes_pending
                .lock()
                .unwrap()
                .iter()
                .any(|x| x.contains(root))
            {
                self.store
                    .nodes_waiting
                    .lock()
                    .unwrap()
                    .get_or_insert(Default::default())
                    .insert(*root);
            }
            Action::Keep
        };

        let mut lock = self.store.timer.lock().unwrap();
        if let Some(mut timer) = lock.take() {
            let dt = ui.input(|mem| mem.unstable_dt);
            timer += dt;
            // wasm_rs_dbg::dbg!(dt, timer, Duration::from_secs(2).as_secs_f32());
            if timer < Duration::from_secs(1).as_secs_f32() {
                *lock = Some(timer);
                return action;
            } else {
                *lock = Some(0.0);
            }
        } else {
            *lock = Some(0.0);
            return action;
        }
        drop(lock);

        if let Some(waiting) = self.store.nodes_waiting.lock().unwrap().take() {
            self.store
                .nodes_pending
                .lock()
                .unwrap()
                .push_back(waiting.clone());
            remote_fetch_nodes_by_ids(
                ui.ctx(),
                api_addr,
                self.store.clone(),
                &self.aspects.commit.repo,
                waiting,
            )
            .ready();
            // TODO need to use promise ?
        };
        if let Some(waiting) = self.store.labels_waiting.lock().unwrap().take() {
            self.store
                .labels_pending
                .lock()
                .unwrap()
                .push_back(waiting.clone());
            remote_fetch_labels(
                ui.ctx(),
                api_addr,
                self.store.clone(),
                &self.aspects.commit.repo,
                waiting,
            )
            .ready();
            // TODO need to use promise ?
        };
        action
    }

    fn ui_both_impl(
        &mut self,
        ui: &mut egui::Ui,
        kind: AnyType,
        size: u32,
        label: LabelIdentifier,
        // depth: usize,
        cs: &[NodeIdentifier],
    ) -> Action {
        if self.is_hidden(kind) {
            let prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
                prefill_cache
            } else {
                PrefillCache {
                    head: 0.0,
                    children: vec![],
                    children_sizes: vec![],
                    next: None,
                }
            };
            self.prefill_cache = Some(prefill);
            return Action::Keep;
        }
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
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
        if self.focus.is_some() {
            // wasm_rs_dbg::dbg!(&self.focus, &self.path, &cs);
            load_with_default_open.set_open(true)
        }

        self.additions_deletions_compute(size);
        ui.spacing_mut().icon_spacing /= 2.0;
        let show: FoldRet<_, _> = load_with_default_open
            .show_header(ui, |ui| {
                // ui.label(format!("{}: {}", kind, label));
                let label_store = self.store.label_store.read().unwrap();
                let label = if let Some(label) = label_store.try_resolve(&label) {
                    label
                } else {
                    if !self
                        .store
                        .labels_pending
                        .lock()
                        .unwrap()
                        .iter()
                        .any(|x| x.contains(&label))
                    {
                        self.store
                            .labels_waiting
                            .lock()
                            .unwrap()
                            .get_or_insert(Default::default())
                            .insert(label);
                    }
                    "..."
                };
                let mut label = egui::RichText::new(label);
                let ret = {
                    let text = format!("{}: ", kind);
                    let mut rt = egui::RichText::new(text).monospace();
                    // wasm_rs_dbg::dbg!(&self.global_pos);
                    if let Some(gp) = &self.global_pos {
                        if self.additions.is_some() || self.deletions.is_some() {
                            let add = self.additions.unwrap_or_default();
                            let del = self.deletions.unwrap_or_default();
                            // wasm_rs_dbg::dbg!(add, del);
                            if add.is_empty() && del.is_empty() {
                                label = label.size(8.0);
                                label = label.color(egui::Color32::GRAY);
                                rt = rt.size(8.0);
                                rt = rt.color(egui::Color32::GRAY);
                            } else if add.last() == Some(gp) {
                                if del.last() == Some(gp) {
                                    rt = rt.color(egui::Color32::DARK_BLUE);
                                } else {
                                    rt = rt.color(egui::Color32::DARK_GREEN);
                                }
                            } else if del.last() == Some(gp) {
                                // wasm_rs_dbg::dbg!(del, gp);
                                // rt = rt.strikethrough();
                                rt = rt.color(egui::Color32::DARK_RED);
                            }
                        }
                    }
                    // if self
                    //     .additions
                    //     .map_or(false, |x| x.contains(&self.global_pos))
                    // {
                    //     if self
                    //         .deletions
                    //         .map_or(false, |x| x.contains(&self.global_pos))
                    //     {
                    //         rt = rt.color(egui::Color32::BLUE);
                    //     } else {
                    //         rt = rt.color(egui::Color32::GREEN);
                    //     }
                    // } else if self
                    //     .deletions
                    //     .map_or(false, |x| x.contains(&self.global_pos))
                    // {
                    //     rt = rt.color(egui::Color32::RED);
                    // }
                    ui.label(rt)
                }
                // .context_menu(|ui| {
                //     ui.label(format!("{:?}", cs));
                //     ui.label(format!("{:?}", self.path));
                // })
                ;
                ui.label(label);
                ret
            })
            .body(|ui| self.children_ui(ui, cs, self.global_pos.map(|x| x - size)))
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
                children_sizes: vec![],
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

        for handle in &mut self.hightlights {
            selection_highlight(ui, handle, min, rect, self.root_ui_id);
        }
        // ui.label(format!("{:?}", show.body_response.map(|x| x.rect)));
        self.prefill_cache = Some(prefill);

        if show
            .header_returned
            .interact(egui::Sense::click())
            .clicked()
        {
            Action::Clicked(self.path.to_vec())
        } else if let Some((&[], _)) = self.focus {
            Action::Focused(min.y)
        } else {
            show.body_returned.unwrap_or(Action::Keep)
        }
    }

    fn ui_children_impl2(
        &mut self,
        ui: &mut egui::Ui,
        kind: AnyType,
        size: u32,
        nid: NodeIdentifier,
        cs: &[NodeIdentifier],
    ) -> Action {
        if self.is_hidden(kind) {
            let prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
                prefill_cache
            } else {
                PrefillCache {
                    head: 0.0,
                    children: vec![],
                    children_sizes: vec![],
                    next: None,
                }
            };
            self.prefill_cache = Some(prefill);
            return Action::Keep;
        }
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
        self.draw_count += 1;
        let id = ui.id().with(&self.path);
        // let id = ui.make_persistent_id("my_collapsing_header");

        self.additions_deletions_compute(size);
        if self.is_pp(kind) {
            let action = self.show_pp(ui, nid);
            return action;
        }

        let mut load_with_default_open =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
        if self.focus.is_some() {
            load_with_default_open.set_open(true)
        }
        let show: FoldRet<_, _> = load_with_default_open
            .show_header(ui, |ui| {
                // ui.label(format!("{}: {}", kind, label));
                {
                    let text = format!("{}: ", kind);
                    let mut rt = egui::RichText::new(text).monospace();
                    if let Some(gp) = &self.global_pos {
                        if self.additions.is_some() || self.deletions.is_some() {
                            let add = self.additions.unwrap_or_default();
                            let del = self.deletions.unwrap_or_default();
                            // wasm_rs_dbg::dbg!(add, del);
                            if add.is_empty() && del.is_empty() {
                                rt = rt.size(8.0);
                                rt = rt.color(egui::Color32::GRAY);
                            } else if add.last() == Some(gp) {
                                if del.last() == Some(gp) {
                                    rt = rt.color(egui::Color32::DARK_BLUE);
                                } else {
                                    rt = rt.color(egui::Color32::DARK_GREEN);
                                }
                            } else if del.last() == Some(gp) {
                                // rt = rt.underline();
                                rt = rt.color(egui::Color32::DARK_RED);
                            }
                        }
                    }
                    // if self
                    //     .additions
                    //     .map_or(false, |x| x.contains(&self.global_pos))
                    // {
                    //     if self
                    //         .deletions
                    //         .map_or(false, |x| x.contains(&self.global_pos))
                    //     {
                    //         rt = rt.color(egui::Color32::BLUE);
                    //     } else {
                    //         rt = rt.color(egui::Color32::GREEN);
                    //     }
                    // } else if self
                    //     .deletions
                    //     .map_or(false, |x| x.contains(&self.global_pos))
                    // {
                    //     rt = rt.color(egui::Color32::RED);
                    // }
                    ui.label(rt)
                }
                // .context_menu(|ui| {
                //     ui.label(format!("{:?}", self.path));

                //     if let Some(gp) = &self.global_pos {
                //         if self.additions.is_some() || self.deletions.is_some() {
                //             let add = self.additions.unwrap_or_default();
                //             let del = self.deletions.unwrap_or_default();
                //             // wasm_rs_dbg::dbg!(add, del);
                //             ui.label(format!("{:?}", add));
                //             ui.label(format!("{:?}", del));
                //         }
                //     }
                // })
            })
            .body(|ui| self.children_ui(ui, cs, self.global_pos.map(|x| x - size)))
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
                children_sizes: vec![],
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

        for handle in &mut self.hightlights {
            selection_highlight(ui, handle, min, rect, self.root_ui_id);
        }

        // ui.label(format!("{:?}", show.body_response.map(|x| x.rect)));
        self.prefill_cache = Some(prefill);

        let interact = show.header_returned.interact(egui::Sense::click_and_drag());

        if interact.drag_released() {
            node_menu(ui, interact, kind)
                .or(show.body_returned)
                .unwrap_or_default()
        } else if interact.clicked() {
            Action::Clicked(self.path.to_vec())
        } else if let Some((&[], _)) = self.focus {
            Action::Focused(min.y)
        } else {
            node_menu(ui, interact, kind)
                .or(show.body_returned)
                .unwrap_or_default()
        }
    }

    fn additions_deletions_compute(&mut self, size: u32) {
        self.additions = if let (Some(add), Some(gp)) = (self.additions, self.global_pos) {
            let lld = gp - size;
            // ldd <=    <= pos
            let start: usize;
            let end: usize;
            let mut i = 0;
            loop {
                if i >= add.len() {
                    start = i;
                    break;
                }
                if lld <= add[i] {
                    start = i;
                    break;
                }
                i += 1;
            }
            loop {
                if i >= add.len() {
                    end = i;
                    break;
                }
                if add[i] == gp {
                    end = i + 1;
                    break;
                }
                if add[i] > gp {
                    end = i;
                    break;
                }
                i += 1;
            }
            Some(&add[start..end])
        } else {
            None
        };
        self.deletions = if let (Some(del), Some(gp)) = (self.deletions, self.global_pos) {
            let lld = gp - size;
            // ldd <=    <= pos
            let start: usize;
            let end: usize;
            let mut i = 0;
            loop {
                if i >= del.len() {
                    start = i;
                    break;
                }
                if lld <= del[i] {
                    start = i;
                    break;
                }
                i += 1;
            }
            loop {
                if i >= del.len() {
                    end = i;
                    break;
                }
                if del[i] == gp {
                    end = i + 1;
                    break;
                }
                if del[i] > gp {
                    end = i;
                    break;
                }
                i += 1;
            }
            Some(&del[start..end])
        } else {
            None
        };
    }

    fn ui_non_loaded(
        &mut self,
        ui: &mut egui::Ui,
        nid: NodeIdentifier,
        offset: usize,
        child: NodeIdentifier,
    ) -> Action {
        // wasm_rs_dbg::dbg!();
        let min = ui.available_rect_before_wrap().min;
        if min.y < 0.0 {
            self.min_before_count += 1;
        }
        self.draw_count += 1;
        let id = ui.id().with(&self.path);
        let mut load_with_default_open =
            egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, false);
        if self.focus.is_some() {
            // wasm_rs_dbg::dbg!();
            load_with_default_open.set_open(true)
        }
        // wasm_rs_dbg::dbg!();
        let show: FoldRet<_, _> = load_with_default_open
            .show_header(ui, |ui| {
                // ui.label(format!("{}: {}", kind, label));
                // wasm_rs_dbg::dbg!();
                ui.monospace(format!("waiting: {}", nid))
                // .context_menu(|ui| {
                //     ui.label(format!("{:?}", self.path));
                // })
            })
            .body(|ui| {
                // wasm_rs_dbg::dbg!();
                let mut act = Action::Keep;
                let mut prefill_old = if let Some(prefill_cache) = self.prefill_cache.take() {
                    prefill_cache
                } else {
                    PrefillCache {
                        head: 0.0,
                        children: vec![],
                        children_sizes: vec![],
                        next: None,
                    }
                };
                let mut prefill = PrefillCache {
                    head: prefill_old.head,
                    children: vec![],
                    children_sizes: vec![],
                    next: None,
                };
                let mut path = self.path.clone();
                path.push(offset);
                // wasm_rs_dbg::dbg!(offset, &self.focus, &path);
                // let mut path_bis = self.path.clone();
                // for o in self.focus.unwrap().0 {
                //     wasm_rs_dbg::dbg!(offset, &self.path, &self.focus, o, &path_bis);
                //     path_bis.push(*o);
                //     let id = ui.id().with(&path_bis);
                //     let mut load_with_default_open =
                //         egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true);
                //     load_with_default_open.set_open(true);
                //     load_with_default_open.store(ui.ctx());
                // }
                self.children_ui_aux(
                    ui,
                    offset,
                    &child,
                    &mut act,
                    &mut prefill_old,
                    &mut prefill,
                    None,
                    None,
                    &mut self.global_pos.clone(),
                    path,
                );
                act
            })
            .into();

        let mut prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
            prefill_cache
        } else {
            PrefillCache {
                head: 0.0,
                children: vec![],
                children_sizes: vec![],
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

        for handle in &mut self.hightlights {
            selection_highlight(ui, handle, min, rect, self.root_ui_id);
        }

        // ui.label(format!("{:?}", show.body_response.map(|x| x.rect)));
        self.prefill_cache = Some(prefill);
        if show
            .header_returned
            .interact(egui::Sense::click())
            .clicked()
        {
            Action::Clicked(self.path.to_vec())
        } else if let Some((&[], _)) = self.focus {
            Action::Focused(min.y)
        } else {
            show.body_returned.unwrap_or(Action::Keep)
        }
    }

    fn ui_labeled_impl2(
        &mut self,
        ui: &mut egui::Ui,
        kind: AnyType,
        _size: u32,
        nid: NodeIdentifier,
        label: LabelIdentifier,
    ) -> Action {
        if self.is_hidden(kind) {
            let prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
                prefill_cache
            } else {
                PrefillCache {
                    head: 0.0,
                    children: vec![],
                    children_sizes: vec![],
                    next: None,
                }
            };
            self.prefill_cache = Some(prefill);
            return Action::Keep;
        }
        let min = ui.available_rect_before_wrap().min;
        self.draw_count += 1;
        let id = ui.id().with(&self.path);
        if self.is_pp(kind) {
            let action = self.show_pp(ui, nid);
            return action;
        }
        let label = if let Some(get) = self.store.label_store.read().unwrap().try_resolve(&label) {
            get.replace("\n", "\\n")
                .replace("\t", "\\t")
                .replace(" ", "·")
        } else {
            if !self
                .store
                .labels_pending
                .lock()
                .unwrap()
                .iter()
                .any(|x| x.contains(&label))
            {
                self.store
                    .labels_waiting
                    .lock()
                    .unwrap()
                    .get_or_insert(Default::default())
                    .insert(label);
            }
            "...".to_string()
        };
        let action;
        let rect = if label.len() > 50 {
            if kind.is_spaces() {
                let monospace = ui.monospace(format!("{}: ", kind))
                // .context_menu(|ui| {
                //     ui.label(format!("{:?}", self.path));
                // })
                ;
                let interact = monospace.interact(egui::Sense::click_and_drag());
                action = if interact.drag_released() {
                    node_menu(ui, interact, kind).unwrap_or_default()
                } else if interact.clicked() {
                    Action::Clicked(self.path.to_vec())
                } else {
                    node_menu(ui, interact, kind).unwrap_or_default()
                };
                let rect1 = monospace.rect;
                let rect2 = ui.label(format!("{}", label)).rect;
                rect1.union(rect2)
            } else {
                let mut label = egui::RichText::new(label);
                let monospace = {
                    let text = format!("{}: ", kind);
                    let mut rt = egui::RichText::new(text).monospace();
                    if let Some(gp) = &self.global_pos {
                        if self.additions.is_some() || self.deletions.is_some() {
                            let add = self.additions.unwrap_or_default();
                            let del = self.deletions.unwrap_or_default();
                            // wasm_rs_dbg::dbg!(add, del);
                            if add.binary_search(gp).is_ok() {
                                if del.last() == Some(gp) {
                                    rt = rt.color(egui::Color32::BLUE);
                                } else {
                                    rt = rt.color(egui::Color32::GREEN);
                                }
                            } else if del.binary_search(gp).is_ok() {
                                rt = rt.underline();
                                rt = rt.color(egui::Color32::RED);
                            } else {
                                label = label.size(8.0);
                                label = label.color(egui::Color32::GRAY);
                                rt = rt.size(8.0);
                                rt = rt.color(egui::Color32::GRAY);
                            }
                        }
                    }
                    ui.label(rt).interact(egui::Sense::click_and_drag())
                }
                // .context_menu(|ui| {
                //     ui.label(format!("{:?}", self.path));
                // })
                ;
                let rect1 = monospace.rect;
                let indent = ui.indent(id, |ui| {
                    ui.label(label).interact(egui::Sense::click())
                    // .context_menu(|ui| {
                    //     ui.label(format!("{:?}", self.path));
                    // })
                });
                // let interaction = (
                //     monospace.interact(egui::Sense::click()),
                //     indent.inner.interact(egui::Sense::click()),
                // );
                action = if monospace.drag_released() {
                    node_menu(ui, monospace, kind).unwrap_or_default()
                } else if monospace.clicked() {
                    Action::Clicked(self.path.to_vec())
                } else {
                    node_menu(ui, monospace, kind).unwrap_or_default()
                };

                let rect2 = indent.response.rect;
                rect1.union(rect2)
            }
        } else {
            let add_contents = |ui: &mut egui::Ui| {
                let action = {
                    let text = format!("{}: ", kind);
                    let mut label = egui::RichText::new(label);
                    let mut rt = egui::RichText::new(text).monospace();
                    if let Some(gp) = &self.global_pos {
                        if self.additions.is_some() || self.deletions.is_some() {
                            let add = self.additions.unwrap_or_default();
                            let del = self.deletions.unwrap_or_default();
                            // wasm_rs_dbg::dbg!(add, del);
                            if add.binary_search(gp).is_ok() {
                                if del.last() == Some(gp) {
                                    rt = rt.color(egui::Color32::BLUE);
                                } else {
                                    rt = rt.color(egui::Color32::GREEN);
                                }
                            } else if del.binary_search(gp).is_ok() {
                                rt = rt.underline();
                                rt = rt.color(egui::Color32::RED);
                            } else {
                                label = label.size(8.0);
                                label = label.color(egui::Color32::GRAY);
                                rt = rt.size(8.0);
                                rt = rt.color(egui::Color32::GRAY);
                            }
                        }
                    }
                    let interact =
                        ui.add(egui::Label::new(rt).sense(egui::Sense::click_and_drag()));
                    ui.label(label);
                    if interact.drag_released() {
                        node_menu(ui, interact, kind).unwrap_or_default()
                    } else if interact.clicked() {
                        Action::Clicked(self.path.to_vec())
                    } else {
                        node_menu(ui, interact, kind).unwrap_or_default()
                    }
                };
                action
            };
            if kind.is_spaces() {
                action = Action::Keep;
                if self.aspects.spacing {
                    ui.horizontal(add_contents).response.rect
                } else {
                    egui::Rect::from_min_max(min, min)
                }
            } else if kind.is_syntax() {
                action = Action::Keep;
                if self.aspects.syntax || self.aspects.syntax {
                    ui.horizontal(add_contents).response.rect
                } else {
                    egui::Rect::from_min_max(min, min)
                }
            } else {
                let h = ui.horizontal(add_contents);
                action = h.inner;
                h.response.rect
            }
        };
        let mut prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
            prefill_cache
        } else {
            PrefillCache {
                head: 0.0,
                children: vec![],
                children_sizes: vec![],
                next: None,
            }
        };
        prefill.head = ui.available_rect_before_wrap().min.y - min.y;

        for handle in &mut self.hightlights {
            if handle.path.is_empty() {
                selection_highlight(ui, handle, min, rect, self.root_ui_id);
                // ui.painter().debug_rect(rect, **c, "");
            }
        }
        self.prefill_cache = Some(prefill);
        action
    }

    fn is_pp(&mut self, kind: AnyType) -> bool {
        if let Some(x) = kind.as_any().downcast_ref() {
            if self.aspects.ser_opt_cpp.contains(x) {
                return true;
            }
        };
        if let Some(x) = kind.as_any().downcast_ref() {
            if self.aspects.ser_opt_java.contains(x) {
                return true;
            }
        };
        false
    }

    fn is_hidden(&mut self, kind: AnyType) -> bool {
        if let Some(x) = kind.as_any().downcast_ref() {
            if self.aspects.hide_opt_cpp.contains(x) {
                return true;
            }
        };
        if let Some(x) = kind.as_any().downcast_ref() {
            if self.aspects.hide_opt_java.contains(x) {
                return true;
            }
        };
        false
    }

    fn show_pp(&mut self, ui: &mut egui::Ui, nid: NodeIdentifier) -> Action {
        let mut prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
            prefill_cache
        } else {
            PrefillCache {
                head: 0.0,
                children: vec![],
                children_sizes: vec![],
                next: None,
            }
        };
        let min = ui.available_rect_before_wrap().min;
        let theme = syntax_highlighter::simple::CodeTheme::from_memory(ui.ctx());
        // TODO fetch entire subtree, line breaks would also be useful
        let layout_job = make_pp_code(self.store.clone(), ui.ctx(), nid, theme);
        let galley = ui.fonts(|f| f.layout_job(layout_job));

        let size = galley.size();
        let resp = ui.allocate_exact_size(size, egui::Sense::click());

        let rect = egui::Rect::from_min_size(min, size);
        if self.additions.is_some() || self.deletions.is_some() {
            let add = self.additions.unwrap_or_default();
            let del = self.deletions.unwrap_or_default();
            if add.is_empty() && del.is_empty() {
                // ui.painter().debug_rect(rect, egui::Color32::GRAY, "");
            } else if !add.is_empty() {
                if !del.is_empty() {
                    ui.painter().debug_rect(rect, egui::Color32::BLUE, "");
                } else {
                    // wasm_rs_dbg::dbg!(self.global_pos, size, add);
                    ui.painter().debug_rect(rect, egui::Color32::GREEN, "");
                }
            } else if !del.is_empty() {
                ui.painter().debug_rect(rect, egui::Color32::RED, "");
            }
        }
        let rect = rect;
        //.expand(3.0);
        ui.painter_at(rect.expand(1.0))
            .galley(min, galley, egui::Color32::RED);
        // rect.max.x += 10.0;

        prefill.head = ui.available_rect_before_wrap().min.y - min.y;

        for handle in &mut self.hightlights {
            // egui::Rect::from_min_size(min, (ui.available_width(), height).into()),
            selection_highlight(ui, handle, min, rect, self.root_ui_id);
        }
        self.prefill_cache = Some(prefill);

        let action = if resp.1.clicked() {
            Action::Clicked(self.path.to_vec())
        } else if let Some((&[], _)) = self.focus {
            Action::Focused(min.y)
        } else {
            Action::Keep
        };
        action
    }

    fn ui_typed_impl2(&mut self, ui: &mut egui::Ui, kind: AnyType, _size: u32) -> Action {
        if self.is_hidden(kind) {
            let prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
                prefill_cache
            } else {
                PrefillCache {
                    head: 0.0,
                    children: vec![],
                    children_sizes: vec![],
                    next: None,
                }
            };
            self.prefill_cache = Some(prefill);
            return Action::Keep;
        }
        let min = ui.available_rect_before_wrap().min;
        self.draw_count += 1;
        let mut resp = None;
        if kind.is_spaces() {
            if self.aspects.spacing {
                resp = Some(ui.monospace(format!("{}", kind)));
            }
        } else if kind.is_syntax() {
            if self.aspects.syntax || self.aspects.syntax {
                resp = Some(ui.monospace(format!("{}", kind)));
            }
        } else {
            resp = Some(ui.monospace(format!("{}", kind)));
        };
        let action = resp
            .and_then(|interact| {
                node_menu(ui, interact.interact(egui::Sense::click_and_drag()), kind)
            })
            .unwrap_or_default();
        // let h = ui.monospace(format!("{}", kind));

        let mut prefill = if let Some(prefill_cache) = self.prefill_cache.take() {
            prefill_cache
        } else {
            PrefillCache {
                head: 0.0,
                children: vec![],
                children_sizes: vec![],
                next: None,
            }
        };
        prefill.head = ui.available_rect_before_wrap().min.y - min.y;
        // TODO selection_highlight
        self.prefill_cache = Some(prefill);
        if let Some((&[], _)) = self.focus {
            Action::Focused(min.y)
        } else {
            action
        }
    }

    pub(crate) fn children_ui(
        &mut self,
        ui: &mut egui::Ui,
        // depth: usize,
        cs: &[NodeIdentifier],
        mut global_pos: Option<u32>,
    ) -> Action {
        let mut action = Action::Keep;
        // if depth > 5 {
        //     for c in cs {
        //         ui.label(c.to_string());
        //     }
        //     return Action::Keep;
        // }
        let additions = self.additions.as_ref().map(|x| &x[..]);
        let deletions = self.deletions.as_ref().map(|x| &x[..]);
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
                children_sizes: vec![],
                next: None,
            }
        };

        let mut prefill = PrefillCache {
            head: prefill_old.head,
            children: vec![],
            children_sizes: vec![],
            next: None,
        };
        for (i, c) in cs.iter().enumerate() {
            let mut path = self.path.clone();
            path.push(i);
            match self.children_ui_aux(
                ui,
                i,
                c,
                &mut action,
                &mut prefill_old,
                &mut prefill,
                additions,
                deletions,
                &mut global_pos,
                path,
            ) {
                ControlFlow::Continue(_) => continue,
                ControlFlow::Break(_) => break,
            }
        }
        self.prefill_cache = Some(prefill);
        action
    }

    fn children_ui_aux(
        &mut self,
        ui: &mut egui::Ui,
        i: usize,
        c: &NodeIdentifier,
        action: &mut Action,
        prefill_old: &mut PrefillCache,
        prefill: &mut PrefillCache,
        additions: Option<&[u32]>,
        deletions: Option<&[u32]>,
        mut global_pos: &mut Option<u32>,
        path: Vec<usize>,
    ) -> ControlFlow<()> {
        let rect = ui.available_rect_before_wrap();
        let focus = self.focus.as_ref().and_then(|x| {
            if x.0.is_empty() {
                None
            } else if x.0[0] == i {
                Some((&x.0[1..], &x.1[1..]))
            } else {
                None
            }
        });
        if self.focus.is_none()
            && rect.min.y > 0.0
            && ui.ctx().screen_rect().height() - CLIP_LEN < rect.min.y
        {
            // wasm_rs_dbg::dbg!(self.focus);
            return ControlFlow::Break(());
        }
        let hightlights: Vec<_> = self
            .hightlights
            .extract_if(|handle| {
                !handle.path.is_empty() && handle.path[0] == i
                // if x.is_empty() {
                //     None
                // } else if x[0] == i {
                //     Some((&x[1..], *c, *p))
                // } else {
                //     None
                // }
            })
            .map(|handle| HightLightHandle {
                path: &handle.path[1..],
                color: handle.color,
                screen_pos: handle.screen_pos,
                id: handle.id,
            })
            .collect();
        // let mut ignore = None;
        let mut imp = if let Some(child) = prefill_old.children.get(i) {
            let child_size = prefill_old.children_sizes.get(i).unwrap(); // children and children_sizes should be the same sizes
            let exact_max_y = rect.min.y + *child;
            if focus.is_none() && exact_max_y < CLIP_LEN {
                // ignore = Some(exact_max_y);
                // FetchedViewImpl {
                //     store: self.store,
                //     prefill_cache: None,
                //     min_before_count: 0,
                //     draw_count: 0,
                // }
                prefill.children.push(*child);
                prefill.children_sizes.push(*child_size);
                if let (Some(child_size), Some(gp)) = (child_size, &mut global_pos) {
                    *gp += child_size.get();
                } else {
                    *global_pos = None;
                }
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
                // wasm_rs_dbg::dbg!(self.focus);
                return ControlFlow::Continue(());
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
                    additions,
                    deletions,
                    global_pos: None,
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
                additions,
                deletions,
                global_pos: None,
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
                additions,
                deletions,
                global_pos: None,
            }
        };
        let _size;
        let ret = if let Some(r) = self
            .store
            .node_store
            .read()
            .unwrap()
            .try_resolve::<NodeIdentifier>(*c)
        {
            let kind = self.store.resolve_type(c); //r.get_type();
            let l = r.try_get_label().copied();
            let cs = r.children();
            let size = r.size();

            if let Some(gp) = global_pos {
                *gp += size as u32;
            }
            _size = Some(size as u32);
            imp.global_pos = *global_pos;

            if let Some(cs) = cs {
                if let Some(label) = l {
                    imp.ui_both_impl(ui, kind, size as u32, label, cs.0.to_vec().as_ref())
                } else {
                    imp.ui_children_impl2(ui, kind, size as u32, *c, cs.0.to_vec().as_ref())
                }
            } else if let Some(label) = l {
                imp.ui_labeled_impl2(ui, kind, size as u32, *c, label)
            } else {
                imp.ui_typed_impl2(ui, kind, size as u32)
            }
        // let ret = if let Some(c) = self.store.both.ids.iter().position(|x| x == c) {
        //     imp.ui_both_impl(ui, depth + 1, c)
        // } else if let Some(c) = self.store.labeled.ids.iter().position(|x| x == c) {
        //     imp.ui_labeled_impl(ui, depth + 1, c)
        // } else if let Some(c) = self.store.children.ids.iter().position(|x| x == c) {
        //     imp.ui_children_impl(ui, depth + 1, c)
        // } else if let Some(c) = self.store.typed.ids.iter().position(|x| x == c) {
        //     imp.ui_typed_impl(ui, depth + 1, c)
        } else {
            let min = ui.available_rect_before_wrap().min;
            let head = ui.available_rect_before_wrap().min.y - min.y;
            imp.prefill_cache = Some(PrefillCache {
                head: head,
                children: vec![],
                children_sizes: vec![],
                next: None,
            });
            _size = None;
            if !self
                .store
                .nodes_pending
                .lock()
                .unwrap()
                .iter()
                .any(|x| x.contains(c))
            {
                self.store
                    .nodes_waiting
                    .lock()
                    .unwrap()
                    .get_or_insert(Default::default())
                    .insert(*c);
            }
            if let Some(focus) = &imp.focus {
                wasm_rs_dbg::dbg!(&focus);
                if let Some(x) = self.focus.as_ref().unwrap().1.first() {
                    imp.additions = None;
                    imp.deletions = None;
                    let a = imp.ui_non_loaded(ui, *c, *focus.0.first().unwrap_or(&0), *x);
                    match a {
                        Action::PartialFocused(x) => Action::PartialFocused(x),
                        Action::Focused(x) => Action::PartialFocused(x),
                        Action::Keep => {
                            Action::PartialFocused(ui.available_rect_before_wrap().min.y)
                        } // TODO find why it is not focused
                        x => panic!("{:?}", x),
                        // x => x,
                    }
                } else {
                    let kind: &'static dyn HyperType = &hyperast_gen_ts_cpp::types::Type::ERROR;
                    imp.additions = None;
                    imp.deletions = None;
                    let a = imp.ui_typed_impl2(ui, AnyType::from(kind), 0);
                    match a {
                        Action::PartialFocused(x) => Action::PartialFocused(x),
                        Action::Focused(x) => Action::PartialFocused(x),
                        x => panic!("{:?}", x),
                        // x => x,
                    }
                }
            } else {
                let min = ui.available_rect_before_wrap().min;
                imp.draw_count += 1;
                ui.horizontal(|ui| {
                    ui.spinner();
                    ui.label(c.to_string());
                });
                let mut prefill = if let Some(prefill_cache) = imp.prefill_cache.take() {
                    prefill_cache
                } else {
                    PrefillCache {
                        head: 0.0,
                        children: vec![],
                        children_sizes: vec![],
                        next: None,
                    }
                };
                prefill.head = ui.available_rect_before_wrap().min.y - min.y;
                imp.prefill_cache = Some(prefill);
                Action::PartialFocused(ui.available_rect_before_wrap().min.y)
            }
        };
        match ret {
            Action::Clicked(_)
            | Action::Focused(_)
            | Action::PartialFocused(_)
            | Action::SerializeKind(_)
            | Action::HideKind(_) => {
                *action = ret;
            }
            _ => (),
        };
        let c_cache = imp.prefill_cache.unwrap();
        let h = c_cache.height();
        // if let Some(e_m_y) = ignore {
        //     prefill.children.push(h);
        //     prefill
        //         .children_sizes
        //         .push(_size.map(|x| x.try_into().unwrap()));
        //     if prefill_old.children.len() == i {
        //         prefill.next = Some(Box::new(c_cache));
        //     }
        //     if DEBUG_LAYOUT {
        //         ui.painter().debug_rect(
        //         egui::Rect::from_min_size(rect.min, (500.0-i as f32 * 3.0, e_m_y-rect.min.y).into()),
        //         egui::Color32::RED,
        //         format!(
        //             "\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t\t{}\t{}\t{}\t{}",
        //             h,rect.min.y, rect.min.y + h, e_m_y
        //         ),
        //     );
        //     }
        //     return ControlFlow::Continue(());
        // }

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
                prefill
                    .children_sizes
                    .push(_size.map(|x| x.try_into().unwrap()));
            }
        } else if prefill.next.is_none() {
            if rect.min.y > CLIP_LEN {
                color = egui::Color32::LIGHT_GREEN;
            }
            prefill.children.push(h);
            prefill
                .children_sizes
                .push(_size.map(|x| x.try_into().unwrap()));
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
        return ControlFlow::Continue(());
    }
}

fn node_menu(ui: &mut egui::Ui, interact: egui::Response, kind: AnyType) -> Option<Action> {
    let mut act = None;
    let popup_id = interact.id.with("popup");
    if interact.secondary_clicked() || interact.double_clicked() || interact.drag_released() {
        ui.memory_mut(|mem| mem.open_popup(popup_id));
    }
    egui::popup_below_widget(
        ui,
        popup_id,
        &interact,
        egui::PopupCloseBehavior::CloseOnClick,
        |ui| {
            if ui.button("hide kind").clicked() {
                act = Some(Action::HideKind(kind));
                ui.memory_mut(|mem| mem.close_popup());
            } else if ui.button("serialize kind").clicked() {
                act = Some(Action::SerializeKind(kind));
                ui.memory_mut(|mem| mem.close_popup());
            } else if ui.button("close menu").clicked() {
                ui.memory_mut(|mem| mem.close_popup());
                // ui.close_menu();
            }
        },
    );
    act
}

#[allow(unused)] // TODO reenable ports after fixing the dependency (I believe it was not maintained anymore, and very convoluted structure btw)
fn show_port(ui: &mut egui::Ui, id: egui::Id, pos: epaint::Pos2) {
    let area = egui::Area::new(id)
        .order(egui::Order::Middle)
        .constrain(true)
        .fixed_pos(pos)
        .interactable(false);
    // area.show(ui.ctx(), |ui| ui.add(Port::new(id)));
}

const DEBUG_LAYOUT: bool = false;
/// increase to debug and see culling in action
const CLIP_LEN: f32 = 0.0; //250.0;

fn subtree_to_layout(
    store: &store::FetchedHyperAST,
    theme: &syntax_highlighter::simple::CodeTheme,
    nid: NodeIdentifier,
) -> (usize, Vec<LayoutSection>) {
    // let read = store.read();
    match hyperast_layouter::Layouter::<_, _>::new(&store.read(), nid, theme).compute() {
        Err(IndentedAlt::FmtError) => panic!(),
        Err(IndentedAlt::NoIndent) => panic!(),
        Ok(x) => x,
    }
}

mod hyperast_layouter {
    use super::syntax_highlighter;
    use epaint::text::LayoutSection;
    use hyperast::nodes::Space;
    use hyperast::{nodes::IndentedAlt, types::NodeId};

    pub struct Layouter<'a, 'b, IdN, HAST, const SPC: bool = false> {
        stores: &'a HAST,
        root: IdN,
        root_indent: &'static str,
        theme: &'b syntax_highlighter::simple::CodeTheme,
    }
    impl<'store, 'b, IdN, HAST, const SPC: bool> Layouter<'store, 'b, IdN, HAST, SPC> {
        pub fn new(
            stores: &'store HAST,
            root: IdN,
            theme: &'b syntax_highlighter::simple::CodeTheme,
        ) -> Self {
            Self {
                stores,
                root,
                root_indent: "\n",
                theme,
            }
        }
    }

    fn make_section(
        theme: &syntax_highlighter::simple::CodeTheme,
        out: &mut Vec<LayoutSection>,
        format: syntax_highlighter::TokenType,
        offset: usize,
        end: usize,
    ) {
        let mut format = theme.formats[format].clone();
        format.font_id = egui::FontId::monospace(12.0);
        out.push(LayoutSection {
            leading_space: 0.0,
            byte_range: offset..end.clone(),
            format,
        });
    }

    use hyperast::types::{
        self, HyperAST, HyperASTShared, HyperType, Childrn,
    };

    impl<'store, 'b, IdN, HAST, const SPC: bool> Layouter<'store, 'b, IdN, HAST, SPC>
    where
        HAST: HyperAST<IdN = IdN>,
        IdN: NodeId<IdN = IdN>,
        // HAST: types::NodeStore<IdN>,
        HAST: types::LabelStore<str>,
        HAST: types::TypeStore,
    {
        pub fn compute(&self) -> Result<(usize, Vec<LayoutSection>), IndentedAlt> {
            let mut layout = vec![];
            let mut offset = 0;
            match self._compute(&self.root, self.root_indent, &mut layout, &mut offset) {
                Err(IndentedAlt::FmtError) => Err(IndentedAlt::FmtError),
                _ => Ok((offset, layout)),
            }
        }
        fn _compute(
            &self,
            id: &HAST::IdN,
            parent_indent: &str,
            out: &mut Vec<LayoutSection>,
            offset: &mut usize,
        ) -> Result<String, IndentedAlt> {
            use types::LabelStore;
            use types::Labeled;
            use types::NodeStore;
            use types::WithChildren;
            let b = self.stores.node_store().resolve(id);
            // let kind = (self.stores.type_store(), b);
            let kind = self.stores.resolve_type(id);
            let label = b.try_get_label();
            let children = b.children();

            if kind.is_spaces() {
                let indent = if let Some(label) = label {
                    let s = self.stores.label_store().resolve(label);
                    let b: String = Space::format_indentation(s.as_bytes())
                        .iter()
                        .map(|x| x.to_string())
                        .collect();
                    // out.write_str(&b).unwrap();
                    let len = s.len();
                    let end = *offset + len;
                    let format = syntax_highlighter::TokenType::Punctuation;
                    make_section(self.theme, out, format, *offset, end);
                    *offset = end;
                    if b.contains("\n") {
                        b
                    } else {
                        parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned()
                    }
                } else {
                    parent_indent[parent_indent.rfind('\n').unwrap_or(0)..].to_owned()
                };
                return Ok(indent);
            }

            match (label, children) {
                (None, None) => {
                    // out.write_str(&kind.to_string()).unwrap();
                    let len = kind.to_string().len();
                    let end = *offset + len;
                    let format = syntax_highlighter::TokenType::Keyword;
                    make_section(self.theme, out, format, *offset, end);
                    *offset = end;
                    return Err(IndentedAlt::NoIndent)
                }
                (label, Some(children)) => {
                    if let Some(label) = label {
                        let s = self.stores.label_store().resolve(label);
                        dbg!(s);
                    }
                    if !children.is_empty() {
                        let mut it = children.iter_children();
                        let op = |alt| {
                            if alt == IndentedAlt::NoIndent {
                                Ok(parent_indent[parent_indent.rfind('\n').unwrap_or(0)..]
                                    .to_owned())
                            } else {
                                Err(alt)
                            }
                        };
                        let mut ind = self
                            ._compute(&it.next().unwrap(), parent_indent, out, offset)
                            .or_else(op)?;
                        for id in it {
                            ind = self._compute(&id, &ind, out, offset).or_else(op)?;
                        }
                    }
                    return Err(IndentedAlt::NoIndent)
                }
                (Some(label), None) => {
                    let s = self.stores.label_store().resolve(label);
                    // out.write_str(&s).unwrap();
                    let len = s.len();
                    let end = *offset + len;
                    let format = syntax_highlighter::TokenType::Punctuation;
                    make_section(self.theme, out, format, *offset, end);
                    *offset = end;
                    return Err(IndentedAlt::NoIndent)
                }
            };
        }
    }
}

fn subtree_to_string(store: &store::FetchedHyperAST, nid: NodeIdentifier) -> String {
    let read = store.read();
    let s = {
        // SAFETY: the transmuted value does not escape the function scope
        // NOTE issue with the usual widening to 'static ...
        let read: &store::AcessibleFetchedHyperAST<'_> = unsafe { std::mem::transmute(&read) };
        ToString::to_string(&hyperast::nodes::TextSerializer::<_, _>::new(read, nid))
    };
    drop(read);
    s
}

pub(crate) fn make_pp_code(
    store: Arc<store::FetchedHyperAST>,
    ctx: &egui::Context,
    nid: NodeIdentifier,
    theme: syntax_highlighter::simple::CodeTheme,
) -> epaint::text::LayoutJob {
    #[derive(Default)]
    struct PrettyPrinter {}
    impl cache::ComputerMut<(&store::FetchedHyperAST, NodeIdentifier), String> for PrettyPrinter {
        fn compute(&mut self, (store, id): (&store::FetchedHyperAST, NodeIdentifier)) -> String {
            subtree_to_string(store, id)
        }
    }

    type PPCache = cache::FrameCache<String, PrettyPrinter>;

    let code = ctx.memory_mut(|mem| mem.caches.cache::<PPCache>().get((store.as_ref(), nid)));
    #[derive(Default)]
    struct Spawner {}
    impl
        syntax_highlighting_async::cache::Spawner<
            (
                Arc<store::FetchedHyperAST>,
                &syntax_highlighter::simple::CodeTheme,
                NodeIdentifier,
                usize,
            ),
            Layouter,
        > for Spawner
    {
        fn spawn(
            &self,
            ctx: &egui::Context,
            (_store, _theme, _id, len): (
                Arc<store::FetchedHyperAST>,
                &syntax_highlighter::simple::CodeTheme,
                NodeIdentifier,
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
        mt: Vec<Arc<Mutex<egui_addon::async_exec::TimeoutHandle>>>,
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
                Arc<store::FetchedHyperAST>,
                &syntax_highlighter::simple::CodeTheme,
                NodeIdentifier,
                usize,
            ),
            Vec<LayoutSection>,
        > for Layouter
    {
        fn increment(
            &mut self,
            _spawner: &Spawner,
            (store, theme, id, len): (
                Arc<store::FetchedHyperAST>,
                &syntax_highlighter::simple::CodeTheme,
                NodeIdentifier,
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
                self.mt.push(Arc::new(Mutex::new(
                    egui_addon::async_exec::spawn_macrotask(Box::new(fut)),
                )));
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
        mem.caches
            .cache::<HCache>()
            .get(ctx, (store.clone(), &theme, nid, code.len()))
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
    handle: &mut HightLightHandle<'_>,
    min: epaint::Pos2,
    rect: epaint::Rect,
    root_ui_id: egui::Id,
) {
    let HightLightHandle {
        path,
        color,
        id,
        screen_pos: ret_pos,
    } = handle;
    if path.is_empty() {
        let clip = ui.clip_rect();
        let min_elem = clip.size().min_elem();
        if clip.intersects(rect) {
            ui.painter().debug_rect(rect, **color, "");
        }
        let clip = if min_elem < 1.0 {
            clip
        } else {
            let mut clip = clip.shrink((min_elem / 2.0).min(4.0));
            clip.set_width((clip.width() - 14.0).max(0.0));
            clip
        };

        if clip.intersects(rect) {
            if *color == &egui::Color32::BLUE {
                let _id = root_ui_id.with("blue_highlight").with(id);
                // wasm_rs_dbg::dbg!("green", id);
                let pos = egui::pos2(min.x - 15.0, min.y - 10.0);
                let pos = clip.clamp(pos);
                if ui.clip_rect().contains(pos) {
                    // show_port(ui, id, pos);
                    **ret_pos = Some(rect);
                }
            } else if *color == &TARGET_COLOR {
                let _id = root_ui_id.with("green_highlight").with(id);
                let pos = egui::pos2(rect.max.x - 10.0, rect.min.y - 10.0);
                let pos = clip.clamp(pos);
                if ui.clip_rect().contains(pos) {
                    // show_port(ui, id, pos);
                    **ret_pos = Some(rect);
                }
            }

            ui.painter().debug_rect(rect, **color, "");
        }
    }
}
