use std::time::Duration;

use axum::{
    error_handling::HandleErrorLayer,
    response::{IntoResponse, Response},
    routing::{get, post},
    BoxError, Router,
};
use tower::ServiceBuilder;
use tower_http::{trace::TraceLayer, ServiceBuilderExt};

use crate::{
    scripting::{ScriptContent, ScriptingError, ScriptingParam, self},
    SharedState,
};

impl IntoResponse for ScriptingError {
    fn into_response(self) -> Response {
        self.to_string().into_response()
    }
}

// #[axum_macros::debug_handler]
async fn scripting(
    axum::extract::Path(path): axum::extract::Path<ScriptingParam>,
    axum::extract::State(state): axum::extract::State<SharedState>,
    axum::extract::Json(script): axum::extract::Json<ScriptContent>,
) -> axum::response::Result<String> {
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
        .request_body_limit(1024 * 5_000 /* ~5mb */)
        .timeout(Duration::from_secs(10))
        .layer(TraceLayer::new_for_http());
    Router::new()
        .route(
            "/script/github/:user/:name/:commit",
            post(scripting).layer(scripting_service_config.clone()), // .with_state(Arc::clone(&shared_state)),
        )
        .route(
            "/script/gitlab/:user/:name/:commit",
            post(scripting).layer(scripting_service_config), // .with_state(Arc::clone(&shared_state)),
        )
}
