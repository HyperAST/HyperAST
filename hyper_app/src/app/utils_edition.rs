use super::{
    crdt_over_ws::{self, DocSharingState, SharedDocView},
    types::WithDesc,
    utils_results_batched::{self, ComputeError, RemoteResult},
    Sharing,
};
use crate::app::code_editor_automerge;
use automerge::sync::SyncDoc;
use futures_util::SinkExt;
use std::{
    ops::DerefMut,
    sync::{Arc, Mutex, RwLock},
};
pub type SharedCodeEditors<T> = std::sync::Arc<std::sync::Mutex<T>>;

// TODO allow to change user name and generate a random default
#[cfg(target_arch = "wasm32")]
pub(crate) const USER: &str = "web";
#[cfg(not(target_arch = "wasm32"))]
pub(crate) const USER: &str = "native";

pub(crate) fn show_shared_code_edition<T, U>(
    ui: &mut egui::Ui,
    query_editors: &mut SharedCodeEditors<T>,
    single: &mut Sharing<U>,
) where
    T: autosurgeon::Reconcile,
    T: super::types::EditorHolder<Item = code_editor_automerge::CodeEditor>,
    // for<'a> &'a mut T: IntoIterator<Item = &'a mut code_editor_automerge::CodeEditor>,
{
    let resps: Vec<Option<egui::Response>> = {
        let mut ce = query_editors.lock().unwrap();
        // ce.into_iter().map(|c| c.ui(ui)).collect()
        ce.iter_editors_mut().map(|c| c.ui(ui)).collect()
    };

    let Some(ws) = &mut single.ws else {
        return;
    };
    if resps.iter().filter_map(|x| x.as_ref()).any(|x| x.changed()) {
        let timer = if ws.timer != 0.0 {
            let dt = ui.input(|mem| mem.unstable_dt);
            ws.timer + dt
        } else {
            0.01
        };
        let rt = &single.rt;
        timed_updater(ui, timer, ws, query_editors, rt);
    } else if ws.timer != 0.0 {
        let dt = ui.input(|mem| mem.unstable_dt);
        let timer = ws.timer + dt;
        let rt = &single.rt;
        timed_updater(ui, timer, ws, query_editors, rt);
    }
}

fn timed_updater<T: autosurgeon::Reconcile>(
    ui: &mut egui::Ui,
    timer: f32,
    ws: &mut crdt_over_ws::WsDoc,
    code_editors: &mut SharedCodeEditors<T>, // QueryEditor<code_editor_automerge::CodeEditor>
    rt: &crdt_over_ws::Rt,
) {
    const TIMER: u64 = 1;
    if timer < std::time::Duration::from_secs(TIMER).as_secs_f32() {
        ws.timer = timer;
        ui.ctx()
            .request_repaint_after(std::time::Duration::from_secs_f32(TIMER as f32));
    } else {
        ws.timer = 0.0;
        let quote: &mut T = &mut code_editors.lock().unwrap();
        ws.changed(rt, quote);
    }
}

