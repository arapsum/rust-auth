#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error(transparent)]
    AppenderInit(#[from] tracing_appender::rolling::InitError),
    #[error(transparent)]
    DirectiveParseError(#[from] tracing_subscriber::filter::ParseError),
    #[error(transparent)]
    EnvFilter(#[from] std::env::VarError),
    #[error(transparent)]
    FromEnv(#[from] tracing_subscriber::filter::FromEnvError),
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error("Non-blocking work guard already set")]
    NonBlockingWorkGuardAlreadySet,
    #[error(transparent)]
    SerdeSaphyr(#[from] serde_saphyr::Error),
    #[error(transparent)]
    TeraError(#[from] tera::Error),
    #[error(transparent)]
    TryInit(#[from] tracing_subscriber::util::TryInitError),
}

pub type ConfigResult<T> = Result<T, ConfigError>;
