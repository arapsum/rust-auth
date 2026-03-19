use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::{controllers::ControllerError, repository::ModelError, validator::ValidationError};

use super::{Error, Report};

impl IntoResponse for Report {
    fn into_response(self) -> Response {
        let err = self.0;
        let err_string = format!("{err}",);

        tracing::error!("Error: {}", err_string);

        if let Some(error) = err.downcast_ref::<Error>() {
            return error.response();
        } else if let Some(error) = err.downcast_ref::<ModelError>() {
            return error.response();
        } else if let Some(error) = err.downcast_ref::<ControllerError>() {
            return error.response();
        } else if let Some(error) = err.downcast_ref::<ValidationError>() {
            return error.response();
        }

        let body = Json(serde_json::json!({
            "error": "An internal server error occurred."
        }));

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl Error {
    #[must_use]
    pub fn response(&self) -> Response {
        let (status, message) = match self {
            Self::Config(_) | Self::IO(_) | Self::Axum(_) | Self::JwtError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal server error occurred.".to_string(),
            ),
            Self::Model(err) => err.response_body(),
            Self::Validation(err) => (StatusCode::UNPROCESSABLE_ENTITY, err.to_string()),
            Self::Controller(err) => err.response_body(),
            Self::InvalidToken => (
                StatusCode::UNAUTHORIZED,
                "Invalid authorisation token".to_string(),
            ),
            Self::MissingCredentials => (
                StatusCode::UNAUTHORIZED,
                "Login credentials are required".to_string(),
            ),
            Self::SessionExpired => (
                StatusCode::UNAUTHORIZED,
                "Session expired. Please log in again.".to_string(),
            ),
        };

        let body = Json(serde_json::json!({
            "error": message
        }));

        (status, body).into_response()
    }
}