pub(super) async fn update_handler<T: autosurgeon::Hydrate>(
    mut receiver: futures_util::stream::SplitStream<tokio_tungstenite_wasm::WebSocketStream>,
    mut sender: futures::channel::mpsc::Sender<tokio_tungstenite_wasm::Message>,
    doc: std::sync::Arc<std::sync::RwLock<DocSharingState>>,
    ctx: egui::Context,
    rt: crdt_over_ws::Rt,
    code_editors: SharedCodeEditors<T>,
) {
    use futures_util::StreamExt;
    #[derive(serde::Deserialize, serde::Serialize, Debug, Clone)]
    enum DbMsgToServer {
        Create { name: String },
        User { name: String },
    }
    let owner = USER.to_string();
    sender
        .send(tokio_tungstenite_wasm::Message::Text(
            serde_json::to_string(&DbMsgToServer::User { name: owner }).unwrap(),
        ))
        .await
        .unwrap();
    match receiver.next().await {
        Some(Ok(tokio_tungstenite_wasm::Message::Binary(bin))) => {
            let (doc, sync_state): &mut (_, _) = &mut doc.write().unwrap();
            let message = automerge::sync::Message::decode(&bin).unwrap();
            doc.sync()
                .receive_sync_message(sync_state, message)
                .unwrap();
            wasm_rs_dbg::dbg!(&doc);
            if let Ok(t) = autosurgeon::hydrate(&*doc) {
                let mut text = code_editors.lock().unwrap();
                *text = t;
            }
            ctx.request_repaint();
        }
        _ => (),
    }
    while let Some(Ok(msg)) = receiver.next().await {
        wasm_rs_dbg::dbg!();
        match msg {
            tokio_tungstenite_wasm::Message::Text(msg) => {
                wasm_rs_dbg::dbg!(&msg);
            }
            tokio_tungstenite_wasm::Message::Binary(bin) => {
                wasm_rs_dbg::dbg!();
                let (doc, sync_state): &mut (_, _) = &mut doc.write().unwrap();
                let message = automerge::sync::Message::decode(&bin).unwrap();
                // doc.merge(other)
                match doc.sync().receive_sync_message(sync_state, message) {
                    Ok(_) => (),
                    Err(e) => {
                        wasm_rs_dbg::dbg!(e);
                    }
                }
                match autosurgeon::hydrate(doc) {
                    Ok(t) => {
                        let mut text = code_editors.lock().unwrap();
                        *text = t;
                    }
                    Err(e) => {
                        wasm_rs_dbg::dbg!(e);
                    }
                }
                ctx.request_repaint();

                wasm_rs_dbg::dbg!();
                let mut sender = sender.clone();
                if let Some(message) = doc.sync().generate_sync_message(sync_state) {
                    wasm_rs_dbg::dbg!();
                    let message =
                        tokio_tungstenite_wasm::Message::Binary(message.encode().to_vec());
                    rt.spawn(async move {
                        sender.send(message).await.unwrap();
                    });
                } else {
                    wasm_rs_dbg::dbg!();
                    let message = tokio_tungstenite_wasm::Message::Binary(vec![]);
                    rt.spawn(async move {
                        sender.send(message).await.unwrap();
                    });
                };
            }
            tokio_tungstenite_wasm::Message::Close(_) => {
                wasm_rs_dbg::dbg!();
                break;
            }
        }
    }
}

pub(super) async fn db_update_handler(
    mut sender: futures::channel::mpsc::Sender<tokio_tungstenite_wasm::Message>,
    mut receiver: futures_util::stream::SplitStream<tokio_tungstenite_wasm::WebSocketStream>,
    owner: String,
    ctx: egui::Context,
    data: Arc<RwLock<(Option<usize>, Vec<Option<SharedDocView>>)>>,
) {
    use futures_util::StreamExt;
    use serde::{Deserialize, Serialize};
    type User = String;

    #[derive(Deserialize, Serialize, Debug, Clone)]
    enum DbMsgToServer {
        Create { name: String },
        User { name: String },
    }
    {
        wasm_rs_dbg::dbg!();
        let name = owner.clone();
        let msg = DbMsgToServer::User { name };
        let msg = serde_json::to_string(&msg).unwrap();
        let msg = tokio_tungstenite_wasm::Message::Text(msg);
        sender.send(msg).await.unwrap();
        wasm_rs_dbg::dbg!();
    }
    while let Some(Ok(msg)) = receiver.next().await {
        wasm_rs_dbg::dbg!();
        match msg {
            tokio_tungstenite_wasm::Message::Text(msg) => {
                wasm_rs_dbg::dbg!(&msg);

                #[derive(Deserialize, Serialize, Debug, Clone)]
                enum DbMsgFromServer {
                    Add(SharedDocView),
                    AddWriter(usize, User),
                    RmWriter(usize, User),
                    // Rename(usize, String),
                    Reset { all: Vec<SharedDocView> },
                }
                let msg = serde_json::from_str(&msg).unwrap();

                match msg {
                    DbMsgFromServer::Add(x) => {
                        let b = x.owner == owner;
                        let guard = &mut data.write().unwrap();
                        let (waiting, vec) = guard.deref_mut();
                        let id = x.id;
                        vec.resize(id + 1, None);
                        vec[id] = Some(x);
                        if b {
                            *waiting = Some(id);
                        }
                        ctx.request_repaint();
                    }
                    DbMsgFromServer::AddWriter(_, _) => todo!(),
                    DbMsgFromServer::RmWriter(_, _) => todo!(),
                    DbMsgFromServer::Reset { all } => {
                        let guard = &mut data.write().unwrap();
                        let (_, vec) = guard.deref_mut();
                        *vec = vec![];
                        for x in all {
                            let id = x.id;
                            vec.resize(id + 1, None);
                            vec[id] = Some(x);
                        }
                        ctx.request_repaint();
                    }
                }
            }
            tokio_tungstenite_wasm::Message::Binary(_bin) => {
                wasm_rs_dbg::dbg!();
            }
            tokio_tungstenite_wasm::Message::Close(_) => {
                wasm_rs_dbg::dbg!();
                break;
            }
        }
    }
}

