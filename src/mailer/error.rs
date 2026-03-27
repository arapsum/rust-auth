#[derive(Debug, thiserror::Error)]
pub enum MailerError {
    #[error(transparent)]
    Address(#[from] lettre::address::AddressError),
    #[error("Input/output error")]
    IO,
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

pub type MailerResult<T> = std::result::Result<T, MailerError>;
