use std::sync::{Arc, OnceLock};

use auth::{
    AppContext, Result,
    config::{Config, Environment},
};

mod repository;

pub async fn boot_test() -> Result<Arc<AppContext>> {
    let config = Config::from_env(&Environment::Testing)?;
    let ctx = Arc::new(AppContext::try_from(&config)?);

    ctx.init().await?;

    Ok(ctx)
}

static CLEANUP_UUID: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_DATE: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();
static CLEANUP_PASSWORD: OnceLock<Vec<(&'static str, &'static str)>> = OnceLock::new();

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
