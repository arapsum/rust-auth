use std::sync::Arc;

use axum::{
    Json, Router,
    http::{StatusCode, Uri},
    response::{IntoResponse, Response},
};

use crate::{AppContext, error::response::ErrorResponse};

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

pub(crate) async fn fallback(_uri: Uri) -> Response {
    (
        StatusCode::NOT_FOUND,
        Json(ErrorResponse::new("Route not found", "route_not_found")),
    )
        .into_response()
}