pub(super) fn update_shared_editors<T, L, S: 'static + autosurgeon::Hydrate + std::marker::Send>(
    ui: &mut egui::Ui,
    single: &mut Sharing<T>,
    api_endpoint: &str,
    code_editors: &mut super::EditingContext<L, S>,
) {
    if let Some(doc_db) = &mut single.doc_db {
        let ctx = ui.ctx().clone();
        let rt = single.rt.clone();
        let owner = USER.to_string();
        let data = doc_db.data.clone();
        if let Err(err) = doc_db.setup_atempt(
            move |sender, receiver| rt.spawn(db_update_handler(sender, receiver, owner, ctx, data)),
            &single.rt,
        ) {
            log::warn!("{}", err);
            if ui.button("try restarting sharing connection").clicked() {
                let url = format!("ws://{}/shared-db", api_endpoint);
                *doc_db = crdt_over_ws::WsDocsDb::new(
                    &single.rt,
                    USER.to_string(),
                    ui.ctx().clone(),
                    url,
                );
            }
        }
        match &mut code_editors.current {
            super::EditStatus::Sharing(shared_script) => {
                let ctx = ui.ctx().clone();
                let rt = single.rt.clone();
                let db = doc_db;
                let guard = &mut db.data.write().unwrap();
                let (waiting, vec) = guard.deref_mut();
                if let Some(i) = waiting {
                    wasm_rs_dbg::dbg!();
                    let i = *i;
                    if let Some(Some(view)) = vec.get(i) {
                        assert_eq!(view.id, i);
                        let url = format!("ws://{}/shared/{}", api_endpoint, i);
                        single.ws = Some(crdt_over_ws::WsDoc::new(&rt, USER.to_string(), ctx, url));
                    }
                    code_editors.current = super::EditStatus::Shared(i, shared_script.clone());
                }
            }
            super::EditStatus::Shared(_, shared_script) => {
                if let Some(ws) = &mut single.ws {
                    wasm_rs_dbg::dbg!();
                    let ctx = ui.ctx().clone();
                    let doc = ws.data.clone();
                    let rt = single.rt.clone();
                    let code_editors = shared_script.clone();
                    if let Err(e) = ws.setup_atempt(
                        |sender, receiver| {
                            single.rt.spawn(update_handler(
                                receiver,
                                sender,
                                doc,
                                ctx,
                                rt,
                                code_editors,
                            ))
                        },
                        &single.rt,
                    ) {
                        log::error!("{}", e);
                    }
                }
            }
            _ => (),
        }
    } else {
        let url = format!("ws://{}/shared-db", api_endpoint);
        single.doc_db = Some(crdt_over_ws::WsDocsDb::new(
            &single.rt,
            USER.to_string(),
            ui.ctx().clone(),
            url,
        ));
    }
}

