use std::fmt::{self, Display};

use jsonwebtoken::errors::{Error as JwtError, ErrorKind as JwtErrorKind};

pub mod response;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),
    #[error("Invalid authorisation token")]
    InvalidToken,
    #[error(transparent)]
    JsonRejection(#[from] axum::extract::rejection::JsonRejection),
    #[error(transparent)]
    JwtError(JwtError),
    #[error("Missing credentials")]
    MissingCredentials,
    #[error(transparent)]
    PathRejection(#[from] axum::extract::rejection::PathRejection),
    #[error("Session expired")]
    SessionExpired,
    #[error("Validation error: {0}")]
    ValidationError(String),
}

#[derive(Debug)]
pub struct Report(pub color_eyre::Report);

impl<E> From<E> for Report
where
    E: Into<color_eyre::Report>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

impl Display for Report {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

pub type Result<T, E = Report> = std::result::Result<T, E>;

impl From<JwtError> for Error {
    fn from(err: JwtError) -> Self {
        match err.kind() {
            JwtErrorKind::ExpiredSignature => Self::SessionExpired,
            JwtErrorKind::InvalidToken => Self::InvalidToken,
            _ => Self::JwtError(err),
        }
    }
}
