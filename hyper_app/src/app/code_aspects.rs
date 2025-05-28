use super::tree_view::FetchedViewImpl;
use super::tree_view::{Action, NodeIdentifier, PrefillCache, store::FetchedHyperAST};
use super::types;
use super::types::Resource;
use super::utils_egui::MyUiExt as _;
use egui_addon::egui_utils::{radio_collapsing, show_wip};
use hyperast::store::nodes::fetched;
use hyperast::store::nodes::fetched::LabelIdentifier;
use poll_promise::Promise;
use std::collections::HashSet;
use std::fmt::Debug;
use std::sync::Arc;

pub(crate) const WANTED: types::SelectedConfig = types::SelectedConfig::Aspects;

pub(crate) fn show_config(
    ui: &mut egui::Ui,
    aspects: &mut types::ComputeConfigAspectViews,
    aspects_result: &mut Option<Promise<Result<Resource<FetchedView>, String>>>,
    api_addr: &str,
    store: Arc<FetchedHyperAST>,
) {
    super::show_repo_menu(ui, &mut aspects.commit.repo);
    ui.push_id(ui.id().with("commit"), |ui| {
        egui::TextEdit::singleline(&mut aspects.commit.id)
            .clip_text(true)
            .desired_width(150.0)
            .desired_rows(1)
            .hint_text("commit")
            .interactive(true)
            .show(ui)
    });
    ui.push_id(ui.id().with("path"), |ui| {
        if egui::TextEdit::singleline(&mut aspects.path)
            .clip_text(true)
            .desired_width(150.0)
            .desired_rows(1)
            .hint_text("path")
            .interactive(true)
            .show(ui)
            .response
            .changed()
        {
            *aspects_result = Some(remote_fetch_node_old(
                ui.ctx(),
                api_addr,
                store.clone(),
                &aspects.commit,
                &aspects.path,
            ));
            // *aspects_result = Some(remote_fetch_tree(ui.ctx(), &aspects.commit, &aspects.path));
        }
        egui::TextEdit::singleline(&mut aspects.hightlight)
            .clip_text(true)
            .desired_width(150.0)
            .desired_rows(1)
            .hint_text("hightlight")
            .interactive(true)
            .show(ui)
    });
    ui.checkbox(&mut aspects.spacing, "Spacing");
    ui.checkbox(&mut aspects.syntax, "Syntax");
    ui.checkbox(&mut aspects.cst, "CST");
    ui.add_enabled_ui(false, |ui| {
        ui.checkbox(&mut aspects.ast, "AST");
        ui.wip(Some(" soon available"));
        ui.checkbox(&mut aspects.type_decls, "Type Decls");
        ui.wip(Some(" soon available"));
        ui.checkbox(&mut aspects.licence, "Licence");
        ui.wip(Some(" soon available"));
        ui.checkbox(&mut aspects.doc, "Doc");
        ui.wip(Some(" soon available"));
    });
    ui.label("serialized Java:");
    let mut rm = None;
    for x in &aspects.ser_opt_java {
        let button = &ui.button(x.to_str());
        if button.clicked() {
            rm = Some(x.clone());
        }
    }
    if let Some(rm) = rm {
        aspects.ser_opt_java.remove(&rm);
    }
    ui.label("serialized Cpp:");
    let mut rm = None;
    for x in &aspects.ser_opt_cpp {
        let button = &ui.button(x.to_str());
        if button.clicked() {
            rm = Some(x.clone());
        }
    }
    if let Some(rm) = rm {
        aspects.ser_opt_cpp.remove(&rm);
    }
    ui.label("hidden Java:");
    let mut rm = None;
    for x in &aspects.hide_opt_java {
        let button = &ui.button(x.to_str());
        if button.clicked() {
            rm = Some(x.clone());
        }
    }
    if let Some(rm) = rm {
        aspects.hide_opt_java.remove(&rm);
    }
    ui.label("hidden Cpp:");
    let mut rm = None;
    for x in &aspects.hide_opt_cpp {
        let button = &ui.button(x.to_str());
        if button.clicked() {
            rm = Some(x.clone());
        }
    }
    if let Some(rm) = rm {
        aspects.hide_opt_cpp.remove(&rm);
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FetchedView {
    #[serde(skip)]
    store: Arc<FetchedHyperAST>,
    #[serde(serialize_with = "ser_node_id", deserialize_with = "de_node_id")]
    pub(crate) root: NodeIdentifier,
    #[serde(skip)]
    /// WARN reset it on changes of state that can affect layout
    prefill_cache: Option<PrefillCache>,
}

fn ser_node_id<S>(id: &NodeIdentifier, s: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    // s.serialize_bytes(&id.to_bytes())
    s.serialize_u32(id.to_u32())
}

#[test]
fn url_limit_on_ids() {
    dbg!(2000 / u64::MAX.to_string().len());
}

fn de_node_id<'de, D>(d: D) -> Result<NodeIdentifier, D::Error>
where
    D: serde::Deserializer<'de>,
{
    use serde::de;
    use std::fmt;
    struct Visitor;
    impl<'de> de::Visitor<'de> for Visitor {
        type Value = NodeIdentifier;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an integer between -2^31 and 2^31")
        }

        // fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
        // where
        //     E: de::Error,
        // {
        //     NodeIdentifier::try_from(v)
        //         .map_err(|_| de::Error::custom(format!("bad node identifier {:?}", v)))
        // }

        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(NodeIdentifier::from_u32((v as u32).try_into().unwrap()))
        }
        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            Ok(NodeIdentifier::from_u32((v as u32).try_into().unwrap()))
        }
    }
    d.deserialize_u64(Visitor)
}