pub(super) fn show_shared<T, L, S: std::default::Default>(
    ui: &mut egui::Ui,
    api_endpoint: &str,
    single: &mut Sharing<T>,
    context: &mut super::EditingContext<L, S>,
    names: Vec<(String, usize)>,
) {
    let mut r = None;
    ui.horizontal_wrapped(|ui| {
        for (name, i) in names.iter() {
            let mut text = egui::RichText::new(name);
            if let super::EditStatus::Shared(j, _) = &context.current {
                if j == i {
                    text = text.strong();
                }
            }
            if ui.button(text).clicked() {
                r = Some(i);
            }
        }
    });
    if let Some(i) = r {
        context.current = super::EditStatus::Shared(*i, Default::default());
        let doc_db = single.doc_db.as_ref().unwrap();
        let doc_views = doc_db.data.write().unwrap();
        let id = doc_views.1.get(*i).unwrap().as_ref().unwrap().id;
        if let Some(ws) = &mut single.ws {
            let ctx = ui.ctx().clone();
            let rt = single.rt.clone();
            let url = format!("ws://{}/shared/{}", api_endpoint, id);
            *ws = crdt_over_ws::WsDoc::new(&rt, USER.to_string(), ctx, url)
        }
    }
}

pub(super) fn show_locals<L: Clone, S>(
    ui: &mut egui::Ui,
    context: &mut super::EditingContext<L, S>,
) -> Option<(egui::Response, L, String)> {
    ui.horizontal_wrapped(|ui| {
        let mut resp = None;
        // let mut n = None;
        for (name, s) in &context.local_scripts {
            let mut text = egui::RichText::new(name);
            if let super::EditStatus::Local {
                name: n,
                content: _,
            } = &context.current
            {
                if name == n {
                    text = text.strong();
                }
            }
            let button = ui.button(text).interact(egui::Sense::click());
            if button.clicked() || button.secondary_clicked() || button.hovered() {
                assert!(resp.is_none());
                resp = Some((button, s.clone(), name.to_string()))
            }
        }
        resp
    })
    .inner
}

pub(super) fn show_interactions<'a, L, S>(
    ui: &mut egui::Ui,
    context: &'a mut super::EditingContext<L, S>,
    docs_db: &Option<crdt_over_ws::WsDocsDb>,
    compute_result: &mut Option<RemoteResult<impl ComputeError + Send + Sync>>,
    examples_names: impl Fn(usize) -> String,
) -> InteractionResp<&'a L> {
    let mut save_button = None;
    let mut share_button = None;
    let mut editor: Option<(String, &L)> = None;
    ui.horizontal(|ui| match &mut context.current {
        super::EditStatus::Example { i, content } => {
            save_button = Some(ui.add(egui::Button::new("Save Script")));
            // let name = &EXAMPLES[*i].name;
            let name = examples_names(*i);
            editor = Some((name, &*content));
        }
        super::EditStatus::Local { name, content } => {
            if let Some(doc_db) = docs_db {
                if doc_db.is_connected() {
                    share_button = Some(ui.add(egui::Button::new("Share Script")));
                }
            }
            save_button = Some(ui.add(egui::Button::new("Save Script")));
            ui.text_edit_singleline(name);
            editor = Some((name.to_string(), &*content));
        }
        _ => (),
    });
    let compute_button = ui
        .horizontal(|ui| {
            let compute_button = ui.add(egui::Button::new("Compute"));
            utils_results_batched::show_short_result(&*compute_result, ui);
            compute_button
        })
        .inner;

    InteractionResp {
        compute_button,
        editor,
        save_button,
        share_button,
    }
}

pub(super) struct InteractionResp<E> {
    pub(super) compute_button: egui::Response,
    pub(super) save_button: Option<egui::Response>,
    pub(super) share_button: Option<egui::Response>,
    pub(super) editor: Option<(String, E)>,
}

pub(super) fn show_available_remote_docs<T, L, S: std::default::Default>(
    ui: &mut egui::Ui,
    api_endpoint: &str,
    single: &mut Sharing<T>,
    context: &mut super::EditingContext<L, S>,
) {
    if let Some(doc_db) = &single.doc_db {
        let names: Vec<_> = doc_db
            .data
            .read()
            .unwrap()
            .1
            .iter()
            .filter_map(|d| d.as_ref())
            .map(|x| (format!("{}/{}", x.owner, x.name), x.id))
            .collect();
        if !names.is_empty() {
            egui::CollapsingHeader::new("Shared Scripts")
                .default_open(true)
                .show(ui, |ui| {
                    show_shared(ui, api_endpoint, single, context, names)
                });
        }
    }
}

