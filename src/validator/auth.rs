use std::{borrow::Cow, sync::LazyLock};

use regex::{Captures, Regex};
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

pub static RE_USERNAME: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^[a-zA-Z0-9_]+$").expect("Regex initialisation failed"));

#[derive(Debug, Deserialize, Clone, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct RegisterUser<'a> {
    #[validate(email)]
    email: Cow<'a, str>,
    #[validate(custom(function = "validate_username"))]
    username: Cow<'a, str>,
    #[validate(custom(function = "validate_password"))]
    password: Cow<'a, str>,
    #[validate(must_match(other = "password"))]
    confirm_password: Cow<'a, str>,
}

impl<'a> RegisterUser<'a> {
    #[must_use]
    pub fn username(&self) -> &str {
        &self.username
    }

    #[must_use]
    pub fn email(&self) -> &str {
        &self.email
    }

    #[must_use]
    pub fn password(&self) -> &str {
        &self.password
    }

    #[must_use]
    pub fn confirm_password(&self) -> &str {
        &self.confirm_password
    }
}

#[derive(Debug, Deserialize, Clone, Serialize, Validate)]
pub struct LoginUser<'a> {
    #[validate(email)]
    email: Cow<'a, str>,

    #[validate(custom(function = "validate_password"))]
    password: Cow<'a, str>,
}

impl<'a> LoginUser<'a> {
    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn password(&self) -> &str {
        &self.password
    }
}

fn validate_password(password: &str) -> Result<(), ValidationError> {
    const MIN_LENGTH: usize = 8;
    const MAX_LENGTH: usize = 48;

    let password = password.trim();
    let length = password.len();

    let error: ValidationError;

    if password.is_empty() {
        error = ValidationError::new("empty_password");
        return Err(error.with_message(Cow::Borrowed("password is required")));
    }

    if length < MIN_LENGTH {
        error = ValidationError::new("short_password");
        return Err(error.with_message(Cow::Borrowed("password requires 8 characters")));
    } else if length > MAX_LENGTH {
        error = ValidationError::new("long_password");
        return Err(error.with_message(Cow::Borrowed("password must be under 48 characters")));
    }

    if password.contains(char::is_whitespace) {
        error = ValidationError::new("whitespace_in_password");
        return Err(error.with_message(Cow::Borrowed("password cannot have spaces")));
    }

    if password.contains(',') {
        error = ValidationError::new("commas_in_password");
        return Err(error.with_message(Cow::Borrowed("password cannot have commas")));
    }

    Ok(())
}

fn validate_username(username: &str) -> Result<(), ValidationError> {
    const MIN_LENGTH: usize = 6;
    const MAX_LENGTH: usize = 32;

    let username = username.trim();
    let length = username.len();

    let error: ValidationError;

    if username.is_empty() {
        error = ValidationError::new("empty_username");
        return Err(error.with_message(Cow::Borrowed("Username is required")));
    }

    if length < MIN_LENGTH {
        error = ValidationError::new("short_username");
        return Err(error.with_message(Cow::Borrowed("Username requires 6 letters")));
    } else if length > MAX_LENGTH {
        error = ValidationError::new("long_username");
        return Err(error.with_message(Cow::Borrowed("Username must be under 32 letters")));
    }

    RE_USERNAME.captures(username).map_or_else(
        || {
            let val_error = ValidationError::new("invalid_name");
            Err(val_error.with_message(Cow::Borrowed(
                "Only letters, numbers and underscores can be used.",
            )))
        },
        |_captures: Captures<'_>| Ok(()),
    )
}
