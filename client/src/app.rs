use std::time::Duration;

use axum::{
    error_handling::HandleErrorLayer,
    response::{IntoResponse, Response},
    routing::{get, post},
    BoxError, Json, Router,
};
use http::StatusCode;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;

use crate::{
    commit, fetch, file, pull_requests, querying,
    scriptingv1::{self, ScriptContent, ScriptContentDepth, ScriptingError, ScriptingParam},
    smells, track, view, SharedState,
};

impl IntoResponse for ScriptingError {
    fn into_response(self) -> Response {
        let mut resp = Json(self).into_response();
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        resp
    }
}

impl IntoResponse for querying::QueryingError {
    fn into_response(self) -> Response {
        let mut resp = Json(self).into_response();
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        resp
    }
}

    #[cfg(feature = "tsg")]
    impl IntoResponse for tsg::QueryingError {
    fn into_response(self) -> Response {
        let mut resp = Json(self).into_response();
        *resp.status_mut() = StatusCode::BAD_REQUEST;
        resp
    }
}
#[cfg(not(feature = "tsg"))]
mod tsg {
    #[derive(serde::Deserialize)]
    pub struct Param {
        user: String,
        name: String,
        commit: String,
    }
    #[derive(serde::Deserialize)]
    pub struct Content;
}


// TODO try to use the extractor pattern more, specifically for the shared state,
// I think it would help inadvertently holding resources longer than necessary,
// and maybe do more preparation stuff here, + measurments ? can it be done by a layer ?

// #[axum_macros::debug_handler]
async fn scripting(
    axum::extract::Path(path): axum::extract::Path<ScriptingParam>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Json(script): axum::extract::Json<ScriptContent>,
) -> axum::response::Result<Json<scriptingv1::ComputeResult>> {
    let r = scriptingv1::simple(script, state, path)?;
    Ok(r)
}
async fn scripting_depth(
    axum::extract::Path(path): axum::extract::Path<ScriptingParam>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Json(script): axum::extract::Json<ScriptContentDepth>,
) -> axum::response::Result<Json<scriptingv1::ComputeResults>> {
    let r = scriptingv1::simple_depth(script, state, path)?;
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
    Router::new()
        .route(
            "/script/github/:user/:name/:commit",
            post(scripting).layer(scripting_service_config.clone()),
        )
        .route(
            "/script-depth/github/:user/:name/:commit",
            post(scripting_depth).layer(scripting_service_config.clone()),
        )
        .route("/sharing-scripts/shared-db", get(crate::ws::connect_db))
        .route(
            "/sharing-scripts/shared/:session",
            get(crate::ws::connect_doc),
        )
    // .route(
    //     "/script/gitlab/:user/:name/:commit",
    //     post(scripting).layer(scripting_service_config),
    // )
}

async fn querying(
    headers: http::HeaderMap,
    axum::extract::Path(path): axum::extract::Path<querying::Param>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Json(script): axum::extract::Json<querying::Content>,
) -> axum::response::Response {
    let accept = headers
        .get(http::header::ACCEPT)
        .map_or("", |x| x.to_str().unwrap_or_default());

    let r = querying::simple(script, state, path);
    if accept.contains("csv") {
        match r {
            Ok(x) => {
                let r = x.results.into_iter().filter_map(|x| x.ok()).map(|x| {
                    format!(
                        "{},{},{}\n",
                        &x.commit[..8],
                        x.inner.result[0],
                        x.inner.compute_time
                    )
                });
                let mut r = Some("id,result0,compute_time\n".to_string())
                    .into_iter()
                    .chain(r)
                    .collect::<String>()
                    .into_response();
                r.headers_mut()
                    .insert("prepare_time", x.prepare_time.to_string().parse().unwrap());
                r.headers_mut().insert(
                    "matching_error_count",
                    x.matching_error_count.to_string().parse().unwrap(),
                );
                r
            }
            Err(err) => {
                let mut r = err.into_response();
                *r.status_mut() = http::StatusCode::BAD_REQUEST;
                r
            }
        }
    } else {
        Json(r).into_response()
    }
}

