#[derive(Debug, thiserror::Error)]
pub enum MailerError {
    #[error(transparent)]
    Address(#[from] lettre::address::AddressError),
    #[error("Input/output error")]
    IO,
    #[error("Mailer initialisation error: {0}")]
    Init(String),
    #[error(transparent)]
    Lettre(#[from] lettre::error::Error),
    #[error(transparent)]
    Render(#[from] handlebars::RenderError),
    #[error(transparent)]
    Smtp(#[from] lettre::transport::smtp::Error),
    #[error(transparent)]
    Stub(#[from] lettre::transport::stub::Error),
    #[error(transparent)]
    Template(#[from] handlebars::TemplateError),
}

pub type MailerResult<T, E = MailerError> = std::result::Result<T, E>;

impl From<&'static Self> for MailerError {
    fn from(e: &'static Self) -> Self {
        Self::Init(e.to_string())
    }
}
