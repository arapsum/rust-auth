#![allow(clippy::missing_errors_doc)]

use serde_json::json;

use crate::{AppContext, Result, repository::UserModel};

use super::{Email, HandlebarsTemplate, MAILER_TEMPLATES, Mailer};

pub struct AuthMailer {
    renderer: HandlebarsTemplate,
}

impl Mailer for AuthMailer {}

impl AuthMailer {
    pub fn init() -> Result<Self> {
        let renderer = MAILER_TEMPLATES.as_ref()?;

        Ok(Self {
            renderer: renderer.clone(),
        })
    }

    /// Function to send a welcoming email to new users.
    ///
    /// # Errors
    /// * SMTP Errors
    /// * Rendering Errors
    pub async fn send_welcome(ctx: &AppContext, user: &UserModel) -> Result<()> {
        let this = Self::init()?;

        let rendered = this.renderer.render_template(
            "welcome",
            &json!({
                "name": &user.username,
                "url": format!("{}/verify/{}", ctx.config().server().url(), &user.verification_token),
                "subject": "Welcome"
            }),
        )?;

        let email = Email {
            to: user.email.as_str().to_string(),
            subject: "Welcome to Kodiak".to_string(),
            text: rendered,
            html: "welcome.hbs".to_string(),
            ..Default::default()
        };

        this.mail(&email, ctx).await?;

        Ok(())
    }

    /// Function to send a reset link to users
    /// who have forgotten their password.
    ///
    /// # Errors
    /// * SMTP Errors
    /// * Rendering Errors
    ///
    /// # Panics
    /// * This function will panic if the reset token is not set
    pub async fn forgot_password(ctx: &AppContext, user: &UserModel) -> Result<()> {
        let this = Self::init()?;

        let rendered = this.renderer.render_template(
            "forgot",
            &json!({
                "name": &user.username,
                "url": format!("{}/reset-password/{}", ctx.config().server().url(), user.reset_token.as_ref().unwrap()),
                "subject": "Forgot Password?"
            }),
        )?;

        let email = Email {
            to: user.email.as_str().to_string(),
            subject: "Forgot Your Password?".to_string(),
            text: rendered,
            html: "forgot.hbs".to_string(),
            ..Default::default()
        };

        this.mail(&email, ctx).await?;

        Ok(())
    }
}