pub(super) fn show_locals_and_interact<T, U, L, S>(
    ui: &mut egui::Ui,
    context: &mut super::EditingContext<L, S>,
    docs: &mut Sharing<T>,
) where
    U: AsRef<str>,
    L: Clone + WithDesc<U> + Into<S>,
    S: autosurgeon::Reconcile,
{
    let Some((button, content, name)) = show_locals(ui, context) else {
        return;
    };
    if button.clicked() {
        // res = Some(ex);
        context.current = super::EditStatus::Local { name, content };
    } else if button.hovered() {
        egui::show_tooltip(ui.ctx(), ui.layer_id(), button.id.with("tooltip"), |ui| {
            let desc = content.desc().as_ref();
            egui_demo_lib::easy_mark::easy_mark(ui, desc);
        });
    } else {
        button.context_menu(|ui| {
            if ui.button("share").clicked() {
                let content = content.into();
                let content = Arc::new(Mutex::new(content));
                context.current = super::EditStatus::Shared(usize::MAX, content.clone());
                let mut content = content.lock().unwrap();
                docs.doc_db.as_mut().unwrap().create_doc_atempt(
                    &docs.rt,
                    name,
                    content.deref_mut(),
                );
            }
            if ui.button("close menu").clicked() {
                ui.close_menu()
            }
        });
    }
}

use egui::text::LayoutJob;
use egui_addon::syntax_highlighting::syntect::CodeTheme;

pub(crate) trait MakeHighlights: Copy + std::hash::Hash {
    const COLORS: u8;
    fn highlights(&self, col: u8) -> (egui::Rgba, impl Iterator<Item = (usize, usize)>);
}

impl MakeHighlights for () {
    const COLORS: u8 = 0;
    fn highlights(&self, _col: u8) -> (egui::Rgba, impl Iterator<Item = (usize, usize)>) {
        (egui::Rgba::BLACK, [].iter().copied())
    }
}

