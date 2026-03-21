#![allow(clippy::missing_errors_doc)]
use std::net::SocketAddr;
use std::{io::IsTerminal, sync::Arc};

use axum::Router;
use clap::Parser;
use color_eyre::config::{HookBuilder, Theme};
use dotenvy::dotenv;
use sqlx::PgPool;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::repository::UserModel;
use crate::{
    AppContext, Result,
    config::{Config, Environment},
    controllers,
    middlewares::trace,
};

/// Auth app configuration
#[derive(Debug, Parser)]
#[command(version = env!("CARGO_PKG_VERSION"), about = "Authentication service using Axum", author = env!("CARGO_PKG_AUTHORS"), long_about=None)]
pub struct App {
    /// The environment to run the app in: development, production, testing, or other
    #[arg(short, long, default_value_t = Environment::default())]
    env: Environment,

    #[command(subcommand)]
    command: Option<Commands>,
}

pub struct AppResult {
    pub listener: TcpListener,
    pub router: Router,
}

impl AppResult {
    pub const fn new(listener: TcpListener, router: Router) -> Self {
        Self { listener, router }
    }
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self::parse()
    }

    pub fn config(&self) -> Result<Config> {
        Config::from_env(&self.env).map_err(Into::into)
    }

    pub async fn init(&self, config: &Config) -> Result<Arc<AppContext>> {
        config.logger().setup()?;
        config.database().init().await?;

        let ctx = AppContext::try_from(config)?;

        match self.command {
            Some(Commands::Seed) => {
                Self::seed(ctx.db()).await?;
            }
            None => {}
        }

        Ok(Arc::new(ctx))
    }

    pub async fn create(&self) -> Result<AppResult> {
        let config = self.config()?;
        let ctx = self.init(&config).await?;

        let listener = TcpListener::bind(config.server().address()).await?;

        let router = Router::new()
            .nest("/api", controllers::router(&ctx))
            .fallback(controllers::fallback)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(trace::make_span_with)
                    .on_request(trace::on_request)
                    .on_response(trace::on_response)
                    .on_failure(trace::on_failure),
            );

        Ok(AppResult::new(listener, router))
    }

    pub async fn run(&self) -> Result<()> {
        dotenv().ok();

        HookBuilder::new().theme(if std::io::stderr().is_terminal() {
            Theme::dark()
        } else {
            Theme::new()
        });

        let this = Self::parse();

        let config = this.config()?;
        let app_result = this.create().await?;

        tracing::info!("Server running at {}", config.server().url());

        axum::serve(
            app_result.listener,
            app_result
                .router
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(Into::into)
    }

    pub async fn seed(db: &PgPool) -> Result<()> {
        UserModel::seed_data(db, "users.json").await?;

        Ok(())
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Commands {
    /// Seeds the database with initial data
    Seed,
}
