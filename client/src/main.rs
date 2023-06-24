#![feature(array_chunks)]
#![feature(core_intrinsics)]
#![feature(build_hasher_simple_hash_one)]
#![feature(map_many_mut)]
#![feature(iter_collect_into)]
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
};

use dashmap::DashMap;
use hyper_ast_cvs_git::{
    git::Forge, multi_preprocessed::PreProcessedRepositories, processing::ConfiguredRepoHandle,
};
use hyper_diff::{decompressed_tree_store::PersistedNode, matchers::mapping_store::VecStore};
use tower_http::cors::CorsLayer;

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
mod commit;
mod examples;
mod fetch;
mod file;
mod matching;
mod scripting;
mod track;
mod utils;
mod view;
mod ws;
mod cli;

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
pub(crate) enum MappingStage {
    Subtree,
    Bottomup,
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

    let shared_state = SharedState::default();
    {
        use hyper_ast_cvs_git::processing::RepoConfig;
        let mut repos = shared_state.repositories.write().unwrap();
        repos.register_config(Forge::Github.repo("INRIA", "spoon"), RepoConfig::JavaMaven);
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
        .merge(fetch_git_file(Arc::clone(&shared_state)))
        .merge(track_code_route(Arc::clone(&shared_state)))
        .merge(view_code_route(Arc::clone(&shared_state)))
        .merge(fetch_code_route(Arc::clone(&shared_state)))
        .merge(commit_metadata_route(Arc::clone(&shared_state)))
        .merge(example_app())
        .layer(CorsLayer::permissive()) // WARN unwanted for deployment
        .with_state(Arc::clone(&shared_state));
    // TODOs auth admin to list pending constructions,
    // all repositories are blacklised by default
    // give provider per forge
    // to whitelist repositories either for all past commits or also all future commits
    // manage users and quota
    tracing::debug!("listening on {}", opts.address);
    axum::Server::bind(&opts.address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
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
