use std::sync::Arc;

use axum::{
    Json, Router, debug_handler,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
};

use crate::AppContext;

#[debug_handler]
async fn register(State(ctx): State<Arc<AppContext>>) -> Response {
    (
        StatusCode::OK,
        Json(serde_json::json!({"message": format!("Endpoint: {}", ctx.config().server().url())})),
    )
        .into_response()
}

pub fn router(ctx: &Arc<AppContext>) -> Router {
    Router::new()
        .route("/sign-up", post(register))
        .with_state(ctx.clone())
}
