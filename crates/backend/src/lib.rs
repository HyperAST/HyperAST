// #![feature(array_chunks)]
// #![feature(map_many_mut)]
// #![feature(iter_collect_into)]
#![allow(unused)]
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use dashmap::DashMap;
use hyper_diff::matchers::mapping_store::VecStore;
use hyperast_vcs_git::multi_preprocessed::PreProcessedRepositories;

use axum::body::Bytes;
use hyperast::store::nodes::legion::NodeIdentifier;

pub mod app;
mod changes;
pub mod cli;
mod commit;
pub mod examples;
mod fetch;
mod file;
mod matching;
mod pull_requests;
mod querying;
mod scriptingv1;
pub mod smells;
pub mod track;
#[cfg(feature = "tsg")]
mod tsg;
mod utils;
mod view;
mod ws;
pub use ws::ws_handler;

// #[derive(Default)]
pub struct AppState {
    pub db: DashMap<String, Bytes>,
    pub repositories: RwLock<PreProcessedRepositories>,
    // configs: RwLock<RepoConfigs>,
    mappings: MappingCache,
    mappings_alone: MappingAloneCache,
    partial_decomps: PartialDecompCache,
    // Single shared doc
    doc: Arc<(
        RwLock<automerge::AutoCommit>,
        (
            tokio::sync::broadcast::Sender<(SocketAddr, Vec<automerge::Change>)>,
            tokio::sync::broadcast::Receiver<(SocketAddr, Vec<automerge::Change>)>,
        ),
        RwLock<Vec<tokio::sync::mpsc::Sender<Option<Vec<u8>>>>>,
    )>,
    // Multiple shared docs
    doc2: ws::SharedDocs,
    pr_cache: RwLock<std::collections::HashMap<commit::Param, pull_requests::RawPrData>>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            db: Default::default(),
            repositories: Default::default(),
            mappings: Default::default(),
            mappings_alone: Default::default(),
            partial_decomps: Default::default(),
            doc: Arc::new((
                RwLock::new(automerge::AutoCommit::new()),
                tokio::sync::broadcast::channel(50),
                Default::default(),
            )),
            doc2: Default::default(),
            pr_cache: Default::default(),
        }
    }
}

// pub(crate) type PartialDecompCache = DashMap<NodeIdentifier, DS<NodeIdentifier>>;
pub(crate) type PartialDecompCache = clashmap::ClashMap<NodeIdentifier, DS<NodeIdentifier>>;
pub(crate) type MappingAloneCache =
    DashMap<(NodeIdentifier, NodeIdentifier), (MappingStage, VecStore<u32>)>;
pub(crate) type MappingAloneCacheRef<'a> =
    dashmap::mapref::one::Ref<'a, (NodeIdentifier, NodeIdentifier), (MappingStage, VecStore<u32>)>;

pub(crate) enum MappingStage {
    Subtree,
    Bottomup,
    Decls,
}

type DS<T> = hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<T, u32>;
pub type PersistableMappings<I> = hyper_diff::matchers::Mapping<DS<I>, DS<I>, VecStore<u32>>;
pub(crate) type MappingCache =
    DashMap<(NodeIdentifier, NodeIdentifier), PersistableMappings<NodeIdentifier>>;
type SharedState = Arc<AppState>;

pub(crate) use hyperast_vcs_git::no_space;

#[cfg(feature = "rerun")]
pub mod log_languages {
    //! Log the grammar of corresponding language to rerun.

    use core::f32;
    use std::{ops::Mul, sync::Arc};

