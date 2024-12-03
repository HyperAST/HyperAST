#![feature(array_chunks)]
#![feature(map_many_mut)]
#![feature(iter_collect_into)]
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use dashmap::DashMap;
use hyper_ast_cvs_git::multi_preprocessed::PreProcessedRepositories;
use hyper_diff::{decompressed_tree_store::PersistedNode, matchers::mapping_store::VecStore};

use axum::body::Bytes;
use hyper_ast::store::nodes::legion::NodeIdentifier;

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
mod smells;
pub mod track;
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

pub(crate) type PartialDecompCache = DashMap<NodeIdentifier, DS<PersistedNode<NodeIdentifier>>>;
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
pub type PersistableMappings<I> =
    hyper_diff::matchers::Mapping<DS<PersistedNode<I>>, DS<PersistedNode<I>>, VecStore<u32>>;
pub(crate) type MappingCache =
    DashMap<(NodeIdentifier, NodeIdentifier), PersistableMappings<NodeIdentifier>>;
type SharedState = Arc<AppState>;

pub(crate) use hyper_ast_cvs_git::no_space;

#[cfg(feature = "rerun")]
pub mod log_languages {
    //! Log the grammar of corresponding language to rerun.

    use core::f32;
    use std::ops::Mul;

    use rerun::external::{arrow2, re_types};
    pub fn log_languages() -> Result<(), Box<dyn std::error::Error>> {
        let rec = rerun::RecordingStream::global(rerun::StoreKind::Recording).unwrap();

        let lang = polyglote::Lang {
            language: hyper_ast_gen_ts_java::language(),
            name: "java",
            node_types: hyper_ast_gen_ts_java::node_types(),
            highlights: "",
            tags: "",
            injects: "",
        };
        let types = polyglote::preprocess_aux(&lang)?;

        eprintln!("{}", types);

        log_language(&rec, lang, types)?;

        let lang = polyglote::Lang {
            language: hyper_ast_gen_ts_cpp::language(),
            name: "cpp",
            node_types: hyper_ast_gen_ts_cpp::node_types(),
            highlights: "",
            tags: "",
            injects: "",
        };
        let types = polyglote::preprocess_aux(&lang)?;

        eprintln!("{}", types);

        log_language(&rec, lang, types)?;

        let lang = polyglote::Lang {
            language: hyper_ast_gen_ts_tsquery::language(),
            name: "tsquery",
            node_types: hyper_ast_gen_ts_tsquery::node_types(),
            highlights: "",
            tags: "",
            injects: "",
        };
        let types = polyglote::preprocess_aux(&lang)?;

        eprintln!("{}", types);

        log_language(&rec, lang, types)?;

        Ok(())
    }

    fn log_language(
        rec: &rerun::RecordingStream,
        lang: polyglote::Lang,
        types: polyglote::preprocess::TypeSys,
    ) -> rerun::RecordingStreamResult<()> {
        let mut map = Map {
            map: std::collections::HashMap::default(),
            rec: rec,
            name: lang.name,
        };
        let leafs: (Vec<_>, Vec<_>) = types.leafs().fold((vec![], vec![]), |mut acc, x| {
            if x.1 { &mut acc.0 } else { &mut acc.1 }.push(x.0);
            acc
        });
        map.log(leafs.1, "leaf", "unnamed", 0.0)?;
        map.log(leafs.0, "leaf", "named", 5.0)?;
        let concrete: (Vec<_>, Vec<_>) = types.concrete().fold((vec![], vec![]), |mut acc, x| {
            if x.1 { &mut acc.0 } else { &mut acc.1 }.push(x.0);
            acc
        });
        map.log(concrete.0, "concrete", "with_fields", 10.0)?;
        map.log(concrete.1, "concrete", "named", 15.0)?;
        let r#abstract = types.r#abstract().collect();
        map.log(r#abstract, "abstract", "", 20.0)?;
        let r#abstract = types.r#abstract_subtypes();
        rec.log(
            format!("language/{}/subtyping", lang.name),
            &map.arrows(r#abstract),
        )?;
        let concrete_children = types.concrete_children();
        rec.log(
            format!("language/{}/children", lang.name),
            &map.arrows(concrete_children),
        )?;
        let concrete_fields: Vec<_> = types.concrete_fields().collect();
        rec.log(
            format!("language/{}/fields", lang.name),
            &map.arrows(
                concrete_fields
                    .iter()
                    .map(|(x, v)| (x.clone(), v.iter().map(|x| x.1.clone()).collect::<Vec<_>>())),
            )
            .with_labels(
                concrete_fields
                    .iter()
                    .map(|x| x.1.iter().map(|x| x.0.clone()))
                    .flatten(),
            ),
        )?;
        Ok(())
    }
    struct Map<'a, 'b> {
        map: std::collections::HashMap<String, [f32; 3]>,
        rec: &'a rerun::RecordingStream,
        name: &'b str,
    }
    impl<'a, 'b> Map<'a, 'b> {
        fn points(&mut self, v: Vec<String>, n: impl AsRef<str>, o: f32) -> CustomPoints3D {
            CustomPoints3D {
                log: rerun::TextLog::new(n.as_ref()),
                cat: n.as_ref().into(),
                points3d: rerun::Points3D::new((0..v.len()).map(|i| {
                    let rad = (i as f32 / v.len() as f32).mul(f32::consts::TAU);
                    let l = (v.len() as f32) / f32::consts::TAU;
                    let p = [rad.sin() * l, o, rad.cos() * l];
                    self.map.insert(v[i].clone(), p.clone());
                    p
                }))
                .with_labels(v.into_iter().map(|x| x)),
            }
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
            let nn = format!("{adj} {t}");
            let path = ["language", self.name, t, adj].map(|x| x.into());
            let path = if adj.is_empty() {
                &path[..path.len() - 1]
            } else {
                &path[..]
            };
            self.rec.log(path, &self.points(v, nn, o))
        }
    }

    struct CustomPoints3D {
        points3d: rerun::Points3D,
        log: rerun::TextLog,
        cat: Cat,
    }

    impl rerun::AsComponents for CustomPoints3D {
        fn as_component_batches(&self) -> Vec<rerun::MaybeOwnedComponentBatch<'_>> {
            let indicator = rerun::NamedIndicatorComponent("user.CustomPoints3DIndicator".into());
            self.log
                .as_component_batches()
                .into_iter()
                .chain(self.points3d.as_component_batches().into_iter())
                .chain(
                    [
                        Some(indicator.to_batch()),
                        Some((&self.cat as &dyn rerun::ComponentBatch).into()),
                    ]
                    .into_iter()
                    .flatten(),
                )
                .collect()
        }
    }

    #[derive(Debug, Clone)]
    struct Cat(rerun::Text);

    impl From<String> for Cat {
        fn from(v: String) -> Self {
            Self(rerun::Text(v.into()))
        }
    }
    impl From<&str> for Cat {
        fn from(v: &str) -> Self {
            v.to_string().into()
        }
    }

    impl rerun::SizeBytes for Cat {
        #[inline]
        fn heap_size_bytes(&self) -> u64 {
            0
        }
    }

    impl rerun::Loggable for Cat {
        type Name = rerun::ComponentName;

        #[inline]
        fn name() -> Self::Name {
            "lang.Cat".into()
        }

        #[inline]
        fn arrow_datatype() -> arrow2::datatypes::DataType {
            rerun::Text::arrow_datatype()
        }

        #[inline]
        fn to_arrow_opt<'a>(
            data: impl IntoIterator<Item = Option<impl Into<std::borrow::Cow<'a, Self>>>>,
        ) -> re_types::SerializationResult<Box<dyn arrow2::array::Array>>
        where
            Self: 'a,
        {
            rerun::Text::to_arrow_opt(
                data.into_iter()
                    .map(|opt| opt.map(Into::into).map(|c| c.as_ref().0.clone())),
            )
        }
    }
}

