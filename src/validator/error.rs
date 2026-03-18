#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("{0}")]
    FieldError(String),
}

pub type ValidationResult<T, E = ValidationError> = Result<T, E>;