    use rerun::external::{arrow, re_types};
    pub fn log_languages() -> Result<(), Box<dyn std::error::Error>> {
        let rec = rerun::RecordingStream::global(rerun::StoreKind::Recording).unwrap();

        let lang = polyglote::Lang {
            language: hyperast_gen_ts_java::language(),
            name: "java",
            node_types: hyperast_gen_ts_java::node_types(),
            highlights: "",
            tags: "",
            injects: "",
        };
        let types = polyglote::preprocess_aux(&lang)?;

        eprintln!("{}", types);

        log_3dgraph_language(&rec, &lang, &types)?;
        log_2dgraph_language(&rec, &lang, &types)?;

        let lang = polyglote::Lang {
            language: hyperast_gen_ts_cpp::language(),
            name: "cpp",
            node_types: hyperast_gen_ts_cpp::node_types(),
            highlights: "",
            tags: "",
            injects: "",
        };
        let types = polyglote::preprocess_aux(&lang)?;

        eprintln!("{}", types);

        log_3dgraph_language(&rec, &lang, &types)?;
        log_2dgraph_language(&rec, &lang, &types)?;

        let lang = polyglote::Lang {
            language: hyperast_gen_ts_tsquery::language(),
            name: "tsquery",
            node_types: hyperast_gen_ts_tsquery::node_types(),
            highlights: "",
            tags: "",
            injects: "",
        };
        let types = polyglote::preprocess_aux(&lang)?;

        eprintln!("{}", types);

        log_3dgraph_language(&rec, &lang, &types)?;
        log_2dgraph_language(&rec, &lang, &types)?;

        Ok(())
    }

