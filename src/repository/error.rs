use argon2::password_hash::Error as ArgonError;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};

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
            Self::SqlxError(err) => {
                tracing::error!("SQLX Error {}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error has occurred".into(),
                )
            }
            Self::InvalidCredentials => {
                (StatusCode::UNAUTHORIZED, "Invalid email or password".into())
            }
            Self::InvalidClaimsKey => (StatusCode::FORBIDDEN, "Invalid claims key".into()),
            Self::PasswordHash(e) => {
                tracing::error!("Argon2 Error {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error has occurred".into(),
                )
            }
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "An internal server error has occurred".into(),
            ),
        }
    }

    #[must_use]
    pub fn response(&self) -> Response {
        let (status, message) = self.response_body();

        let body = Json(serde_json::json!({ "error": message }));

        (status, body).into_response()
    }
}
