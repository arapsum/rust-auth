use argon2::password_hash::Error as ArgonError;
use axum::{http::StatusCode, response::Response};

use crate::error::response::ErrorResponse;

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("User with email already exists")]
    EmailTaken,
    #[error("Entity already exists")]
    EntityAlreadyExists,
    #[error("Entity not found")]
    EntityNotFound,
    #[error("File not found")]
    FileNotFound,
    #[error("Invalid claims key")]
    InvalidClaimsKey,
    #[error("Invalid credentials provided")]
    InvalidCredentials,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Password hashing error: {0}")]
    PasswordHash(ArgonError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    SerdeSaphyr(#[from] serde_saphyr::Error),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
    #[error("Only JSON and YAML file types supported")]
    UnsupportedFileType,
}

pub type ModelResult<T, E = ModelError> = Result<T, E>;

impl From<ArgonError> for ModelError {
    fn from(err: ArgonError) -> Self {
        match err {
            ArgonError::Password => Self::InvalidCredentials,
            other => Self::PasswordHash(other),
        }
    }
}

impl ModelError {
    #[must_use]
    pub fn response_body(&self) -> (StatusCode, String) {
        match self {
            Self::EmailTaken => (
                StatusCode::CONFLICT,
                "User with email already exists".into(),
            ),
            Self::EntityAlreadyExists => (StatusCode::CONFLICT, "Entity already exists".into()),
            Self::EntityNotFound => (StatusCode::NOT_FOUND, "Entity not found".into()),
            Self::SqlxError(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal server error occurred.".into(),
            ),
            Self::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid email or password".into())
            }
            Self::InvalidClaimsKey => (StatusCode::FORBIDDEN, "Invalid claims key".into()),
            Self::PasswordHash(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal server error occurred.".into(),
            ),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal server error occurred.".into(),
            ),
        }
    }

    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::EmailTaken => "email_taken",
            Self::EntityAlreadyExists => "entity_already_exists",
            Self::EntityNotFound => "entity_not_found",
            Self::FileNotFound => "file_not_found",
            Self::InvalidClaimsKey => "invalid_claims",
            Self::InvalidCredentials => "invalid_credentials",
            Self::UnsupportedFileType => "unsupported_file_type",
            Self::IO(_)
            | Self::PasswordHash(_)
            | Self::SerdeJson(_)
            | Self::SerdeSaphyr(_)
            | Self::SqlxError(_) => "internal_error",
        }
    }

    #[must_use]
    pub fn response(&self) -> Response {
        let (status, message) = self.response_body();

        ErrorResponse::new(message, self.code()).into_response(status)
    }
}
