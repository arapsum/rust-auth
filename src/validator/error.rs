use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("{0}")]
    FieldError(String),
}

pub type ValidationResult<T, E = ValidationError> = Result<T, E>;

impl ValidationError {
    #[must_use]
    pub fn response(&self) -> Response {
        let (status, message) = match self {
            Self::FieldError(error) => (StatusCode::UNPROCESSABLE_ENTITY, error.clone()),
        };

        let body = Json(serde_json::json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
