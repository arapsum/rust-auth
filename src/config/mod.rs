#![allow(clippy::missing_errors_doc)]
use std::{
    fmt::{self, Display},
    path::{Path, PathBuf},
    time::Duration,
};

use jsonwebtoken::{DecodingKey, EncodingKey};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres, migrate::Migrator, postgres::PgPoolOptions};
use tera::{Context, Tera};

mod error;
mod log;

pub use self::{
    error::{ConfigError, ConfigResult},
    log::{Format, Level, Logger},
};

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    server: ServerConfig,
    logger: Logger,
    database: DatabaseConfig,
    auth: AuthConfig,
}

impl Config {
    pub fn from_env(env: &Environment) -> ConfigResult<Self> {
        let base_path = std::env::current_dir()?;
        let config_dir = base_path.join("config");

        let filename = config_dir.join(format!("{}.yaml", env.as_str()));

        let contents = std::fs::read_to_string(&filename)?;

        let rendered = render_string(&contents, &serde_json::json!({}))?;

        let config: Self = serde_saphyr::from_str(&rendered)?;

        Ok(config)
    }

    #[must_use]
    pub const fn server(&self) -> &ServerConfig {
        &self.server
    }

    #[must_use]
    pub const fn logger(&self) -> &Logger {
        &self.logger
    }

    #[must_use]
    pub const fn database(&self) -> &DatabaseConfig {
        &self.database
    }

    #[must_use]
    pub const fn auth(&self) -> &AuthConfig {
        &self.auth
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub(crate) protocol: String,
    pub(crate) host: String,
    pub(crate) port: u16,
}

impl ServerConfig {
    #[must_use]
    pub fn url(&self) -> String {
        format!("{}://{}:{}", self.protocol, self.host, self.port)
    }

    #[must_use]
    pub fn address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

impl Display for ServerConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.url())
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub(crate) uri: String,
    pub(crate) max_connections: u32,
    pub(crate) min_connections: u32,
    pub(crate) connection_timeout: u64,
    pub(crate) idle_timeout: u64,
    pub(crate) auto_migrate: bool,
    pub(crate) dangerously_truncate: bool,
    pub(crate) dangerously_recreate: bool,
}

impl DatabaseConfig {
    pub fn pool(&self) -> ConfigResult<Pool<Postgres>> {
        PgPoolOptions::new()
            .max_connections(self.max_connections)
            .min_connections(self.min_connections)
            .idle_timeout(Duration::from_secs(self.idle_timeout))
            .acquire_timeout(Duration::from_secs(self.connection_timeout))
            .connect_lazy(&self.uri)
            .map_err(Into::into)
    }

    #[allow(clippy::cast_possible_wrap)]
    pub async fn init(&self) -> ConfigResult<()> {
        let pool = self.pool()?;
        let migrator = Migrator::new(Path::new("migrations")).await?;

        let migrations = migrator.iter().count() as i64;

        if migrations == 0 {
            return Ok(());
        }

        if self.dangerously_recreate && self.dangerously_truncate {
            migrator.undo(&pool, migrations).await?;
            migrator.run(&pool).await?;
            return Ok(());
        }

        // TODO: delete all the data in the tables without dropping the tables.
        // if self.dangerously_truncate {
        // }

        if self.dangerously_recreate {
            migrator.undo(&pool, migrations).await?;
        }

        if self.auto_migrate {
            migrator.run(&pool).await?;
        }

        Ok(())
    }

    #[must_use]
    pub fn uri(&self) -> &str {
        &self.uri
    }

    #[must_use]
    pub const fn max_connections(&self) -> u32 {
        self.max_connections
    }

    #[must_use]
    pub const fn min_connections(&self) -> u32 {
        self.min_connections
    }

    #[must_use]
    pub const fn connection_timeout(&self) -> u64 {
        self.connection_timeout
    }

    #[must_use]
    pub const fn idle_timeout(&self) -> u64 {
        self.idle_timeout
    }

    #[must_use]
    pub const fn auto_migrate(&self) -> bool {
        self.auto_migrate
    }

    #[must_use]
    pub const fn dangerously_truncate(&self) -> bool {
        self.dangerously_truncate
    }

    #[must_use]
    pub const fn dangerously_recreate(&self) -> bool {
        self.dangerously_recreate
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct JwtConfig {
    pub(crate) private_key: PathBuf,
    pub(crate) public_key: PathBuf,
    pub(crate) maxage: i64,
}

impl JwtConfig {
    pub fn encoding_key(&self) -> ConfigResult<EncodingKey> {
        let contents = std::fs::read(&self.private_key)?;

        EncodingKey::from_rsa_pem(&contents).map_err(Into::into)
    }

    pub fn decoding_key(&self) -> ConfigResult<DecodingKey> {
        let contents = std::fs::read(&self.public_key)?;

        DecodingKey::from_rsa_pem(&contents).map_err(Into::into)
    }

    #[must_use]
    pub const fn maxage(&self) -> i64 {
        self.maxage
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    pub(crate) access: JwtConfig,
    pub(crate) refresh: JwtConfig,
}

impl AuthConfig {
    #[must_use]
    pub const fn access(&self) -> &JwtConfig {
        &self.access
    }

    #[must_use]
    pub const fn refresh(&self) -> &JwtConfig {
        &self.refresh
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Eq, PartialOrd, Ord)]
pub enum Environment {
    #[default]
    Development,
    Production,
    Testing,
    Other(String),
}

impl Environment {
    #[must_use]
    pub const fn as_str(&self) -> &str {
        match self {
            Self::Development => "development",
            Self::Production => "production",
            Self::Testing => "testing",
            Self::Other(other) => other.as_str(),
        }
    }
}

impl From<&str> for Environment {
    fn from(env: &str) -> Self {
        match env {
            "development" | "dev" => Self::Development,
            "production" | "prod" => Self::Production,
            "testing" | "test" => Self::Testing,
            _ => Self::Other(env.to_string()),
        }
    }
}

impl From<String> for Environment {
    fn from(env: String) -> Self {
        Self::from(env.as_str())
    }
}

impl Display for Environment {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

pub fn render_string(templ: &str, locals: &serde_json::Value) -> ConfigResult<String> {
    let text = Tera::one_off(templ, &Context::from_serialize(locals)?, false)?;

    Ok(text)
}
