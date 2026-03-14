#![allow(clippy::missing_errors_doc)]
use jsonwebtoken::{DecodingKey, EncodingKey};
use sqlx::PgPool;

use crate::{
    Error,
    config::{AuthConfig, Config, JwtConfig},
};

#[derive(Clone)]
pub struct AppContext {
    db: PgPool,
    auth: AuthContext,
    config: Config,
}

impl AppContext {
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    #[must_use]
    pub const fn db(&self) -> &PgPool {
        &self.db
    }

    #[must_use]
    pub const fn auth(&self) -> &AuthContext {
        &self.auth
    }
}

impl TryFrom<Config> for AppContext {
    type Error = Error;

    fn try_from(cfg: Config) -> Result<Self, Self::Error> {
        Ok(Self {
            db: cfg.database().pool()?,
            auth: cfg.auth().try_into()?,
            config: cfg,
        })
    }
}

#[derive(Clone)]
pub struct AuthContext {
    access: JwtContext,
    refresh: JwtContext,
}

impl AuthContext {
    #[must_use]
    pub const fn refresh(&self) -> &JwtContext {
        &self.refresh
    }

    #[must_use]
    pub const fn access(&self) -> &JwtContext {
        &self.access
    }
}

impl TryFrom<&AuthConfig> for AuthContext {
    type Error = Error;

    fn try_from(cfg: &AuthConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            access: cfg.access().try_into()?,
            refresh: cfg.refresh().try_into()?,
        })
    }
}

#[derive(Clone)]
pub struct JwtContext {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    maxage: i64,
}

impl JwtContext {
    #[must_use]
    pub const fn encoding_key(&self) -> &EncodingKey {
        &self.encoding_key
    }

    #[must_use]
    pub const fn decoding_key(&self) -> &DecodingKey {
        &self.decoding_key
    }

    #[must_use]
    pub const fn maxage(&self) -> i64 {
        self.maxage
    }
}

impl TryFrom<&JwtConfig> for JwtContext {
    type Error = Error;
    fn try_from(cfg: &JwtConfig) -> Result<Self, Self::Error> {
        Ok(Self {
            encoding_key: cfg.encoding_key()?,
            decoding_key: cfg.decoding_key()?,
            maxage: cfg.maxage,
        })
    }
}
