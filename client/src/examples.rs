use axum::{
    body::Bytes,
    // error_handling::HandleErrorLayer,
    extract::DefaultBodyLimit,
    handler::Handler,
    // response::{IntoResponse, Response},
    routing::{
        get,
        // post
    },
    Router,
};
use tower::limit::ConcurrencyLimitLayer;
use tower_http::{compression::CompressionLayer, limit::RequestBodyLimitLayer};

use crate::SharedState;

/// axum handler for "GET /" which returns a string and causes axum to
/// immediately respond with status code `200 OK` and with the string.
pub async fn hello() -> String {
    "Hello, World!".into()
}

pub(super) fn example_app() -> Router<SharedState> {
    Router::new().route("/", get(hello))
    // .route("/demo.html", get(get_demo_html))
    // .route("/hello.html", get(hello_html))
    // .route("/demo-status", get(demo_status))
    // .route("/demo-uri", get(demo_uri))
    // .route("/demo.png", get(get_demo_png))
    // .route(
    //     "/foo",
    //     get(get_foo)
    //         .put(put_foo)
    //         .patch(patch_foo)
    //         .post(post_foo)
    //         .delete(delete_foo),
    // )
    // .route("/items/:id", get(get_items_id))
    // .route("/items", get(get_items))
    // .route("/demo.json", get(get_demo_json).put(put_demo_json))
}

async fn kv_get(
    axum::extract::Path(key): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<SharedState>,
) -> Result<Bytes, hyper::StatusCode> {
    let db = &state.db;

    if let Some(value) = db.get(&key) {
        Ok(value.clone())
    } else {
        Err(hyper::StatusCode::NOT_FOUND)
    }
}

async fn kv_set(
    axum::extract::Path(key): axum::extract::Path<String>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    bytes: Bytes,
) {
    state.db.insert(key, bytes);
}

async fn list_keys(axum::extract::State(state): axum::extract::State<SharedState>) -> String {
    let db = &state.db;

    db.iter()
        .map(|key| key.key().to_string())
        .collect::<Vec<String>>()
        .join("\n")
}

pub(super) fn kv_store_app(st: SharedState) -> Router<SharedState> {
    Router::new()
        .route(
            "/:key",
            // Add compression to `kv_get`
            get(kv_get.layer(CompressionLayer::new()))
                // But don't compress `kv_set`
                .post_service(
                    kv_set
                        .layer((
                            DefaultBodyLimit::disable(),
                            RequestBodyLimitLayer::new(1024 * 5_000 /* ~5mb */),
                            ConcurrencyLimitLayer::new(1),
                        ))
                        .with_state(st),
                ),
        )
        .route("/keys", get(list_keys))
}
