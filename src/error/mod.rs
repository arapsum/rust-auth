use std::fmt::{self, Display};

use jsonwebtoken::errors::{Error as JwtError, ErrorKind as JwtErrorKind};

use crate::{mailer::MailerError, repository::ModelError, validator::ValidationError};

pub mod response;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Axum(#[from] axum::Error),
    #[error(transparent)]
    Config(#[from] crate::config::ConfigError),
    #[error(transparent)]
    Controller(#[from] crate::controllers::ControllerError),
    #[error("Invalid authorisation token")]
    InvalidToken,
    #[error(transparent)]
    IO(#[from] tokio::io::Error),
    #[error(transparent)]
    JwtError(JwtError),
    #[error(transparent)]
    Mailer(#[from] MailerError),
    #[error("Missing credentials")]
    MissingCredentials,
    #[error(transparent)]
    Model(#[from] ModelError),
    #[error("Session expired")]
    SessionExpired,
    #[error(transparent)]
    Validation(#[from] ValidationError),
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
