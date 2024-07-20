use hyper_ast::{
    store::{
        defaults,
        labels::label_id_from_usize, // ::fetched,
        nodes::{
            self,
            fetched::{self, NodeIdentifier},
        },
    },
    types::WithChildren,
};
use hyper_ast_cvs_git::TStore;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::{app::Timed, SharedState};

#[derive(Deserialize, Clone, Debug)]
pub struct Parameters {
    user: String,
    name: String,
    commit: String,
    path: Option<String>,
}

type NodeId = u64;

#[derive(Serialize, Clone, Debug)]
pub struct TypeSys(Vec<String>);

#[derive(Serialize, Clone, Debug)]
pub struct FetchedRes {
    // #[serde(serialize_with = "ser_label_store")]
    // label_store: fetched::FetchedLabels,
    label_ids: Vec<nodes::fetched::LabelIdentifier>,
    labels: Vec<String>,
    variants: Vec<nodes::fetched::RawVariant>,
}

#[derive(Serialize, Clone, Debug)]
pub struct FetchedLabels {
    label_ids: Vec<nodes::fetched::LabelIdentifier>,
    labels: Vec<String>,
}

#[derive(Serialize)]
pub struct FetchedNodes {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    root: Vec<NodeIdentifier>,
    node_store: fetched::SimplePacked<&'static str>,
}

pub fn fetch(mut state: SharedState, path: Parameters) -> Result<FetchedNodes, String> {
    let now = Instant::now();
    let Parameters {
        user,
        name,
        commit,
        path,
    } = path;
    dbg!(&path);
    let repo_spec = hyper_ast_cvs_git::git::Forge::Github.repo(user, name);
    let repo = state
        .repositories
        .write()
        .unwrap()
        .get_config(repo_spec)
        .ok_or_else(|| "missing config for repository".to_string())?;
    let mut repo = repo.fetch();
    log::warn!("done cloning {}", repo.spec);
    let commits = state
        .repositories
        .write()
        .unwrap()
        .pre_process_with_limit(&mut repo, "", &commit, 2)
        .map_err(|e| e.to_string())?;
    log::warn!("done construction of {commits:?} in {}", repo.spec);
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories.get_commit(&repo.config, &commits[0]).unwrap();
    let src_tr = commit_src.ast_root;
    dbg!(src_tr);
    let node_store = &repositories.processor.main_stores.node_store;

    log::error!("searching for {path:?}");
    let curr = resolve_path(src_tr, path, node_store);
    dbg!(curr);
    let ids = vec![curr];
    let node_store = extract_nodes(
        &ids,
        &repositories.processor.main_stores, //label_store
    );
    dbg!(&ids);
    let ids = ids.into_iter().map(|x| x.into()).collect();
    Ok(FetchedNodes {
        node_store,
        root: ids,
    })
}

pub fn fetch_with_node_ids<'a>(
    state: SharedState,
    ids: impl Iterator<Item = &'a str>,
) -> Result<Timed<FetchedNodes>, String> {
    let now = Instant::now();
    let ids: Vec<_> = ids
        .into_iter()
        .map(|id| {
            let id: u64 = id.parse().unwrap();
            if id == 0 {
                panic!()
            }
            let id: defaults::NodeIdentifier = unsafe { std::mem::transmute(id) };
            id
        })
        .collect();
    let mut get_mut = state;
    let repositories = get_mut.repositories.read().unwrap();

    let node_store = extract_nodes(
        &ids,
        &repositories.processor.main_stores, //label_store
    );
    Ok(Timed {
        time: now.elapsed().as_secs_f64(),
        content: FetchedNodes {
            node_store,
            root: vec![],
        },
    })
}

pub fn fetch_labels<'a>(
    state: SharedState,
    ids: impl Iterator<Item = &'a str>,
) -> Result<Timed<FetchedLabels>, String> {
    let now = Instant::now();
    let ids = ids.into_iter().map(|id| {
        let id: usize = id.parse().unwrap();
        // if id == 0 {
        //     panic!()
        // }
        let id = label_id_from_usize(id).unwrap();
        id
    });
    let mut get_mut = state;
    let repositories = get_mut.repositories.read().unwrap();
    let node_store = &repositories.processor.main_stores.node_store;
    let label_store = &repositories.processor.main_stores.label_store;
    use hyper_ast::types::LabelStore;
    let (label_ids, labels) = ids
        .map(|x| {
            (
                nodes::fetched::LabelIdentifier::from(x),
                label_store.resolve(&x).to_string(),
            )
        })
        .unzip();
    Ok(Timed {
        time: now.elapsed().as_secs_f64(),
        content: FetchedLabels { label_ids, labels },
    })
}

fn resolve_path(
    root: defaults::NodeIdentifier,
    path: Option<String>,
    node_store: &hyper_ast::store::nodes::legion::NodeStore,
) -> defaults::NodeIdentifier {
    let mut curr = root;
    for i in path.unwrap_or_default().split("/") {
        dbg!(i);
        let i = i.parse();
        let Ok(i) = i else { break };
        let Some(n) = node_store.resolve(curr).child(&i) else {
            break;
        };
        curr = n;
    }
    curr
}

/// ids would better be deduplicated
fn extract_nodes(
    ids: &[defaults::NodeIdentifier],
    store: &hyper_ast::store::SimpleStores<TStore>,
    // label_store: &hyper_ast::store::labels::LabelStore,
) -> fetched::SimplePacked<&'static str> {
    let mut builder = fetched::SimplePackedBuilder::default();
    for id in ids {
        let node = store.node_store.resolve(*id);
        builder.add(&store.type_store, id.clone().into(), node);
    }
    // dbg!(&ids);

    builder.build(
        // label_store
    )
}

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