    fn log_2dgraph_language(
        rec: &rerun::RecordingStream,
        lang: &polyglote::Lang,
        types: &polyglote::preprocess::TypeSys,
    ) -> rerun::RecordingStreamResult<()> {
        let path = &["language", lang.name].map(|x| x.into())[..];
        fn split_f(
            mut acc: (Vec<String>, Vec<String>),
            x: (String, bool),
        ) -> (Vec<String>, Vec<String>) {
            if x.1 { &mut acc.0 } else { &mut acc.1 }.push(x.0);
            acc
        }
        let leafs = types.leafs().fold((vec![], vec![]), split_f);
        let concrete = types.concrete().fold((vec![], vec![]), split_f);
        let r#abstract = types.r#abstract().collect();

        let triplet = make_triplet(&leafs.1, [80, 140, 255]); // unnamed
        let triplet = chaining(triplet, make_triplet(&leafs.0, [140, 80, 255])); //named
        let triplet = chaining(triplet, make_triplet(&concrete.0, [80, 255, 140])); //with_fields
        let triplet = chaining(triplet, make_triplet(&concrete.1, [140, 255, 80])); //named
        let triplet = chaining(triplet, make_triplet(&r#abstract, [255, 80, 80]));

        let (node_ids, labels, colors) = triplet;
        let graph_nodes = rerun::GraphNodes::new(node_ids)
            .with_labels(labels)
            .with_colors(colors);
        rec.log(path, &graph_nodes)?;

        let subtypes = types.r#abstract_subtypes();
        let concrete_children = types.concrete_children();
        let concrete_fields = types
            .concrete_fields()
            .map(|(x, v)| (x.clone(), v.iter().map(|x| x.1.clone()).collect::<Vec<_>>()));
        let arr = subtypes.chain(concrete_children).chain(concrete_fields);
        let graph_edges = rerun::GraphEdges::new(
            arr.into_iter()
                .map(|(t, cs)| cs.into_iter().map(|y| (t.clone(), y)).collect::<Vec<_>>())
                .flatten()
                .collect::<Vec<_>>(),
        )
        .with_directed_edges();
        rec.log(path, &graph_edges)
    }
    fn log_3dgraph_language(
        rec: &rerun::RecordingStream,
        lang: &polyglote::Lang,
        types: &polyglote::preprocess::TypeSys,
    ) -> rerun::RecordingStreamResult<()> {
        let mut map = Map {
            map: std::collections::HashMap::default(),
            rec: rec.clone(),
            name: lang.name,
        };
        let r#abstract = types.r#abstract().collect();
        dbg!(&r#abstract);
        map.log(r#abstract, "abstract", "abstract", 20.0)?;
        let leafs = types.leafs().fold((vec![], vec![]), split_f);
        map.log(leafs.1, "leaf", "unnamed", 0.0)?;
        map.log(leafs.0, "leaf", "named", 5.0)?;
        let concrete = types.concrete().fold((vec![], vec![]), split_f);
        map.log(concrete.0, "concrete", "with_fields", 10.0)?;
        map.log(concrete.1, "concrete", "named", 15.0)?;
        let subtypes = types.r#abstract_subtypes();
        rec.clone().log(
            format!("language/{}/subtyping", lang.name),
            &map.arrows(subtypes),
        )?;
        let concrete_children = types.concrete_children();
        rec.clone().log(
            format!("language/{}/children", lang.name),
            &map.arrows(concrete_children),
        )?;
        let concrete_fields: Vec<_> = types.concrete_fields().collect();
        let labels = concrete_fields
            .iter()
            .flat_map(|x| x.1.iter().map(|x| x.0.clone()));
        let concrete_fields = concrete_fields
            .iter()
            .map(|(x, v)| (x.clone(), v.iter().map(|x| x.1.clone()).collect::<Vec<_>>()));
        rec.clone().log(
            format!("language/{}/fields", lang.name),
            &map.arrows(concrete_fields).with_labels(labels),
        )?;
        Ok(())
    }

    struct Map {
        map: std::collections::HashMap<String, [f32; 3]>,
        rec: rerun::RecordingStream,
        name: &'static str,
    }
    impl Map {
        fn points(&mut self, v: Vec<String>, o: f32) -> rerun::Points3D {
            let l = (v.len() as f32) / f32::consts::TAU;
            rerun::Points3D::new((0..v.len()).map(|i| {
                let rad = (i as f32 / v.len() as f32).mul(f32::consts::TAU);
                let p = [rad.sin() * l, o, rad.cos() * l];
                self.map.insert(v[i].clone(), p.clone());
                p
            }))
            .with_labels(v.into_iter().map(|x| x))
        }
        fn arrows(&self, v: impl Iterator<Item = (String, Vec<String>)>) -> rerun::Arrows3D {
            let translate = |x: &[f32; 3], y: &[f32; 3]| [y[0] - x[0], y[1] - x[1], y[2] - x[2]];
            let (ori, dest): (Vec<&[f32; 3]>, Vec<[f32; 3]>) = v
                .map(|(t, cs)| {
                    let x = self.map.get(&t).unwrap();
                    cs.into_iter()
                        .filter_map(|t| self.map.get(&t))
                        .map(|y| (x, translate(x, y)))
                        .collect::<Vec<_>>()
                })
                .flatten()
                .unzip();
            rerun::Arrows3D::from_vectors(dest).with_origins(ori)
        }

        fn log(
            &mut self,
            v: Vec<String>,
            t: &str,
            adj: &str,
            o: f32,
        ) -> rerun::RecordingStreamResult<()> {
            let path = ["language", self.name, t, adj].map(|x| x.into());
            let path = if adj.is_empty() {
                &path[..path.len() - 1]
            } else {
                &path[..]
            };
            let points = self.points(v, o);
            self.rec.log(path, &points)
        }
    }

    fn split_f(
        mut acc: (Vec<String>, Vec<String>),
        x: (String, bool),
    ) -> (Vec<String>, Vec<String>) {
        if x.1 { &mut acc.0 } else { &mut acc.1 }.push(x.0);
        acc
    }

    fn make_triplet(
        v: &Vec<String>,
        color: [u8; 3],
    ) -> (
        impl Iterator<Item = String> + '_,
        impl Iterator<Item = String> + '_,
        impl Iterator<Item = [u8; 3]> + '_,
    ) {
        let colors = (0..v.len()).map(move |_| color.clone());
        let node_ids = v.iter().cloned();
        let labels = v.iter().cloned();
        (node_ids, labels, colors)
    }
    fn chaining<
        It0: Iterator,
        It1: Iterator,
        It2: Iterator,
        Itb0: Iterator<Item = It0::Item>,
        Itb1: Iterator<Item = It1::Item>,
        Itb2: Iterator<Item = It2::Item>,
    >(
        ta: (It0, It1, It2),
        tb: (Itb0, Itb1, Itb2),
    ) -> (
        impl Iterator<Item = It0::Item>,
        impl Iterator<Item = It1::Item>,
        impl Iterator<Item = It2::Item>,
    ) {
        (ta.0.chain(tb.0), ta.1.chain(tb.1), ta.2.chain(tb.2))
    }
}

#[ignore] // ignore (from normal cargo test) for now, later make a feature
#[test]
// slow test, more of an integration test, try using release
fn test_measuring_size() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("backend=debug,hyperast_vcs_git=info,hyperast=error")
        .try_init()
        .unwrap();

    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo("graphhopper", "graphhopper");
    let config = hyperast_vcs_git::processing::RepoConfig::JavaMaven;
    let commit = "f5f2b7765e6b392c5e8c7855986153af82cc1abe";
    let language = "Java";
    let prepro = hyperast::scripting::lua_scripting::PREPRO_SIZE_WITH_FINISH.into();
    run_scripting(repo_spec, config, commit, language, prepro, "size")
}

#[ignore] // ignore (from normal cargo test) for now, later make a feature
#[test]
// slow test, more of an integration test, try using release
fn test_measuring_mcc() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("backend=debug,hyperast_vcs_git=info,hyperast=error")
        .try_init()
        .unwrap();

    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo("graphhopper", "graphhopper");
    let config = hyperast_vcs_git::processing::RepoConfig::JavaMaven;
    let commit = "f5f2b7765e6b392c5e8c7855986153af82cc1abe";
    let language = "Java";
    let prepro = hyperast::scripting::lua_scripting::PREPRO_MCC_WITH_FINISH.into();
    run_scripting(repo_spec, config, commit, language, prepro, "mcc")
}

#[ignore] // ignore (from normal cargo test) for now, later make a feature
#[test]
// slow test, more of an integration test, try using release
fn test_measuring_loc() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("backend=debug,hyperast_vcs_git=info,hyperast=error")
        .try_init()
        .unwrap();

    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo("graphhopper", "graphhopper");
    let config = hyperast_vcs_git::processing::RepoConfig::JavaMaven;
    let commit = "f5f2b7765e6b392c5e8c7855986153af82cc1abe";
    let language = "Java";
    let prepro = hyperast::scripting::lua_scripting::PREPRO_LOC.into();
    run_scripting(repo_spec, config, commit, language, prepro, "LoC")
}

fn run_scripting(
    repo_spec: hyperast_vcs_git::git::Repo,
    config: hyperast_vcs_git::processing::RepoConfig,
    commit: &str,
    language: &str,
    prepro: &str,
    show: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let state = crate::AppState::default();
    state
        .repositories
        .write()
        .unwrap()
        .register_config_with_prepro(repo_spec.clone(), config, prepro.into());
    // state
    //     .repositories
    //     .write()
    //     .unwrap()
    //     .register_config(repo_spec.clone(), config);
    let repo = state
        .repositories
        .read()
        .unwrap()
        .get_config(repo_spec)
        .ok_or_else(|| "missing config for repository".to_string())?;
    let mut repository = repo.fetch();
    log::debug!("done cloning {}", repository.spec);
    let commits = state.repositories.write().unwrap().pre_process_with_limit(
        &mut repository,
        "",
        &commit,
        1,
    )?;
    {
        let repositories = state.repositories.read().unwrap();
        let commit = repositories
            .get_commit(&repository.config, &commits[0])
            .unwrap();
        let store = &state.repositories.read().unwrap().processor.main_stores;
        let n = store.node_store.resolve(commit.ast_root);
        let dd = n
            .get_component::<hyperast::scripting::DerivedData>()
            .unwrap();
        let s = dd.0.get(show);
        log::debug!("{show} ! {:?}", s);
        log::debug!("size:{}", n.size());
        log::debug!("size_no_spaces:{}", n.size_no_spaces());
        log::debug!("height:{}", n.height());
        if let Ok(mcc) = n.get_component::<hyperast::cyclomatic::Mcc>() {
            log::debug!("Mcc:{:?}", mcc);
        }
    }
    let commits = state.repositories.write().unwrap().pre_process_with_limit(
        &mut repository,
        "",
        &commit,
        2,
    )?;
    let repositories = state.repositories.read().unwrap();
    let commit = repositories
        .get_commit(&repository.config, &commits[1])
        .unwrap();
    let stores = &state.repositories.read().unwrap().processor.main_stores;
    let n = stores.node_store.resolve(commit.ast_root);
    let dd = n
        .get_component::<hyperast::scripting::DerivedData>()
        .unwrap();
    let s = dd.0.get(show);
    log::debug!("{:?}", s);
    use hyperast::types::WithStats;
    log::debug!("size:{}", n.size());
    log::debug!("size_no_spaces:{}", n.size_no_spaces());
    log::debug!("height:{}", n.height());
    if let Ok(mcc) = n.get_component::<hyperast::cyclomatic::Mcc>() {
        log::debug!("Mcc:{:?}", mcc);
    }

    #[cfg(feature = "subtree-stats")]
    {
        log::error!(
            "height_counts_non_dedup : {:3?}",
            stores.node_store.inner.height_counts_non_dedup
        );
        log::error!(
            "height_counts           : {:3?}",
            stores.node_store.inner.height_counts
        );
        log::error!(
            "height_counts_label     : {:3?}",
            stores.node_store.inner.height_counts_label
        );
        log::error!(
            "height_counts_structural: {:3?}",
            stores.node_store.inner.height_counts_structural
        );
    }

    Ok(())
}

#[ignore] // ignore (from normal cargo test) for now, later make a feature
#[test]
// slow test, more of an integration test, try using release
fn test_tsg_incr() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("backend=debug,hyperast_vcs_git=info,hyperast=error")
        .try_init()
        .unwrap();