// impl Hash for FetchedView {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         // self.label_list.hash(state);
//         // self.type_sys.hash(state);
//         self.root.hash(state);
//         // self.labeled.hash(state);
//         // self.children.hash(state);
//         // self.both.hash(state);
//         // self.typed.hash(state);
//         // self.prefill_cache.hash(state);
//     }
// }

pub(crate) fn show(
    aspects_result: &mut poll_promise::Promise<Result<types::Resource<FetchedView>, String>>,
    ui: &mut egui::Ui,
    api_addr: &str,
    aspects: &mut types::ComputeConfigAspectViews,
) {
    if let Some(aspects_result) = aspects_result.ready_mut() {
        match aspects_result {
            Ok(aspects_result) => {
                let ui = &mut ui
                    .new_child(egui::UiBuilder::new().max_rect(ui.available_rect_before_wrap()));
                let _scroll = egui::ScrollArea::both()
                    .auto_shrink([false, false])
                    .show_viewport(ui, |ui, _viewport| {
                        ui.set_height(3_000.0);
                        // ui.set_clip_rect(ui.ctx().screen_rect());
                        if let Some(content) = &mut aspects_result.content {
                            let _hightlight: Vec<usize> = aspects
                                .hightlight
                                .split("/")
                                .filter_map(|x| x.parse().ok())
                                .collect();
                            let action = content.show(
                                ui,
                                api_addr,
                                aspects,
                                None,
                                vec![], //(&hightlight, &egui::Color32::RED, &mut None)
                                None,
                                None,
                                &aspects.path,
                            );
                            match action {
                                super::tree_view::Action::SerializeKind(k) => {
                                    use hyperast::types::HyperType;
                                    let k = &k.as_any();
                                    if let Some(k) =
                                        k.downcast_ref::<hyperast_gen_ts_cpp::types::Type>()
                                    {
                                        aspects.ser_opt_cpp.insert(k.to_owned());
                                    } else if let Some(k) =
                                        k.downcast_ref::<hyperast_gen_ts_java::types::Type>()
                                    {
                                        aspects.ser_opt_java.insert(k.to_owned());
                                    }
                                }
                                super::tree_view::Action::HideKind(k) => {
                                    use hyperast::types::HyperType;
                                    let k = &k.as_any();
                                    if let Some(k) =
                                        k.downcast_ref::<hyperast_gen_ts_cpp::types::Type>()
                                    {
                                        aspects.hide_opt_cpp.insert(k.to_owned());
                                    } else if let Some(k) =
                                        k.downcast_ref::<hyperast_gen_ts_java::types::Type>()
                                    {
                                        aspects.hide_opt_java.insert(k.to_owned());
                                    }
                                }
                                _ => (),
                            }
                        }
                    });
                // egui::Window::new("scroller button").show(ui.ctx(), |ui| {
                //     egui::Slider::new(&mut scroll.state.offset.y, 0.0..=200.0).ui(ui);

                //     scroll.state.store(ui.ctx(), scroll.id);
                // });
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
}

pub(crate) struct HightLightHandle<'a> {
    pub path: &'a [usize],
    /// primary key
    pub color: &'a egui::Color32,
    /// secondary key
    pub id: usize,
    /// return value by reference
    pub screen_pos: &'a mut Option<egui::Rect>,
}

impl FetchedView {
    pub(crate) fn show(
        &mut self,
        ui: &mut egui::Ui,
        api_addr: &str,
        aspects: &types::ComputeConfigAspectViews,
        focus: Option<(&[usize], &[NodeIdentifier])>,
        hightlights: Vec<HightLightHandle<'_>>,
        additions: Option<&[u32]>,
        deletions: Option<&[u32]>,
        path: &str,
    ) -> Action {
        let take = self.prefill_cache.take();
        // ui.allocate_space((h, ui.available_size().x).into());
        let path = path.split("/").filter_map(|x| x.parse().ok()).collect();
        let mut imp = FetchedViewImpl::new(
            self.store.clone(),
            aspects,
            take,
            hightlights,
            focus,
            path,
            ui.id(),
            additions,
            deletions,
        );
        let r = imp.show(ui, api_addr, &self.root);
        // wasm_rs_dbg::dbg!(&imp);
        self.prefill_cache = imp.prefill_cache;
        r
    }
}

#[allow(unused)]
impl Resource<FetchedView> {
    fn from_response(ctx: &egui::Context, response: ehttp::Response) -> Self {
        wasm_rs_dbg::dbg!(&response);
        let content_type = response.content_type().unwrap_or_default();
        let text = response.text();
        wasm_rs_dbg::dbg!(&text);
        let text = text.map(|x| serde_json::from_str(x).unwrap());

        Self {
            response,
            content: text,
        }
    }
}
#[allow(unused)]
impl Resource<FetchedNodes> {
    fn from_response(ctx: &egui::Context, response: ehttp::Response) -> Self {
        wasm_rs_dbg::dbg!(&response);
        let content_type = response.content_type().unwrap_or_default();
        let text = response.text();
        let text = text.map(|x| serde_json::from_str(x).unwrap());

        Self {
            response,
            content: text,
        }
    }
}
#[allow(unused)]
impl Resource<FetchedNode> {
    fn from_response(ctx: &egui::Context, response: ehttp::Response) -> Self {
        wasm_rs_dbg::dbg!(&response);
        let content_type = response.content_type().unwrap_or_default();
        let text = response.text();
        let text = text.map(|x| serde_json::from_str(x).unwrap());

        Self {
            response,
            content: text,
        }
    }
}
#[allow(unused)]
impl Resource<FetchedLabels> {
    fn from_response(ctx: &egui::Context, response: ehttp::Response) -> Self {
        wasm_rs_dbg::dbg!(&response);
        let content_type = response.content_type().unwrap_or_default();
        let text = response.text();
        let text = text.map(|x| serde_json::from_str(x).unwrap());

        Self {
            response,
            content: text,
        }
    }
}

pub(super) type RemoteView = Promise<ehttp::Result<Resource<FetchedView>>>;

#[allow(unused)]
pub(super) fn remote_fetch_tree(
    ctx: &egui::Context,
    api_addr: &str,
    commit: &types::Commit,
    path: &str,
) -> Promise<Result<Resource<FetchedView>, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/view/github/{}/{}/{}/{}",
        api_addr, &commit.repo.user, &commit.repo.name, &commit.id, &path,
    );

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        let resource =
            response.map(|response| Resource::<FetchedView>::from_response(&ctx, response));
        sender.send(resource);
    });
    promise
}