#[axum_macros::debug_handler]
async fn querying_streamed(
    axum::extract::Path(path): axum::extract::Path<querying::Param>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Json(script): axum::extract::Json<querying::Content>,
) -> axum::response::Response {
    querying::streamed(state, path, script)
}

async fn querying_differential(
    axum::extract::Path(path): axum::extract::Path<querying::ParamDifferential>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Json(script): axum::extract::Json<querying::Content>,
) -> axum::response::Result<Json<querying::ComputeResultsDifferential>> {
    let r = querying::differential(script, state, path)?;
    Ok(r)
}

pub fn querying_app(_st: SharedState) -> Router<SharedState> {
    let querying_service_config = ServiceBuilder::new()
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
    Router::new()
        .route(
            "/query/github/:user/:name/*commit",
            post(querying).layer(querying_service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/query-st/github/:user/:name/*commit",
            post(querying_streamed).layer(querying_service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/query-differential/github/:user/:name/:commit/:baseline",
            post(querying_differential).layer(querying_service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/sharing-queries/shared-db",
            get(crate::ws::connect_db), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/sharing-queries/shared/:session",
            get(crate::ws::connect_doc), // .with_state(Arc::clone(&shared_state)),
        )
}

async fn tsg(
    axum::extract::Path(path): axum::extract::Path<tsg::Param>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Json(script): axum::extract::Json<tsg::Content>,
) -> impl IntoResponse {
    #[cfg(not(feature = "tsg"))]
    {
        Result::<(),_>::Err(r#"{"error": "tsg comptime-feature is disabled on backend"}"#)
    }
    #[cfg(feature = "tsg")]
    {
        Ok(tsg::simple(script, state, path)?)
    }
}

pub fn tsg_app(_st: SharedState) -> Router<SharedState> {
    let tsg_service_config = ServiceBuilder::new()
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
    Router::new()
        .route(
            "/tsg/github/:user/:name/:commit",
            post(tsg).layer(tsg_service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/sharing-tsg/shared-db",
            get(crate::ws::connect_db), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/sharing-tsg/shared/:session",
            get(crate::ws::connect_doc), // .with_state(Arc::clone(&shared_state)),
        )
}

async fn smells(
    axum::extract::Path(path): axum::extract::Path<smells::Param>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Json(examples): axum::extract::Json<smells::Examples>,
) -> axum::response::Result<Json<smells::SearchResults>> {
    let r = smells::smells(examples, state, path)?;
    Ok(r)
}

async fn smells_ex_from_diffs(
    axum::extract::Path(path): axum::extract::Path<smells::Diffs>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<Json<smells::ExamplesResults>> {
    let r = smells::smells_ex_from_diffs(state, path)?;
    Ok(r)
}

pub fn smells_app(_st: SharedState) -> Router<SharedState> {
    let smells_service_config = ServiceBuilder::new()
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
    Router::new()
        .route(
            "/smells/github/:user/:name/:commit/:len",
            post(smells).layer(smells_service_config.clone()),
        )
        .route(
            "/smells_ex_from_diffs/github/:user/:name/:commit/:len",
            post(smells_ex_from_diffs).layer(smells_service_config.clone()),
        )
}

pub fn fetch_git_file(_st: SharedState) -> Router<SharedState> {
    let service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        .load_shed()
        .concurrency_limit(8)
        .buffer(20)
        .rate_limit(5, Duration::from_secs(1))
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
        .rate_limit(2, Duration::from_secs(2))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new()
        .route(
            "/track/github/:user/:name/:commit/*file",
            get(track_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/track_at_path/github/:user/:name/:commit/*path",
            get(track_code_at_path).layer(service_config.clone()),
        )
        .route(
            "/track_at_path_with_changes/github/:user/:name/:commit/*path",
            get(track_code_at_path_with_changes).layer(service_config.clone()),
        )
}

// #[axum_macros::debug_handler]
async fn track_code(
    axum::extract::Path(path): axum::extract::Path<track::TrackingParam>,
    axum::extract::Query(query): axum::extract::Query<track::TrackingQuery>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> impl IntoResponse {
    dbg!(&path);
    dbg!(&query);
    track::track_code(state, path, query)
}
async fn track_code_at_path(
    axum::extract::Path(path): axum::extract::Path<track::TrackingAtPathParam>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Query(query): axum::extract::Query<track::TrackingQuery>,
) -> impl IntoResponse {
    dbg!(&path);
    dbg!(&query);
    track::track_code_at_path(state, path, query)
}
async fn track_code_at_path_with_changes(
    axum::extract::Path(path): axum::extract::Path<track::TrackingAtPathParam>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Query(query): axum::extract::Query<track::TrackingQuery>,
) -> impl IntoResponse {
    dbg!(&path);
    dbg!(&query);
    track::track_code_at_path_with_changes(state, path, query)
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
    Router::new()
        .route(
            "/view/github/:user/:name/:commit/*path",
            get(view_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/view/github/:user/:name/:commit/",
            get(view_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
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

pub fn fetch_code_route(_st: SharedState) -> Router<SharedState> {
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
    Router::new()
        .route(
            "/fetch/github/:user/:name/:commit/*path",
            get(fetch_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/fetch/github/:user/:name/:commit/",
            get(fetch_code).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/fetch-ids/*ids",
            get(fetch_code_with_node_ids).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/fetch-labels/*ids",
            get(fetch_labels).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
}
// #[axum_macros::debug_handler]
async fn fetch_code(
    axum::extract::Path(path): axum::extract::Path<fetch::Parameters>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<fetch::FetchedNodes> {
    dbg!(&path);
    fetch::fetch(state, path).map_err(|err| err.into())
}
async fn fetch_code_with_node_ids(
    axum::extract::Path(ids): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<Timed<fetch::FetchedNodes>> {
    dbg!(&ids);
    fetch::fetch_with_node_ids(state, ids.split("/")).map_err(|err| err.into())
}
async fn fetch_labels(
    axum::extract::Path(ids): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<Timed<fetch::FetchedLabels>> {
    dbg!(&ids);
    fetch::fetch_labels(state, ids.split("/")).map_err(|err| err.into())
}

impl IntoResponse for fetch::FetchedLabels {
    fn into_response(self) -> Response {
        let resp = Json(self).into_response();
        resp
    }
}

impl IntoResponse for fetch::FetchedNodes {
    fn into_response(self) -> Response {
        dbg!();
        let to_string = serde_json::to_string(&self);
        dbg!();
        let var_name = to_string.unwrap();
        dbg!();
        let resp = var_name.into_response();
        // let resp = Json(self).into_response();
        dbg!();
        resp
    }
}

pub fn commit_metadata_route(_st: SharedState) -> Router<SharedState> {
    let service_config = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            dbg!(e);
        }))
        // .load_shed()
        .concurrency_limit(16)
        .buffer(64)
        .rate_limit(60, Duration::from_secs(1))
        // .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new()
        .route(
            "/commit/github/:user/:name/:version",
            get(commit_metadata).layer(service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/pr/github/:user/:name/:version",
            get(pull_requests::pr_commits).layer(service_config.clone()),
        )
        .route(
            "/fork/github/:user/:name/:other_user/:other_name/:head",
            post(add_remote).layer(service_config.clone()),
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

#[axum_macros::debug_handler]
async fn add_remote(
    axum::extract::Path(path): axum::extract::Path<commit::ParamRemote>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> axum::response::Result<()> {
    dbg!(&path);
    commit::add_remote(state, path).map_err(|err| err.into())
}
pub struct Timed<T> {
    pub(crate) time: f64,
    pub(crate) content: T,
}

impl<T: IntoResponse> IntoResponse for Timed<T> {
    fn into_response(self) -> Response {
        let mut resp = self.content.into_response();
        let headers = resp.headers_mut();
        // eg. Server-Timing: cache;desc="Cache Read";dur=23.2
        headers.insert(
            "Server-Timing",
            format!("db;desc=\"DB Read\";dur={}", self.time)
                .parse()
                .unwrap(),
        );
        resp
    }
}
