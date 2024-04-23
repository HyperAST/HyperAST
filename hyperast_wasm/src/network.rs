use crate::store::FetchedHyperAST;
use crate::types;
use types::ApiAddr;
use hyper_ast::store::nodes::fetched;
use hyper_ast::store::nodes::fetched::LabelIdentifier;
use hyper_ast::store::nodes::fetched::NodeIdentifier;
use poll_promise::Promise;

use std::sync::Arc;
use std::{
    collections::{HashSet},
};

#[derive(Debug)]
pub(crate) struct Resource<T> {
    /// HTTP response
    pub(crate) response: ehttp::Response,

    pub(crate) content: Option<T>,
    // /// If set, the response was an image.
    // image: Option<RetainedImage>,
}

impl<T> Resource<T> {
    pub fn map<U, F: Fn(T) -> U>(self, f: F) -> Resource<U> {
        Resource {
            response: self.response,
            content: self.content.map(f),
        }
    }
}

impl Resource<FetchedNodes> {
    fn from_response(response: ehttp::Response) -> Self {
        wasm_rs_dbg::dbg!(&response);
        let _content_type = response.content_type().unwrap_or_default();
        let text = response.text();
        let text = text.map(|x| serde_json::from_str(x).unwrap());

        Self {
            response,
            content: text,
        }
    }
}
impl Resource<FetchedNode> {
    fn from_response(response: ehttp::Response) -> Self {
        wasm_rs_dbg::dbg!(&response);
        let _content_type = response.content_type().unwrap_or_default();
        let text = response.text();
        let text = text.map(|x| serde_json::from_str(x).unwrap());

        Self {
            response,
            content: text,
        }
    }
}
impl Resource<FetchedLabels> {
    fn from_response(response: ehttp::Response) -> Self {
        wasm_rs_dbg::dbg!(&response);
        let _content_type = response.content_type().unwrap_or_default();
        let text = response.text();
        let text = text.map(|x| serde_json::from_str(x).unwrap());

        Self {
            response,
            content: text,
        }
    }
}

impl Resource<FetchedView> {
    fn from_response(response: ehttp::Response) -> Self {
        wasm_rs_dbg::dbg!(&response);
        let _content_type = response.content_type().unwrap_or_default();
        let text = response.text();
        wasm_rs_dbg::dbg!(&text);
        let text = text.map(|x| serde_json::from_str(x).unwrap());

        Self {
            response,
            content: text,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct FetchedView {
    #[serde(skip)]
    store: Arc<FetchedHyperAST>,
    #[serde(serialize_with = "ser_node_id", deserialize_with = "de_node_id")]
    root: NodeIdentifier,
    // #[serde(skip)]
    // /// WARN reset it on changes of state that can affect layout
    // prefill_cache: Option<PrefillCache>,
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

pub(super) type RemoteView = Promise<ehttp::Result<Resource<FetchedView>>>;

pub(super) fn remote_fetch_tree(
    api_addr: &ApiAddr,
    commit: &types::Commit,
    path: &str,
) -> Promise<Result<Resource<FetchedView>, String>> {
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/view/github/{}/{}/{}/{}",
        api_addr, &commit.repo.user, &commit.repo.name, &commit.id, &path,
    );

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);

    ehttp::fetch(request, move |response| {
        let resource = response.map(|response| Resource::<FetchedView>::from_response(response));
        sender.send(resource);
    });
    promise
}

pub(super) fn remote_fetch_node(
    api_addr: &ApiAddr,
    store: Arc<FetchedHyperAST>,
    commit: &types::Commit,
    path: &str,
) -> Promise<Result<Resource<FetchedView>, String>> {
    let (sender, promise) = Promise::new();
    let url = format!(
        "http://{}/fetch/github/{}/{}/{}/{}",
        api_addr, &commit.repo.user, &commit.repo.name, &commit.id, &path,
    );

    wasm_rs_dbg::dbg!(&url);
    let request = ehttp::Request::get(&url);

    let store = store.clone();
    ehttp::fetch(request, move |response| {
        let resource = response.map(|response| {
            let res = Resource::<FetchedNode>::from_response(response);
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
                    // prefill_cache: Default::default(),
                }),
            }
        });

        sender.send(resource);
    });
    promise
}

pub(super) fn remote_fetch_nodes_by_ids(
    api_addr: &ApiAddr,
    store: Arc<FetchedHyperAST>,
    _repo: &types::Repo,
    ids: HashSet<NodeIdentifier>,
) -> Promise<Result<Resource<()>, String>> {
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
        store.nodes_pending.lock().unwrap().pop_front();
        let resource = response.map(|response| {
            let res = Resource::<FetchedNodes>::from_response(response);
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

pub(super) fn remote_fetch_labels(
    api_addr: &ApiAddr,
    store: Arc<FetchedHyperAST>,
    _repo: &types::Repo,
    ids: HashSet<LabelIdentifier>,
) -> Promise<Result<Resource<()>, String>> {
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
        store.labels_pending.lock().unwrap().pop_front();
        let resource = response.map(|response| {
            let res = Resource::<FetchedLabels>::from_response(response);
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