/// Wrap a highlighter to also do custom background highlights given a set of ranges with colors (see [`MakeHighlights`]),
/// e.g. visualizing deleted, moved and inserted ranges of code
/// WARN apparently doesn't work well with transparency... the background rectangles are slighlty expanded for looks... so it makes ugly border looking things
#[derive(Default)]
pub(crate) struct HiHighlighter(egui_addon::syntax_highlighting::syntect::Highlighter);
impl<MH: MakeHighlights>
    egui::util::cache::ComputerMut<(&CodeTheme, &str, &str, MH), egui::text::LayoutJob>
    for HiHighlighter
{
    fn compute(&mut self, (theme, code, lang, mh): (&CodeTheme, &str, &str, MH)) -> LayoutJob {
        let mut layout_job = self.0.highlight(theme, code, lang);
        let sections = std::mem::take(&mut layout_job.sections);
        let mut starts = vec![];
        let mut ends = vec![];
        let mut colors = vec![];
        for c in 0..MH::COLORS {
            let x = mh.highlights(c);
            colors.push(x.0);
            for (start, end) in x.1 {
                if end - start > 1 {
                    continue;
                }
                starts.push((start, c));
                ends.push((end, c));
            }
        }
        let colors: Vec<_> = colors.into_iter().map(|c| c.into()).collect();
        starts.sort_by(|a, b| a.0.cmp(&b.0));
        ends.sort_by(|a, b| a.0.cmp(&b.0));

        let mut starts = starts.into_iter().peekable();
        let mut ends = ends.into_iter().peekable();

        let mut current_ratios: Vec<u8> = vec![];
        let mut current_color = egui::Color32::TRANSPARENT;

        let cr_less = |current_ratios: &mut Vec<u8>, less: &u8| {
            let iii = current_ratios.iter().rev().position(|x| x == less).unwrap();
            current_ratios.remove(current_ratios.len() - iii - 1);
        };

        fn acc_col(acc: &mut egui::Color32, curr: egui::Color32) {
            for i in 0..4 {
                acc[i] += curr[i];
            }
        }
        let sections = sections
            .into_iter()
            .flat_map(|mut section| {
                let mut sections = vec![];
                loop {
                    if section.byte_range.len() == 0 {
                        return sections;
                    }
                    let mut f = |pos: usize,
                                 section: &mut egui::text::LayoutSection,
                                 current_ratios: &[u8]| {
                        if section.byte_range.start != pos {
                            let mut prev = section.clone();
                            prev.byte_range.end = pos;
                            acc_col(&mut prev.format.background, current_color);
                            prev.format.line_height = Some(20.0);
                            sections.push(prev);
                        }
                        compute_hi_color(current_ratios, &mut current_color, &colors);
                        section.byte_range.start = pos;
                    };
                    if let (Some((start, more)), Some((end, less))) = (starts.peek(), ends.peek()) {
                        if start == end {
                            if more == less {
                                // special case where color would not have changed
                                starts.next().unwrap();
                                ends.next().unwrap();
                                continue;
                            }
                            if section.byte_range.contains(start) {
                                let pos = *start;
                                cr_less(&mut current_ratios, less);
                                current_ratios.push(*more);
                                f(pos, &mut section, &current_ratios);
                                starts.next().unwrap();
                                ends.next().unwrap();
                                continue;
                            }
                        } else if start < end {
                            if section.byte_range.contains(start) {
                                let pos = *start;
                                current_ratios.push(*more);
                                f(pos, &mut section, &current_ratios);
                                starts.next().unwrap();
                                continue;
                            }
                            assert!(!section.byte_range.contains(end))
                        } else {
                            if section.byte_range.contains(end) {
                                let pos = *end;
                                cr_less(&mut current_ratios, less);
                                f(pos, &mut section, &current_ratios);
                                ends.next().unwrap();
                                continue;
                            }
                            assert!(!section.byte_range.contains(start))
                        }
                    } else if let Some((start, more)) = starts.peek() {
                        if section.byte_range.contains(start) {
                            let pos = *start;
                            current_ratios.push(*more);
                            f(pos, &mut section, &current_ratios);
                            starts.next().unwrap();
                            continue;
                        }
                    } else if let Some((end, less)) = ends.peek() {
                        if section.byte_range.contains(end) {
                            let pos = *end;
                            cr_less(&mut current_ratios, less);
                            f(pos, &mut section, &current_ratios);
                            ends.next().unwrap();
                            continue;
                        }
                    }
                    acc_col(&mut section.format.background, current_color);
                    section.format.line_height = Some(20.0);
                    sections.push(section);
                    break;
                }
                sections
            })
            .collect();

        layout_job.sections = sections;

        layout_job
    }
}

fn compute_hi_color(
    current_ratios: &[u8],
    current_color: &mut egui::Color32,
    colors: &[epaint::Hsva],
) {
    use std::ops::Add;
    use std::ops::Div;
    let mut res = epaint::Hsva::new(0.0, 0.0, 0.0, 0.0);
    for r in 0..current_ratios.len() {
        res.v += colors[current_ratios[r] as usize].v.div(7.0).add(1.0).ln();
        res.a += colors[current_ratios[r] as usize].a.div(7.0).add(1.0).ln();
    }
    if let Some(r) = current_ratios.last() {
        res.h = colors[*r as usize].h;
        res.s = colors[*r as usize].s;
    }
    res.v = res.v.min(0.9);
    res.a = res.a.min(0.9);
    *current_color = res.into();
}

#[derive(Default)]
pub(crate) struct HiHighlighter2;

