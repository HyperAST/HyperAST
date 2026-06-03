use std::hash::{Hash, Hasher};

use axum::Json;
use const_chunks::IteratorConstChunks;
use hyperast::{
    compat::HashMap,
    store::defaults::{LabelIdentifier, NodeIdentifier},
    types::{self, Children, Childrn, HyperAST, LabelStore, Labeled, NodeStore, WithChildren},
};
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::SharedState;

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
pub struct ViewRes {
    type_sys: TypeSys,
    #[serde(flatten)]
    view: View,
}

#[derive(Serialize, Clone, Debug)]
pub struct View {
    root: NodeId,
    label_list: Vec<String>,
    labeled: ViewLabeled,
    children: ViewChildren,
    both: ViewBoth,
    typed: ViewTyped,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ViewTyped {
    ids: Vec<NodeId>,
    kinds: Vec<u16>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ViewLabeled {
    ids: Vec<NodeId>,
    kinds: Vec<u16>,
    labels: Vec<u32>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ViewChildren {
    ids: Vec<NodeId>,
    kinds: Vec<u16>,
    cs_ofs: Vec<u32>,
    cs_lens: Vec<u32>,
    children: Vec<NodeId>,
}

#[derive(Serialize, Clone, Debug, Default)]
pub struct ViewBoth {
    ids: Vec<NodeId>,
    kinds: Vec<u16>,
    labels: Vec<u32>,
    cs_ofs: Vec<u32>,
    cs_lens: Vec<u32>,
    children: Vec<NodeId>,
}

pub fn view(state: SharedState, path: Parameters) -> Result<Json<ViewRes>, String> {
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
        .write()
        .unwrap()
        .get_config(repo_spec)
        .ok_or_else(|| "missing config for repository".to_string())?;
    let mut repo = repo.fetch();
    log::info!("done cloning {}", repo.spec);
    let commits = state
        .repositories
        .write()
        .unwrap()
        .pre_process_with_limit(&mut repo, "", &commit, 2)
        .map_err(|e| e.to_string())?;
    log::info!("done construction of {commits:?} in {}", repo.spec);
    let repositories = state.repositories.read().unwrap();
    let commit_src = repositories.get_commit(&repo.config, &commits[0]).unwrap();
    let src_tr = commit_src.ast_root;
    dbg!(src_tr);
    let node_store = &repositories.processor.main_stores.node_store;
    let label_store = &repositories.processor.main_stores.label_store;

    log::info!("searching for {path:?}");
    let curr = resolve_path(src_tr, path, node_store);
    todo!(
        "should deprecate or accomodate changes in type repr ie. lang + type btw. could allow paking as u16 like before"
    );
    // let type_sys = TypeSys(types::Type::it().map(|x| x.to_string()).collect());

    // let view = make_view(vec![(curr, 20)], &repositories.processor.main_stores);
    // let view_res = ViewRes { type_sys, view };
    // Ok(view_res.into())
}

pub fn view_with_node_id(state: SharedState, id: u64) -> Result<Json<ViewRes>, String> {
    let now = Instant::now();
    if id == 0 {
        return Err("wrong node id".into());
    }
    dbg!(&id);
    let id: NodeIdentifier = unsafe { std::mem::transmute(id) };
    dbg!(&id);
    let mut get_mut = state;
    let repositories = get_mut.repositories.read().unwrap();
    let node_store = &repositories.processor.main_stores.node_store;
    let label_store = &repositories.processor.main_stores.label_store;

    todo!(
        "should deprecate or accomodate changes in type repr ie. lang + type btw. could allow paking as u16 like before"
    );
    // let type_sys = TypeSys(types::Type::it().map(|x| x.to_string()).collect());

    // if node_store.try_resolve(id).is_none() {
    //     return Err(format!("{id:?} is absent from the HyperAST"));
    // }
    // let view = make_view(vec![(id, 8)], &repositories.processor.main_stores);
    // let view_res = ViewRes { type_sys, view };
    // Ok(view_res.into())
}

fn resolve_path(
    root: NodeIdentifier,
    path: Option<String>,
    node_store: &hyperast::store::nodes::legion::NodeStore,
) -> NodeIdentifier {
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

fn make_view<'a, HAST>(
    mut queue: Vec<(HAST::IdN, usize)>,
    stores: &'a HAST,
    // node_store: &hyperast::store::nodes::legion::NodeStore,
    // label_store: &hyperast::store::labels::LabelStore,
) -> View
where
    HAST::IdN: Hash,
    <HAST::TS as types::TypeStore>::Ty: Into<u16>,
    // HAST: NodeStore<HAST::IdN, R<'a> = HAST::T> + LabelStore<str, I = HAST::Label>,
    HAST: HyperAST<Label = LabelIdentifier>,
    HAST::IdN: hyperast::types::NodeId<IdN = HAST::IdN>,
{
    use num::cast::ToPrimitive;
    let mut label_list = vec![];
    let mut labeled = ViewLabeled::default();
    let mut with_children = ViewChildren::default();
    let mut with_both = ViewBoth::default();
    let mut only_typed = ViewTyped::default();
    // let mut ids = vec![];
    // let mut kinds = vec![];
    // let mut cs_ofs = vec![];
    // let mut cs_lens = vec![];
    // let mut children = vec![];
    // let mut labels = vec![];
    let mut label_map = HashMap::<LabelIdentifier, u32>::default();

    #[derive(Default)]
    pub struct EntityHasher(u64);
    impl Hasher for EntityHasher {
        fn write(&mut self, a: &[u8]) {
            self.0 = u64::from_be_bytes(
                a.into_iter()
                    .cloned()
                    .const_chunks::<8>()
                    .next()
                    .unwrap()
                    .clone(),
            )
        }
        fn finish(&self) -> u64 {
            self.0
        }
    }

    assert_eq!(1, queue.len());
    let root = {
        let mut id = EntityHasher::default();
        queue[0].0.hash(&mut id);
        let nid = id.finish();
        nid
    };

    while let Some((curr, advance)) = queue.pop() {
        let mut id = EntityHasher::default();
        curr.hash(&mut id);
        let nid = id.finish();
        let n = stores.node_store().resolve(&curr); //hyperast::types::NodeStore::resolve(stores, &curr);
        let k = stores.resolve_type(&curr);
        if let Some(l) = n.try_get_label() {
            let l = label_map.entry(*l).or_insert_with(|| {
                let i = label_list.len() as u32;
                label_list.push(*l);
                i
            });
            if let Some(cs) = n.children() {
                with_both.ids.push(nid);
                with_both.kinds.push(k.into());
                with_both.cs_ofs.push(with_both.children.len() as u32);
                with_both.cs_lens.push(cs.child_count().to_u32().unwrap());
                with_both.children.extend(cs.iter_children().map(|curr| {
                    if advance > 0 {
                        queue.push((curr.clone(), advance - 1));
                    }
                    let mut id = EntityHasher::default();
                    curr.hash(&mut id);
                    let id = id.finish();
                    id
                }));
                with_both.labels.push(*l);
            } else {
                labeled.ids.push(nid);
                labeled.kinds.push(k.into());
                labeled.labels.push(*l);
            }
        } else if let Some(cs) = n.children() {
            with_children.ids.push(nid);
            with_children.kinds.push(k.into());
            with_children
                .cs_ofs
                .push(with_children.children.len() as u32);
            with_children
                .cs_lens
                .push(cs.child_count().to_u32().unwrap());
            with_children
                .children
                .extend(cs.iter_children().map(|curr| {
                    if advance > 0 {
                        queue.push((curr.clone(), advance - 1));
                    }
                    let mut id = EntityHasher::default();
                    curr.hash(&mut id);
                    let id = id.finish();
                    id
                }));
        } else {
            only_typed.ids.push(nid);
            only_typed.kinds.push(k.into());
        }
    }
    dbg!(&labeled.ids.len());
    dbg!(&with_children.ids.len());
    dbg!(&with_both.ids.len());
    dbg!(&only_typed.ids.len());
    let label_list = label_list
        .into_iter()
        .map(|l| stores.label_store().resolve(&l).to_string())
        .collect();
    let view = View {
        label_list,
        root,
        labeled,
        children: with_children,
        both: with_both,
        typed: only_typed,
    };
    view
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
