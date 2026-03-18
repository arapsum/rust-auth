use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};

#[derive(Debug, thiserror::Error)]
pub enum ControllerError {
    #[error(transparent)]
    JsonRejection(#[from] JsonRejection),
}

pub type ControllerResult<T> = Result<T, ControllerError>;

impl ControllerError {
    #[must_use]
    pub fn response_body(&self) -> (StatusCode, String) {
        match self {
            Self::JsonRejection(err) => (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()),
        }
    }

    #[must_use]
    pub fn response(&self) -> Response {
        let (status, message) = self.response_body();

        let body = Json(serde_json::json!({ "error": message }));

        (status, body).into_response()
    }
}
