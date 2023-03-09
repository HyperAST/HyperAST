use std::time::Duration;

use axum::{
    error_handling::HandleErrorLayer,
    response::{IntoResponse, Response},
    routing::{get, post},
    BoxError, Json, Router,
};
use http::StatusCode;
use tower::ServiceBuilder;
use tower_http::{limit::RequestBodyLimitLayer, trace::TraceLayer};

use crate::{
    file,
    scripting::{self, ComputeResult, ScriptContent, ScriptingError, ScriptingParam},
    SharedState, track, view, commit,
};

impl IntoResponse for ScriptingError {
    fn into_response(self) -> Response {
        let mut resp = Json(self).into_response();
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        resp
    }
}

// #[axum_macros::debug_handler]
async fn scripting(
    axum::extract::Path(path): axum::extract::Path<ScriptingParam>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Json(script): axum::extract::Json<ScriptContent>,
) -> axum::response::Result<Json<ComputeResult>> {
    let r = scripting::simple(script, state, path)?;
    Ok(r)
}

pub fn scripting_app(_st: SharedState) -> Router<SharedState> {
    let scripting_service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(16)
        .buffer(200)
        .rate_limit(10, Duration::from_secs(5))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new().route(
        "/script/github/:user/:name/:commit",
        post(scripting).layer(scripting_service_config.clone()), // .with_state(Arc::clone(&shared_state)),
    )
    // .route(
    //     "/script/gitlab/:user/:name/:commit",
    //     post(scripting).layer(scripting_service_config), // .with_state(Arc::clone(&shared_state)),
    // )
}

pub fn fetch_git_file(_st: SharedState) -> Router<SharedState> {
    let service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(8)
        .buffer(20)
        .rate_limit(2, Duration::from_secs(5))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new().route(
        "/file/github/:user/:name/:commit/*file",
        get(file).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
    )
}

// #[axum_macros::debug_handler]
async fn file(
    axum::extract::Path(path): axum::extract::Path<file::FetchFileParam>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<String> {
    dbg!(&path);
    file::from_hyper_ast(state, path).map_err(|err| err.into())
}

pub fn track_code_route(_st: SharedState) -> Router<SharedState> {
    let service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(8)
        .buffer(20)
        .rate_limit(2, Duration::from_secs(5))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new().route(
        "/track/github/:user/:name/:commit/*file",
        get(track_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
    )
}

// #[axum_macros::debug_handler]
async fn track_code(
    axum::extract::Path(path): axum::extract::Path<track::TrackingParam>,
    axum::extract::Query(query): axum::extract::Query<track::TrackingQuery>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<Json<track::TrackingResult>> {
    dbg!(&path);
    dbg!(&query);
    track::track_code(state, path, query).map_err(|err| err.into())
}


pub fn view_code_route(_st: SharedState) -> Router<SharedState> {
    let service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(8)
        .buffer(20)
        .rate_limit(2, Duration::from_secs(5))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new().route(
        "/view/github/:user/:name/:commit/*path",
        get(view_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
    ).route(
        "/view/github/:user/:name/:commit/",
        get(view_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
    ).route(
        "/view/:id",
        get(view_code_with_node_id).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
    )
}

// #[axum_macros::debug_handler]
async fn view_code(
    axum::extract::Path(path): axum::extract::Path<view::Parameters>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<Json<view::ViewRes>> {
    dbg!(&path);
    view::view(state, path).map_err(|err| err.into())
}
async fn view_code_with_node_id(
    axum::extract::Path(id): axum::extract::Path<u64>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<Json<view::ViewRes>> {
    view::view_with_node_id(state, id).map_err(|err| err.into())
}



pub fn commit_metadata_route(_st: SharedState) -> Router<SharedState> {
    let service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(8)
        .buffer(20)
        .rate_limit(2, Duration::from_secs(5))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new().route(
        "/commit/github/:user/:name/:version",
        get(commit_metadata).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
    )
}

#[axum_macros::debug_handler]
async fn commit_metadata(
    axum::extract::Path(path): axum::extract::Path<commit::Param>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<Json<commit::Metadata>> {
    dbg!(&path);
    commit::commit_metadata(state, path).map_err(|err| err.into())
}