    let repo_spec = hyperast_vcs_git::git::Forge::Github.repo("INRIA", "spoon");
    let config = hyperast_vcs_git::processing::RepoConfig::JavaMaven;
    let commit = "56e12a0c0e0e69ea70863011b4f4ca3305e0542b";
    let language = "Java";
    let tsg = r#"
(class_declaration name:(_)@name)@class {
    node @class.decl
    attr (@class.decl) name = (source-text @name)
}
"#;

    run_tsg(repo_spec, config, commit, language, tsg)
}

fn run_tsg(
    repo_spec: hyperast_vcs_git::git::Repo,
    config: hyperast_vcs_git::processing::RepoConfig,
    commit: &str,
    language: &str,
    tsg: &str,
) -> std::result::Result<(), Box<dyn std::error::Error>> {
    let state = crate::AppState::default();
    state
        .repositories
        .write()
        .unwrap()
        .register_config_with_tsg(repo_spec.clone(), config, tsg.into());
    let repo = state
        .repositories
        .read()
        .unwrap()
        .get_config(repo_spec)
        .ok_or_else(|| "missing config for repository".to_string())?;
    let mut repository = repo.fetch();
    log::debug!("done cloning {}", repository.spec);
    let commits = state.repositories.write().unwrap().pre_process_with_limit(
        &mut repository,
        "",
        &commit,
        1,
    )?;
    #[cfg(feature = "subtree-stats")]
    dbg!(
        &state
            .repositories
            .read()
            .unwrap()
            .processor
            .main_stores
            .node_store
            .inner
            .height_counts
    );
    Ok(())
}
