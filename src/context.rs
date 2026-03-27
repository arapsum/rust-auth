#![allow(clippy::missing_errors_doc)]
use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation};
use redis::Client;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    Error,
    config::{AuthConfig, Config, JwtConfig},
};

#[derive(Clone)]
pub struct AppContext {
    db: PgPool,
    auth: AuthContext,
    config: Config,
    redis: Client,
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
    pub const fn redis(&self) -> &Client {
        &self.redis
    }

    #[must_use]
    pub const fn auth(&self) -> &AuthContext {
        &self.auth
    }

    pub async fn init(&self) -> Result<(), crate::Error> {
        self.config().logger().setup()?;
        self.config().database().init().await?;

        Ok(())
    }
}

impl TryFrom<Config> for AppContext {
    type Error = Error;

    fn try_from(cfg: Config) -> Result<Self, Self::Error> {
        Ok(Self {
            db: cfg.database().pool()?,
            auth: cfg.auth().try_into()?,
            redis: cfg.redis().connection()?,
            config: cfg,
        })
    }
}

impl TryFrom<&Config> for AppContext {
    type Error = Error;

    fn try_from(cfg: &Config) -> Result<Self, Self::Error> {
        Ok(Self {
            redis: cfg.redis().connection()?,
            db: cfg.database().pool()?,
            auth: cfg.auth().try_into()?,
            config: cfg.clone(),
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

    pub fn generate_access_token(&self, sub: &str) -> Result<String, crate::Error> {
        self.access.generate_token(sub)
    }

    pub fn generate_refresh_token(&self, sub: &str) -> Result<String, crate::Error> {
        self.refresh.generate_token(sub)
    }

    pub fn verify_access_token(&self, token: &str) -> Result<Claims, crate::Error> {
        self.access.verify_token(token)
    }

    pub fn verify_refresh_token(&self, token: &str) -> Result<Claims, crate::Error> {
        self.refresh.verify_token(token)
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

    pub fn generate_token(&self, sub: &str) -> Result<String, crate::Error> {
        let now = chrono::Utc::now();

        let exp = (now + chrono::Duration::seconds(self.maxage)).timestamp();

        let claims = Claims::new(sub, exp, now);

        let header = Header::new(Algorithm::RS256);

        let token = jsonwebtoken::encode(&header, &claims, self.encoding_key())?;

        Ok(token)
    }

    pub fn verify_token(&self, token: &str) -> Result<Claims, crate::Error> {
        let token = jsonwebtoken::decode::<Claims>(
            token,
            self.decoding_key(),
            &Validation::new(Algorithm::RS256),
        )?;
        Ok(token.claims)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    id: String,
    sub: String,
    exp: i64,
    nbf: i64,
    iat: i64,
}

impl Claims {
    #[must_use]
    pub fn new(sub: &str, exp: i64, now: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            sub: sub.to_string(),
            exp,
            nbf: now.timestamp(),
            iat: now.timestamp(),
        }
    }

    #[must_use]
    pub fn sub(&self) -> &str {
        &self.sub
    }

    #[must_use]
    pub const fn exp(&self) -> i64 {
        self.exp
    }

    #[must_use]
    pub const fn nbf(&self) -> i64 {
        self.nbf
    }

    pub const fn set_nbf(&mut self, nbf: i64) {
        self.nbf = nbf;
    }

    pub const fn set_exp(&mut self, exp: i64) {
        self.exp = exp;
    }

    pub fn set_sub(&mut self, sub: String) {
        self.sub = sub;
    }

    #[must_use]
    pub fn id(&self) -> &str {
        &self.id
    }
}
