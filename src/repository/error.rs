use argon2::password_hash::Error as ArgonError;
use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("User with email already exists")]
    EmailTaken,
    #[error("Entity already exists")]
    EntityAlreadyExists,
    #[error("Entity not found")]
    EntityNotFound,
    #[error("Invalid credentials provided")]
    InvalidCredentials,
    #[error("Password hashing error: {0}")]
    PasswordHash(ArgonError),
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
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
            Self::PasswordHash(e) => {
                tracing::error!("Argon2 Error {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "An internal server error has occurred".into(),
                )
            }
        }
    }
}
