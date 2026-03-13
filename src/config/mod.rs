use serde::{Deserialize, Serialize};
use tera::{Context, Tera};

mod error;
mod log;

pub use self::{
    error::{ConfigError, ConfigResult},
    log::{Format, Level, Logger},
};

#[derive(Debug, Deserialize)]
pub struct Config {
    server: ServerConfig,
    logger: Logger,
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
}

#[derive(Debug, Deserialize)]
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
    fn from(value: &str) -> Self {
        match value {
            "development" | "dev" => Self::Development,
            "production" | "prod" => Self::Production,
            "testing" | "test" => Self::Testing,
            _ => Self::Other(value.to_string()),
        }
    }
}

pub fn render_string(templ: &str, locals: &serde_json::Value) -> ConfigResult<String> {
    let text = Tera::one_off(templ, &Context::from_serialize(locals)?, false)?;

    Ok(text)
}
