use hyperast::{
    store::{
        defaults,
        labels::label_id_from_usize, // ::fetched,
        nodes::{
            self,
            fetched::{self, NodeIdentifier},
        },
    },
    types::{Childrn, WithChildren, WithSerialization, WithStats},
};
use hyperast_vcs_git::TStore;
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
    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo(user, name);
    let repo = state
        .repositories
        .read()
        .unwrap()
        .get_config(repo_spec)
        .ok_or_else(|| "missing config for repository".to_string())?;
    let mut repo = repo.fetch();
    log::info!("done cloning {}", repo.spec);

    let commits = crate::utils::handle_pre_processing(&state, &mut repo, "", &commit, 2)
        .map_err(|e| e.to_string())?;
    log::info!("done construction of {commits:?} in {}", repo.spec);
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories.get_commit(&repo.config, &commits[0]).unwrap();
    let src_tr = commit_src.ast_root;
    dbg!(src_tr);
    let stores = &repositories.processor.main_stores;
    let node_store = &stores.node_store;

    log::info!("searching for {path:?}");
    let curr = if let Some(path) = path {
        if let Some((path, rest)) = path.split_once(":") {
            resolve_file_path(stores, src_tr, path.split("/"))
                .and_then(|d| resolve_in_file(stores, d, rest))
        } else {
            resolve_path(node_store, src_tr, path.split("/"))
        }
    } else {
        Ok(src_tr)
    };
    let curr = match curr {
        Ok(x) => dbg!(x),
        Err(x) => dbg!(x),
    };
    let ids = vec![curr];
    let node_store = extract_nodes(&ids, &repositories.processor.main_stores);
    dbg!(&ids);
    let ids = ids.into_iter().map(|x| x.into()).collect();
    Ok(FetchedNodes {
        node_store,
        root: ids,
    })
}

fn resolve_file_path<'a>(
    stores: &hyperast::store::SimpleStores<TStore>,
    root: defaults::NodeIdentifier,
    mut path: impl Iterator<Item = &'a str>,
) -> Result<defaults::NodeIdentifier, defaults::NodeIdentifier> {
    use hyperast::types::LabelStore;
    let mut d = root;
    loop {
        let Some(l) = path.next() else {
            break Ok(d);
        };
        let n = stores.node_store.resolve(d);
        let Some(l) = stores.label_store.get(l) else {
            break Err(d);
        };
        let Some(n) = n.get_child_by_name(&l) else {
            break Err(d);
        };
        d = n
    }
}

fn resolve_in_file(
    stores: &hyperast::store::SimpleStores<TStore>,
    root: defaults::NodeIdentifier,
    rest: &str,
) -> Result<defaults::NodeIdentifier, defaults::NodeIdentifier> {
    let mut d = root;
    if rest.is_empty() {
        return Ok(d);
    }
    let byte_r: Vec<_> = rest.split("-").collect();
    if let Ok(range) = TryInto::<[&str; 2]>::try_into(byte_r) {
        // start-end
        // TODO use try_map when stable
        let [Ok(start), Ok(end)] = [range[0].parse::<usize>(), range[1].parse::<usize>()] else {
            return Err(d);
        };
        if end > start {
            return Err(d);
        }
        let mut b = 0;
        loop {
            let n = stores.node_store.resolve(d);
            let Some(n) = n.children() else {
                if b <= start
                    && start <= b + n.try_bytes_len().unwrap_or_default()
                    && b <= end
                    && end <= b + n.try_bytes_len().unwrap_or_default()
                {
                    return Ok(d);
                }
                return Err(d);
            };
            // TODO debug all that garbage
            for i in n.iter_children() {
                let n = stores.node_store.resolve(d);
                let l = n.try_bytes_len().unwrap_or_default();
                if start < b {
                    b += l;
                    d = i;
                    continue;
                }
                d = i;
                return Err(d);
            }
        }
    }
    let byte_r: Vec<_> = rest.split(":").collect();
    let (row, col) = if let Ok(row_col) = TryInto::<[&str; 2]>::try_into(byte_r) {
        // row:col
        let [Ok(row), Ok(col)] = [row_col[0].parse::<usize>(), row_col[1].parse::<usize>()] else {
            return Err(d);
        };
        (row, Some(col))
    } else if let Ok(row) = rest.parse::<usize>() {
        (row, None)
    } else {
        return Err(d);
    };
    let n = stores.node_store.resolve(d);
    if row > n.line_count() {
        return Err(d);
    }
    let mut l = 0;
    'l: loop {
        let Some(n) = n.children() else {
            return Err(d);
        };
        // TODO debug all that garbage
        for i in n.iter_children() {
            let n = stores.node_store.resolve(i);
            if n.line_count() == 0 && l == row {
                d = i;
                break 'l;
            }
            if l < row && row < l + n.line_count() {
                l += n.line_count();
                d = i;
            }
        }
        return Err(d);
    }
    let Some(col) = col else {
        return Ok(d);
    };
    // TODO also use the col
    Err(d)
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
    use hyperast::types::LabelStore;
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

fn resolve_path<'a>(
    node_store: &hyperast::store::nodes::legion::NodeStore,
    root: defaults::NodeIdentifier,
    mut path: impl Iterator<Item = &'a str>,
) -> Result<defaults::NodeIdentifier, defaults::NodeIdentifier> {
    let mut curr = root;
    while let Some(i) = path.next() {
        let Ok(i) = i.parse() else {
            return Err(curr);
        };
        let Some(n) = node_store.resolve(curr).child(&i) else {
            return Err(curr);
        };
        curr = n;
    }
    Ok(curr)
}

/// ids would better be deduplicated
fn extract_nodes(
    ids: &[defaults::NodeIdentifier],
    store: &hyperast::store::SimpleStores<TStore>,
    // label_store: &hyperast::store::labels::LabelStore,
) -> fetched::SimplePacked<&'static str> {
    let mut builder = fetched::SimplePackedBuilder::default();
    for id in ids {
        builder.add(store, id);
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
