// #![feature(array_chunks)]
// #![feature(map_many_mut)]
// #![feature(iter_collect_into)]
use std::{net::SocketAddr, sync::Arc};

use backend::*;

use axum::Router;
use backend::{
    app::{
        commit_metadata_route, fetch_code_route, fetch_git_file, querying_app, scripting_app,
        smells_app, track_code_route, tsg_app, view_code_route,
    },
    examples::{example_app, kv_store_app},
};
use hyper_diff::matchers::mapping_store::VecStore;
use hyperast_vcs_git::git::Forge;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[cfg(not(target_env = "msvc"))]
use jemallocator::Jemalloc;

#[cfg(not(target_env = "msvc"))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

type DS<T> = hyper_diff::decompressed_tree_store::lazy_post_order::LazyPostOrder<T, u32>;
pub type PersistableMappings<I> = hyper_diff::matchers::Mapping<DS<I>, DS<I>, VecStore<u32>>;
type SharedState = Arc<AppState>;

#[tokio::main]
async fn main() {
    let opts = backend::cli::parse();
    #[cfg(feature = "rerun")]
    {
        if let Err(e) = backend::log_languages::log_languages() {
            log::error!("error logging languages: {}", e)
        };
    }
    let shared_state = SharedState::default();
    {
        use hyperast_vcs_git::processing::RepoConfig;
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
        repos.register_config(
            Forge::Github.repo("systemd", "systemd"),
            RepoConfig::CppMake,
        );
        opts.repository.iter().for_each(|x| {
            repos.register_config(x.repo.clone(), x.config);
        })
    }
    let app = Router::new()
        .fallback(fallback)
        .route("/ws", axum::routing::get(backend::ws_handler))
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
