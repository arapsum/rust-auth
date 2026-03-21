use std::{future::Future, sync::Arc};

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
mod utils;

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
