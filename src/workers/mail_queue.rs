#![allow(clippy::missing_errors_doc)]

use std::sync::Arc;

use apalis::prelude::*;
use apalis_redis::{ConnectionManager, RedisStorage};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AppContext, config::RedisConfig, mailer::AuthMailer, repository::UserModel};

#[derive(Debug, Serialize, Deserialize)]
pub struct MailJob {
    pub user_id: Uuid,
}

pub struct MailQueue {
    pub welcome: RedisStorage<MailJob>,
    pub forgot: RedisStorage<MailJob>,
}

impl MailQueue {
    pub async fn init(cfg: &RedisConfig) -> crate::Result<Self> {
        let conn: ConnectionManager = apalis_redis::connect(cfg.url()).await?;

        Ok(Self {
            welcome: RedisStorage::new(conn.clone()),
            forgot: RedisStorage::new(conn),
        })
    }
}

pub async fn handle_welcome(job: MailJob, ctx: Data<Arc<AppContext>>) -> Result<(), Error> {
    let user = UserModel::find_user_by_id(ctx.db(), job.user_id)
        .await
        .map_err(|e| Error::Failed(Arc::new(e.into())))?;

    AuthMailer::send_welcome(&ctx, &user)
        .await
        .map_err(|e| Error::Failed(Arc::new(e.into())))?;

    Ok(())
}

pub async fn handle_forgot_password(job: MailJob, ctx: Data<Arc<AppContext>>) -> Result<(), Error> {
    let user = UserModel::find_user_by_id(ctx.db(), job.user_id)
        .await
        .map_err(|e| Error::Failed(Arc::new(e.into())))?;

    AuthMailer::forgot_password(&ctx, &user)
        .await
        .map_err(|e| Error::Failed(Arc::new(e.into())))?;

    Ok(())
}
