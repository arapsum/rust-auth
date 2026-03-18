use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::validator::auth::RegisterUser;

use super::{ModelError, ModelResult};

#[derive(Debug, Deserialize, Clone, Serialize, FromRow, Encode)]
#[serde(rename_all = "camelCase")]
pub struct UserModel {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub password_hash: String,
    pub image: Option<String>,
    pub verified_at: Option<DateTime<FixedOffset>>,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
}

impl UserModel {
    pub async fn register_user<'e, C>(db: &C, params: &RegisterUser<'_>) -> ModelResult<Self>
    where
        for<'a> &'a C: Executor<'e, Database = Postgres>,
    {
        let password_hash = Self::hash_password(params.password().trim())?;

        let new_user = sqlx::query_as::<_, Self>(
            r"
            INSERT INTO users (email, password_hash, username)
            VALUES ($1, $2, $3)
            RETURNING *
        ",
        )
        .bind(params.email().trim())
        .bind(password_hash)
        .bind(params.username().trim())
        .fetch_one(db)
        .await?;

        Ok(new_user)
    }

    fn hash_password(password: &str) -> ModelResult<String> {
        let argon = Argon2::default();
        let salt = SaltString::generate(&mut OsRng);

        Ok(argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(ModelError::PasswordHash)?
            .to_string())
    }
}
