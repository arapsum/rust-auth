use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::Value;

use crate::{controllers::ControllerError, repository::ModelError, validator::ValidationError};

use super::{Error, Report};

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl ErrorResponse {
    #[must_use]
    pub fn new<T: Into<String>>(error: T, code: &'static str) -> Self {
        Self {
            error: error.into(),
            code,
            field: None,
            details: None,
        }
    }

    #[must_use]
    pub fn set_field<T: Into<String>>(mut self, field: Option<T>) -> Self {
        self.field = field.map(Into::into);
        self
    }

    #[must_use]
    pub fn set_details(mut self, details: Option<Value>) -> Self {
        self.details = details;
        self
    }
}

impl IntoResponse for Report {
    fn into_response(self) -> Response {
        let err = self.0;
        let err_string = format!("{err}");

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
    pub fn response_body(&self) -> (StatusCode, String) {
        let (status, message) = match self {
            Self::Config(_) | Self::IO(_) | Self::Axum(_) | Self::JwtError(_) | Self::Mailer(_) => {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error occurred.".to_string(),
                )
            }
            Self::Model(err) => err.response_body(),
            Self::ValidationError(err) => (StatusCode::UNPROCESSABLE_ENTITY, err.clone()),
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
            Self::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),
            Self::PathRejection(rejection) => (rejection.status(), rejection.body_text()),
        };

        (status, message)
    }

    fn field(&self) -> Option<String> {
        match self {
            Self::PathRejection(rejection) => path_parameter_name(&rejection.body_text()),
            _ => None,
        }
    }

    fn details(&self) -> Option<Value> {
        match self {
            Self::ValidationError(error) => serde_json::from_str(error)
                .ok()
                .filter(|details: &Value| details.is_object()),
            _ => None,
        }
    }

    const fn code(&self) -> &'static str {
        match self {
            Self::InvalidToken => "invalid_token",
            Self::MissingCredentials => "missing_credentials",
            Self::SessionExpired => "session_expired",
            Self::ValidationError(_) => "validation_error",
            Self::PathRejection(_) => "invalid_path_parameter",
            Self::JsonRejection(_) => "invalid_json",
            _ => "internal_error",
        }
    }

    #[must_use]
    pub fn response(&self) -> Response {
        let (status, mut message) = self.response_body();

        let details = self.details();
        if details.is_some() && matches!(self, Self::ValidationError(_)) {
            message = "One or more fields failed validation".to_string();
        }

        let mut body = ErrorResponse::new(message, self.code());
        body = body.set_field(self.field());
        body = body.set_details(self.details());

        (status, Json(body)).into_response()
    }
}

fn path_parameter_name(message: &str) -> Option<String> {
    let (_, remainder) = message.split_once("Cannot parse `")?;
    let (field, _) = remainder.split_once('`')?;

    (!field.is_empty()
        && field
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_'))
    .then(|| field.to_string())
}