#[ignore] // ignore (from normal cargo test) for now, later make a feature
#[test]
// slow test, more of an integration test, try using release
fn test_measuring_size() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("client=debug,hyper_ast_cvs_git=info,hyper_ast=error")
        .try_init()
        .unwrap();

    let repo_spec = hyper_ast_cvs_git::git::Forge::Github.repo("graphhopper", "graphhopper");
    let config = hyper_ast_cvs_git::processing::RepoConfig::JavaMaven;
    let commit = "f5f2b7765e6b392c5e8c7855986153af82cc1abe";
    let language = "Java";
    let prepro = hyper_ast::scripting::lua_scripting::PREPRO_SIZE_WITH_FINISH.into();
    run_scripting(repo_spec, config, commit, language, prepro, "size")
}

#[ignore] // ignore (from normal cargo test) for now, later make a feature
#[test]
// slow test, more of an integration test, try using release
fn test_measuring_mcc() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("client=debug,hyper_ast_cvs_git=info,hyper_ast=error")
        .try_init()
        .unwrap();

    let repo_spec = hyper_ast_cvs_git::git::Forge::Github.repo("graphhopper", "graphhopper");
    let config = hyper_ast_cvs_git::processing::RepoConfig::JavaMaven;
    let commit = "f5f2b7765e6b392c5e8c7855986153af82cc1abe";
    let language = "Java";
    let prepro = hyper_ast::scripting::lua_scripting::PREPRO_MCC_WITH_FINISH.into();
    run_scripting(repo_spec, config, commit, language, prepro, "mcc")
}

#[ignore] // ignore (from normal cargo test) for now, later make a feature
#[test]
// slow test, more of an integration test, try using release
fn test_measuring_loc() -> std::result::Result<(), Box<dyn std::error::Error>> {
    let _ = tracing_subscriber::fmt()
        .with_env_filter("client=debug,hyper_ast_cvs_git=info,hyper_ast=error")
        .try_init()
        .unwrap();

    let repo_spec = hyper_ast_cvs_git::git::Forge::Github.repo("graphhopper", "graphhopper");
    let config = hyper_ast_cvs_git::processing::RepoConfig::JavaMaven;
    let commit = "f5f2b7765e6b392c5e8c7855986153af82cc1abe";
    let language = "Java";
    let prepro = hyper_ast::scripting::lua_scripting::PREPRO_LOC.into();
    run_scripting(repo_spec, config, commit, language, prepro, "LoC")
}

fn run_scripting(
    repo_spec: hyper_ast_cvs_git::git::Repo,
    config: hyper_ast_cvs_git::processing::RepoConfig,
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
            .get_component::<hyper_ast::scripting::lua_scripting::DerivedData>()
            .unwrap();
        let s = dd.0.get("show");
        log::debug!("{show} ! {:?}", s);
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
    let store = &state.repositories.read().unwrap().processor.main_stores;
    let n = store.node_store.resolve(commit.ast_root);
    let dd = n
        .get_component::<hyper_ast::scripting::lua_scripting::DerivedData>()
        .unwrap();
    let s = dd.0.get(show);
    log::debug!("{:?}", s);

    Ok(())
}
