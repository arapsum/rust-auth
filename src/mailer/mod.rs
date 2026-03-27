#![allow(clippy::missing_errors_doc)]
mod auth;
mod error;

use std::sync::LazyLock;

use handlebars::{DirectorySourceOptions, Handlebars};
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor, Transport,
    message::MultiPart,
    transport::{smtp::authentication::Credentials, stub::StubTransport},
};
use serde::{Deserialize, Serialize};

use crate::{AppContext, config::SmtpConfig};

pub use self::{
    auth::AuthMailer,
    error::{MailerError, MailerResult},
};

pub const DEFAULT_FROM_SENDER: &str = "System <admin@system.com>";

pub static MAILER_TEMPLATES: LazyLock<MailerResult<HandlebarsTemplate>> =
    LazyLock::new(|| -> MailerResult<HandlebarsTemplate> {
        let mut handlebars = HandlebarsTemplate::init()?;

        handlebars.add_template("styles", Some("partials"))?;
        handlebars.add_template("base", Some("layouts"))?;
        handlebars.add_template("welcome", None)?;
        handlebars.add_template("forgot", None)?;

        Ok(handlebars)
    });

#[derive(Clone)]
pub struct HandlebarsTemplate {
    pub registry: Handlebars<'static>,
}

impl HandlebarsTemplate {
    pub fn init() -> MailerResult<Self> {
        let mut registry = Handlebars::new();

        registry.register_templates_directory("/templates", DirectorySourceOptions::default())?;

        Ok(Self { registry })
    }

    pub fn add_template(&mut self, name: &str, path: Option<&str>) -> MailerResult<()> {
        let path = path.map_or_else(
            || Ok(format!("./templates/{name}.hbs")),
            |mut path: &str| -> MailerResult<String> {
                if path.starts_with('/') {
                    path = path.strip_prefix('/').ok_or_else(|| MailerError::IO)?;
                }

                if path.ends_with('/') {
                    path = path.strip_prefix('/').ok_or_else(|| MailerError::IO)?;
                }

                Ok(format!("./templates/{path}/{name}.hbs"))
            },
        )?;

        self.registry.register_template_file(name, path)?;

        Ok(())
    }

    pub fn render_template(
        &self,
        template: &str,
        locals: &serde_json::Value,
    ) -> MailerResult<String> {
        self.registry.render(template, locals).map_err(Into::into)
    }
}

#[async_trait::async_trait]
pub trait Mailer {
    #[must_use]
    fn opts() -> MailerOpts {
        MailerOpts {
            from: DEFAULT_FROM_SENDER.to_string(),
            ..Default::default()
        }
    }

    fn transporter(&self, context: &AppContext) -> MailerResult<EmailSender> {
        let mailer = context.config().mailer();

        EmailSender::sender(mailer.smtp())
    }

    async fn mail(&self, email: &Email, context: &AppContext) -> MailerResult<()> {
        let opts = Self::opts();
        let mut email = email.clone();

        email.from = Some(email.from.unwrap_or_else(|| opts.from.clone()));
        email.reply_to = email.reply_to.or_else(|| opts.reply_to.clone());

        self.transporter(context)?.mail(&email).await
    }
}

#[derive(Debug, Clone, Default)]
pub struct EmailArgs {
    pub from: Option<String>,
    pub to: String,
    pub reply_to: Option<String>,
    pub locals: serde_json::Value,
    pub bcc: Option<String>,
    pub cc: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct Email {
    /// Mailbox to `From` header
    pub from: Option<String>,
    /// Mailbox to `To` header
    pub to: String,
    /// Mailbox to `ReplyTo` header
    pub reply_to: Option<String>,
    /// Subject header to message
    pub subject: String,
    /// Plain text message
    pub text: String,
    /// HTML template
    pub html: String,
    /// BCC header to message
    pub bcc: Option<String>,
    /// CC header to message
    pub cc: Option<String>,
}

#[derive(Debug, Default)]
pub struct MailerOpts {
    pub from: String,
    pub reply_to: Option<String>,
}

#[derive(Debug, Clone)]
pub enum EmailTransport {
    Smtp(AsyncSmtpTransport<Tokio1Executor>),
    Test(StubTransport),
}

#[derive(Debug, Clone)]
pub struct EmailSender {
    pub transport: EmailTransport,
}

impl EmailSender {
    /// Creates a new `EmailSender` using the SMTP transport method based on the
    /// provided SMTP configuration.
    ///
    /// # Errors
    ///
    /// This function will err if:
    /// When could not initialize SMTP transport
    pub fn sender(cfg: &SmtpConfig) -> MailerResult<Self> {
        let mut builder = if cfg.secure {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.host)?.port(cfg.port)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&cfg.host).port(cfg.port)
        };

        if let Some(auth) = cfg.auth.as_ref() {
            builder =
                builder.credentials(Credentials::new(auth.user.clone(), auth.password.clone()));
        }

        Ok(Self {
            transport: EmailTransport::Smtp(builder.build()),
        })
    }

    #[must_use]
    pub fn stub() -> Self {
        Self {
            transport: EmailTransport::Test(StubTransport::new_ok()),
        }
    }

    pub async fn mail(&self, email: &Email) -> MailerResult<()> {
        let content = MultiPart::alternative_plain_html(email.html.clone(), email.text.clone());
        let mut builder = Message::builder()
            .from(
                email
                    .from
                    .clone()
                    .unwrap_or_else(|| DEFAULT_FROM_SENDER.to_string())
                    .parse()?,
            )
            .to(email.to.parse()?);

        if let Some(bcc) = &email.bcc {
            builder = builder.bcc(bcc.parse()?);
        }

        if let Some(cc) = &email.cc {
            builder = builder.cc(cc.parse()?);
        }

        if let Some(reply_to) = &email.reply_to {
            builder = builder.reply_to(reply_to.parse()?);
        }

        let msg = builder.subject(email.subject.clone()).multipart(content)?;

        match &self.transport {
            EmailTransport::Smtp(xp) => {
                xp.send(msg).await?;
            }
            EmailTransport::Test(xp) => {
                xp.send(&msg)?;
            }
        }

        Ok(())
    }
}
