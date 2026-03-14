#![allow(clippy::missing_errors_doc)]
use std::net::SocketAddr;
use std::{io::IsTerminal, sync::Arc};

use axum::Router;
use clap::Parser;
use color_eyre::config::{HookBuilder, Theme};
use dotenvy::dotenv;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{
    AppContext, Result,
    config::{Config, Environment},
    controllers,
    middlewares::trace,
};

/// Auth app configuration
#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
pub struct App {
    /// The environment to run the app in
    #[arg(short, long, default_value_t = Environment::default())]
    env: Environment,
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self::parse()
    }
    pub async fn run(&self) -> Result<()> {
        dotenv().ok();

        HookBuilder::new().theme(if std::io::stderr().is_terminal() {
            Theme::dark()
        } else {
            Theme::new()
        });

        let config = Config::from_env(&self.env)?;

        config.logger().setup()?;

        config.database().init().await?;

        let listener = TcpListener::bind(config.server().address()).await?;

        tracing::info!("Server running at {}", config.server().url());

        let ctx = Arc::new(AppContext::try_from(config)?);

        let app = Router::new()
            .nest("/api", controllers::router(&ctx))
            .fallback(controllers::fallback)
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(trace::make_span_with)
                    .on_request(trace::on_request)
                    .on_response(trace::on_response)
                    .on_failure(trace::on_failure),
            );

        axum::serve(
            listener,
            app.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .map_err(Into::into)
    }
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}
