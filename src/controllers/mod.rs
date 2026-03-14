use std::sync::Arc;

use axum::{
    Json, Router,
    http::{StatusCode, Uri},
};

use crate::AppContext;

pub mod auth;

pub fn router(ctx: &Arc<AppContext>) -> Router {
    Router::new()
        .route(
            "/",
            axum::routing::get(|| async {
                axum::Json(serde_json::json!({"message": "Server is up and running!"}))
            }),
        )
        .nest("/auth", auth::router(ctx))
}

pub(crate) async fn fallback(uri: Uri) -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::NOT_FOUND,
        Json(serde_json::json!({"message": format!("Page not found {uri}")})),
    )
}