impl<'a, 'b, MH: MakeHighlights, G: AsRef<std::sync::Arc<egui::Galley>>>
    egui::util::cache::ComputerMut<(G, MH), Vec<(egui::Color32, Vec<egui::Rect>)>>
    for HiHighlighter2
{
    fn compute(&mut self, (galley, highlights): (G, MH)) -> Vec<(egui::Color32, Vec<egui::Rect>)> {
        let galley = galley.as_ref();
        let mut colors = vec![];
        let mut results = vec![];
        for c in 0..MH::COLORS {
            let mut shapes = vec![];
            let x = highlights.highlights(c);
            colors.push(x.0);
            let color: egui::Color32 = x.0.into();
            for (start, end) in x.1 {
                let cursor_range = egui_addon::egui_utils::compute_cursor_range(
                    galley,
                    &std::ops::Range { start, end },
                );
                let row_range = cursor_range.map(|c| c.rcursor.row);
                if row_range[0] + 1 < row_range[1] {
                    let mut mid_rect =
                        egui_addon::egui_utils::compute2_bounding_rect_from_row_range(
                            galley,
                            [row_range[0] + 1, row_range[1] - 1],
                        );
                    mid_rect.max.x += 1.0;
                    mid_rect.min.x -= 1.0;
                    let rect = &galley.rows[row_range[0]];
                    let left = if cursor_range[0].rcursor.column == 0 {
                        egui_addon::egui_utils::first_ws_x(rect)
                            .unwrap_or(rect.x_offset(cursor_range[0].rcursor.column))
                    } else {
                        rect.x_offset(cursor_range[0].rcursor.column) - 5.0
                    };
                    let mut rect = rect.rect;
                    rect.min.x = left; //.min(mid_rect.min.x);
                                       // mid_rect.min.x = left.min(mid_rect.min.x);
                    rect.max.x = rect.max.x.max(mid_rect.max.x);
                    shapes.push(rect);
                    let rect = &galley.rows[row_range[1]];
                    let left2 = egui_addon::egui_utils::first_ws_x(rect);
                    let right = rect.x_offset(cursor_range[1].rcursor.column) + 2.0;
                    let mut rect = rect.rect;
                    if let Some(left2) = left2 {
                        let x = left2 - 5.0;
                        if mid_rect.min.x < x {
                            rect.min.x = mid_rect.min.x;
                        } else {
                            rect.min.x = x;
                            mid_rect.min.x = ((x + rect.max.x) / 2.0).min(rect.min.x + 5.0);
                        }
                    } else {
                        rect.min.x = rect.min.x.min(mid_rect.left());
                    }
                    rect.max.x = right.min(mid_rect.right());
                    shapes.push(rect);
                    shapes.push(mid_rect);
                } else if row_range[0] + 1 == row_range[1] {
                    let first = &galley.rows[row_range[0]];
                    let second = &galley.rows[row_range[1]];
                    let left = first.x_offset(cursor_range[0].rcursor.column);
                    let right = second.x_offset(cursor_range[1].rcursor.column);
                    let left2 = second.glyphs.iter().find(|x| !x.chr.is_ascii_whitespace());
                    let mut first = first.rect;
                    let mut second = second.rect;
                    first.min.x = left;
                    second.max.x = right;
                    if let Some(left2) = left2 {
                        second.min.x = left2.pos.x - 5.0;
                    }
                    // if right > left && left2 < first.max.x {
                    // }
                    shapes.push(first);
                    shapes.push(second);
                } else if row_range[0] == row_range[1] {
                    let rect = &galley.rows[row_range[0]];
                    let left = rect.x_offset(cursor_range[0].rcursor.column);
                    let right = rect.x_offset(cursor_range[1].rcursor.column);
                    let mut rect = rect.rect;
                    rect.min.x = left - 1.0;
                    rect.max.x = right + 1.0;
                    shapes.push(rect);
                } else {
                    // let rect = compute2_bounding_rect(galley, cursor_range);
                    // let rect = rect.translate(aa.inner.text_draw_pos.to_vec2());
                    // shapes.push(egui::Shape::rect_filled(
                    //     rect,
                    //     1.0,
                    //     color.linear_multiply(0.01),
                    // ));
                }
                // // galley.rows[row_range[0]].glyphs[0]
                // let rect = compute2_bounding_rect_from_row_range(galley, row_range);
            }
            results.push((color, shapes))
        }
        results
    }
}
