#![feature(array_chunks)]
#![feature(map_many_mut)]
#![feature(iter_collect_into)]
use std::{
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use app::{querying_app, smells_app, tsg_app};
use dashmap::DashMap;
use hyper_ast_cvs_git::{git::Forge, multi_preprocessed::PreProcessedRepositories};
use hyper_diff::{decompressed_tree_store::PersistedNode, matchers::mapping_store::VecStore};
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{
    app::{
        commit_metadata_route, fetch_code_route, fetch_git_file, scripting_app, track_code_route,
        view_code_route,
    },
    examples::{example_app, kv_store_app},
};
use axum::{body::Bytes, Router};
use hyper_ast::store::nodes::legion::NodeIdentifier;

mod app;
mod changes;
mod cli;
mod commit;
mod examples;
mod fetch;
mod file;
mod matching;
mod pull_requests;
mod querying;
mod scripting;
mod smells;
mod track;
mod tsg;
mod utils;
mod view;
mod ws;

// #[derive(Default)]
pub struct AppState {
    db: DashMap<String, Bytes>,
    repositories: RwLock<PreProcessedRepositories>,
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

// #[derive(Default)]
// struct RepoConfigs(HashMap<hyper_ast_cvs_git::git::Repo, hyper_ast_cvs_git::processing::RepoConfig2>);
// impl RepoConfigs {
//     pub(crate) fn resolve(&self, specifier: hyper_ast_cvs_git::git::Repo) -> Option<ConfiguredRepoHandle> {
//         let config = self.0
//             .get(&specifier)?;
//         Some(ConfiguredRepoHandle {
//             spec: specifier,
//             config: *config,
//         })
//     }
// }

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

#[tokio::main]
async fn main() {
    let opts = crate::cli::parse();
    #[cfg(feature = "rerun")]
    {
        if let Err(e) = log_languages::log_languages() {
            log::error!("error logging languages: {}", e)
        };
    }
    let shared_state = SharedState::default();
    {
        use hyper_ast_cvs_git::processing::RepoConfig;
        let mut repos = shared_state.repositories.write().unwrap();
        repos.register_config(Forge::Github.repo("INRIA", "spoon"), RepoConfig::JavaMaven);
        repos.register_config(Forge::Github.repo("google", "gson"), RepoConfig::JavaMaven);
        repos.register_config(
            Forge::Github.repo("Marcono1234", "gson"),
            RepoConfig::JavaMaven,
        );
        repos.register_config(
            Forge::Github.repo("official-stockfish", "Stockfish"),
            RepoConfig::CppMake,
        );
        repos.register_config(Forge::Github.repo("torvalds", "linux"), RepoConfig::CppMake);
        opts.repository.iter().for_each(|x| {
            repos.register_config(x.repo.clone(), x.config);
        })
    }
    let app = Router::new()
        .fallback(fallback)
        .route("/ws", axum::routing::get(ws::ws_handler))
        .merge(kv_store_app(Arc::clone(&shared_state)))
        .merge(scripting_app(Arc::clone(&shared_state)))
        .merge(querying_app(Arc::clone(&shared_state)))
        .merge(tsg_app(Arc::clone(&shared_state)))
        .merge(smells_app(Arc::clone(&shared_state)))
        .merge(fetch_git_file(Arc::clone(&shared_state)))
        .merge(track_code_route(Arc::clone(&shared_state)))
        .merge(view_code_route(Arc::clone(&shared_state)))
        .merge(fetch_code_route(Arc::clone(&shared_state)))
        .merge(commit_metadata_route(Arc::clone(&shared_state)))
        .merge(example_app())
        .layer(CorsLayer::permissive()) // WARN unwanted for deployment
        .layer(TraceLayer::new_for_http())
        .with_state(Arc::clone(&shared_state));
    // TODOs auth admin to list pending constructions,
    // all repositories are blacklised by default
    // give provider per forge
    // to whitelist repositories either for all past commits or also all future commits
    // manage users and quota
    tracing::debug!("listening on {}", opts.address);
    let listener = tokio::net::TcpListener::bind(&opts.address).await.unwrap();
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .unwrap();
}
pub(crate) use hyper_ast_cvs_git::no_space;
/// axum handler for any request that fails to match the router routes.
/// This implementation returns HTTP status code Not Found (404).
pub async fn fallback(uri: axum::http::Uri) -> impl axum::response::IntoResponse {
    (
        axum::http::StatusCode::NOT_FOUND,
        format!("No route {}", uri),
    )
}

/// Tokio signal handler that will wait for a user to press CTRL+C.
/// We use this in our hyper `Server` method `with_graceful_shutdown`.
async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("expect tokio signal ctrl-c");
    println!("signal shutdown");
}

// pub(crate) use hyper_ast::store::nodes::no_space;
// #[test]
// fn test_scripting() -> Result<(), Box<dyn std::error::Error>> {
//     let client = reqwest::blocking::Client::default();
//     let req_build = client.post(
//         "http://localhost:8080/script/github/INRIA/spoon/4acedc53a13a727be3640fe234f7e261d2609d58",
//     );
//     use crate::scripting::ScriptContent;

//     let script = ScriptContent {
//         init: r##"#{depth:0, files: 0, type_decl: 0}"##.to_string(),
//         filter: r##"
// if is_directory() {
//     children().map(|x| {[x, #{depth: s.depth + 1, files: s.files, type_decl: s.type_decl}]})
// } else if is_file() {
//     children().map(|x| {[x, #{depth: s.depth + 1, type_decl: s.type_decl}]})
// } else {
//     []
// }"##
//         .to_string(),
//         accumulate: r##"
// if is_directory() {
//     p.files += s.files;
//     p.type_decl += s.type_decl;
// } else if is_file() {
//     p.files += 1;
//     p.type_decl += s.type_decl;
// } else if is_type_decl() {
//     p.type_decl += 1;
// }"##
//         .to_string(),
//     };

//     let req = req_build
//         .timeout(Duration::from_secs(60 * 60))
//         .header("content-type", "application/json")
//         .body(serde_json::to_string(&script).unwrap())
//         .build()?;
//     let resp = client.execute(req)?;
//     println!("{:#?}", resp.text()?);
//     Ok(())
// }

static CASE_BIG1: &'static str = r#"class A{class C{}class B{{while(1){if(1){}else{}};}}}class D{class E{}class F{{while(2){if(2){}else{}};}}}"#;

static CASE_BIG2: &'static str = r#"class A{class C{}}class B{{while(1){if(1){}else{}};}}class D{class E{}}class F{{while(2){if(2){}else{}};}}"#;

#[cfg(feature = "rerun")]
mod log_languages {
    //! Log the grammar of corresponding language to rerun.

    use core::f32;
    use std::ops::Mul;

    use rerun::external::{arrow2, re_types};
    pub(super) fn log_languages() -> Result<(), Box<dyn std::error::Error>> {
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
