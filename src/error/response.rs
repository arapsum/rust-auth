use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use super::{Error, Report};

impl IntoResponse for Report {
    fn into_response(self) -> Response {
        let err = self.0;
        let err_string = format!("{}", err);

        tracing::error!("Error: {}", err_string);

        if let Some(error) = err.downcast_ref::<Error>() {
            return error.response();
        }

        let body = Json(serde_json::json!({
            "message": "An internal server error occurred."
        }));

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl Error {
    pub fn response(&self) -> Response {
        tracing::error!("{}", self);

        let (status, message) = match self {
            Self::Config(_) | Self::IO(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal server error occurred.",
            ),
        };

        let body = Json(serde_json::json!({
            "message": message
        }));

        (status, body).into_response()
    }
}