pub(super) fn remote_fetch_node_old(
    ctx: &egui::Context,
    api_addr: &str,
    store: Arc<FetchedHyperAST>,
    commit: &types::Commit,
    path: &str,
) -> Promise<Result<Resource<FetchedView>, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/fetch/github/{}/{}/{}/{}",
        api_addr, &commit.repo.user, &commit.repo.name, &commit.id, &path,
    );

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);

    let store = store.clone();
    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        let resource = response.map(|response| {
            let res = Resource::<FetchedNode>::from_response(&ctx, response);
            let fetched_node = res.content.unwrap();
            store
                .node_store
                .write()
                .unwrap()
                .extend(fetched_node.node_store);
            Resource {
                response: res.response,
                content: Some(FetchedView {
                    store,
                    root: fetched_node.root[0],
                    prefill_cache: Default::default(),
                }),
            }
        });

        sender.send(resource);
    });
    promise
}

pub(super) fn remote_fetch_node(
    ctx: &egui::Context,
    api_addr: &str,
    store: Arc<FetchedHyperAST>,
    commit: &types::Commit,
    path: &str,
) -> Promise<Result<NodeIdentifier, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/fetch/github/{}/{}/{}/{}",
        api_addr, &commit.repo.user, &commit.repo.name, &commit.id, &path,
    );

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);

    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        let resource = response.map(|response| {
            let res = Resource::<FetchedNode>::from_response(&ctx, response);
            let fetched_node = res.content.unwrap();
            store
                .node_store
                .write()
                .unwrap()
                .extend(fetched_node.node_store);
            fetched_node.root[0]
        });

        sender.send(resource);
    });
    promise
}

