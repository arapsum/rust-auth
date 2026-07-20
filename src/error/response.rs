use axum::{
    Json,
    extract::rejection::JsonRejection,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use serde_json::Value;

use crate::repository::ModelError;

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

    #[must_use]
    pub fn into_response(self, status: StatusCode) -> Response {
        (status, Json(self)).into_response()
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
        }

        ErrorResponse::new("An internal server error occurred.", "internal_error")
            .into_response(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl Error {
    #[must_use]
    pub fn response_body(&self) -> (StatusCode, String) {
        let (status, message) = match self {
            Self::Config(_) | Self::JwtError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal server error occurred.".to_string(),
            ),
            Self::ValidationError(err) => (StatusCode::UNPROCESSABLE_ENTITY, err.clone()),
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
            Self::JsonRejection(rejection) => json_rejection_response(rejection),
            Self::PathRejection(rejection) => {
                (rejection.status(), "Invalid path parameter".to_string())
            }
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
            Self::JsonRejection(rejection) => json_rejection_code(rejection),
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

        ErrorResponse::new(message, self.code())
            .set_field(self.field())
            .set_details(self.details())
            .into_response(status)
    }
}

fn json_rejection_response(rejection: &JsonRejection) -> (StatusCode, String) {
    match rejection {
        JsonRejection::JsonDataError(_) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            "Request JSON does not match the expected schema".to_string(),
        ),
        JsonRejection::JsonSyntaxError(_) => (
            StatusCode::BAD_REQUEST,
            "Request body contains malformed JSON".to_string(),
        ),
        JsonRejection::MissingJsonContentType(_) => (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Expected a JSON request body".to_string(),
        ),
        JsonRejection::BytesRejection(_) => (
            StatusCode::BAD_REQUEST,
            "Unable to read request body".to_string(),
        ),
        _ => (
            rejection.status(),
            "Unable to process JSON request body".to_string(),
        ),
    }
}

const fn json_rejection_code(rejection: &JsonRejection) -> &'static str {
    match rejection {
        JsonRejection::MissingJsonContentType(_) => "invalid_content_type",
        JsonRejection::BytesRejection(_) => "invalid_request_body",
        _ => "invalid_json",
    }
}

fn path_parameter_name(message: &str) -> Option<String> {
    let (_, remainder) = message
        .split_once("Cannot parse `")
        .or_else(|| message.split_once("Invalid UTF-8 in `"))?;
    let (field, _) = remainder.split_once('`')?;

    (!field.is_empty()
        && field
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || character == '_'))
    .then(|| field.to_string())
}
