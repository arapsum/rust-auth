use axum::http::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum ModelError {
    #[error("User with email already exists")]
    EmailTaken,
    #[error("Entity already exists")]
    EntityAlreadyExists,
    #[error("Entity not found")]
    EntityNotFound,
    #[error(transparent)]
    SqlxError(#[from] sqlx::Error),
}

pub type ModelResult<T, E = ModelError> = Result<T, E>;

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
        }
    }
}