#[allow(unused)]
pub(super) fn remote_fetch_nodes_by_ids(
    ctx: &egui::Context,
    api_addr: &str,
    store: Arc<FetchedHyperAST>,
    repo: &types::Repo,
    ids: HashSet<NodeIdentifier>,
) -> Promise<Result<Resource<()>, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let mut url = format!("http://{}/fetch-ids", api_addr,);
    // TODO group ids by arch
    for id in ids {
        url.push('/');
        let id = id.to_u32();
        url += &id.to_string();
    }

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);
    let store = store.clone();
    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        store.nodes_pending.lock().unwrap().pop_front();
        let resource = response.map(|response| {
            let res = Resource::<FetchedNodes>::from_response(&ctx, response);
            store
                .node_store
                .write()
                .unwrap()
                .extend(res.content.unwrap().node_store);
            Resource {
                response: res.response,
                content: Some(()),
            }
        });
        sender.send(resource);
    });
    promise
}

#[allow(unused)]
pub(super) fn remote_fetch_labels(
    ctx: &egui::Context,
    api_addr: &str,
    store: Arc<FetchedHyperAST>,
    repo: &types::Repo,
    ids: HashSet<LabelIdentifier>,
) -> Promise<Result<Resource<()>, String>> {
    let ctx = ctx.clone();
    let (sender, promise) = Promise::new();
    let mut url = format!("http://{}/fetch-labels", api_addr,);
    for id in ids {
        url.push('/');
        let id: u32 = id.into();
        url += &id.to_string();
    }

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);
    let store = store.clone();
    ehttp::fetch(request, move |response| {
        ctx.request_repaint(); // wake up UI thread
        store.labels_pending.lock().unwrap().pop_front();
        let resource = response.map(|response| {
            let res = Resource::<FetchedLabels>::from_response(&ctx, response);
            let mut hash_map = store.label_store.write().unwrap();
            let fetched_labels = res.content.unwrap();
            for (k, v) in fetched_labels
                .label_ids
                .into_iter()
                .zip(fetched_labels.labels)
            {
                hash_map.insert(k, v);
            }
            Resource {
                response: res.response,
                content: Some(()),
            }
        });
        sender.send(resource);
    });
    promise
}

#[derive(serde::Deserialize)]
pub struct FetchedNodes {
    node_store: fetched::SimplePacked<String>,
}
#[derive(serde::Deserialize)]
pub struct FetchedNode {
    root: Vec<NodeIdentifier>,
    node_store: fetched::SimplePacked<String>,
}

#[derive(serde::Deserialize, Clone, Debug)]
pub struct FetchedLabels {
    label_ids: Vec<fetched::LabelIdentifier>,
    labels: Vec<String>,
}
