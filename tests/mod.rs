use std::{
    future::Future,
    sync::{Arc, OnceLock},
};

use auth::{
    App, AppContext, Result,
    config::{Config, Environment},
    controllers,
};
use axum_test::{TestServer, TestServerConfig};
use sqlx::PgPool;

mod repository;
mod requests;
mod seed;

pub async fn boot_test() -> Result<Arc<AppContext>> {
    let config = Config::from_env(&Environment::Testing)?;
    let ctx = Arc::new(AppContext::try_from(&config)?);

    ctx.init().await?;

    Ok(ctx)
}

pub async fn request<F, Fut>(f: F)
where
    F: FnOnce(TestServer, Arc<AppContext>) -> Fut,
    Fut: Future<Output = ()>,
{
    let ctx = boot_test().await.unwrap();

    let cfg = TestServerConfig {
        default_content_type: Some("application/json".into()),
        save_cookies: true,
        ..Default::default()
    };

    let server = TestServer::new_with_config(controllers::router(&ctx), cfg);

    f(server, ctx).await
}

pub async fn seed_data(db: &PgPool) -> Result<()> {
    App::seed(db).await
}

static CLEANUP_UUID: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_DATE: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_PASSWORD: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_JWT: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_HEADERS: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();

pub fn cleanup_uuid() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_UUID.get_or_init(|| {
        vec![(
            r"([0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12})",
            "ID",
        )]
    })
}

pub fn cleanup_date() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_DATE.get_or_init(|| {
        vec![
            (
                r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}(\.\d+)?\+\d{2}:\d{2}",
                "DATE",
            ), // with tz
            (r"\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d+", "DATE"),
            (r"(\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})", "DATE"),
            (r"(\d{2})-(\d{2})-(\d{4})\s+(\d{2}):(\d{2}):(\d{2})", "DATE"),
            (r#"\d{2}-\d{2}-\d{4} \d{2}:\d{2}"#, "DATE"),
            (r"\d{4}[-/]\d{2}[-/]\d{2}", "DATE"),
        ]
    })
}

pub fn cleanup_password() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_PASSWORD
        .get_or_init(|| vec![(r"password_hash: (.*{60}),", "password_hash: \"PASSWORD\",")])
}

pub fn cleanup_jwt() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_JWT.get_or_init(|| vec![(r"[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+\.[A-Za-z0-9-_]+", "JWT")])
}

pub fn cleanup_headers() -> &'static Vec<(&'static str, &'static str)> {
    CLEANUP_HEADERS.get_or_init(|| {
        vec![(
            r#""content-length":\s*"\d+""#,
            r#""content-length": "NUMBER""#,
        )]
    })
}
