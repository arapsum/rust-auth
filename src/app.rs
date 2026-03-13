use std::io::IsTerminal;
use std::net::SocketAddr;

use axum::Router;
use clap::Parser;
use color_eyre::config::{HookBuilder, Theme};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

use crate::{
    Result,
    config::{Config, Environment},
    middlewares::trace,
};

/// Auth app configuration
#[derive(Debug, Parser)]
#[command(version, about, long_about=None)]
pub struct App {
    /// The environment to run the app in
    #[arg(short, long)]
    env: Environment,
}

impl App {
    #[must_use]
    pub fn new() -> Self {
        Self::parse()
    }
    pub async fn run(&self) -> Result<()> {
        HookBuilder::new().theme(if std::io::stderr().is_terminal() {
            Theme::dark()
        } else {
            Theme::new()
        });

        let config = Config::from_env(&self.env)?;

        config.logger().setup()?;

        let listener = TcpListener::bind(config.server().address()).await?;
        let app = Router::new()
            .route(
                "/",
                axum::routing::get(|| async {
                    axum::Json(serde_json::json!({"message": "Server is up and running!"}))
                }),
            )
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(trace::make_span_with)
                    .on_request(trace::on_request)
                    .on_response(trace::on_response)
                    .on_failure(trace::on_failure),
            );

        tracing::info!("Server running at {}", config.server().url());

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
