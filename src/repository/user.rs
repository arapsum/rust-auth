#![allow(clippy::missing_errors_doc)]
use argon2::{
    Argon2, PasswordHash, PasswordVerifier,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};
use sqlx::{Encode, Executor, Postgres, prelude::FromRow};
use uuid::Uuid;

use crate::validator::{LoginUser, RegisterUser};

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
        let password_hash: String = Self::hash_password(params.password().trim())?;

        match sqlx::query_as::<_, Self>(
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
        .await
        {
            Ok(new_user) => Ok(new_user),
            Err(sqlx::Error::Database(err)) => {
                if err.is_unique_violation() {
                    Err(ModelError::EmailTaken)
                } else {
                    Err(ModelError::SqlxError(sqlx::Error::Database(err)))
                }
            }
            Err(e) => Err(ModelError::SqlxError(e)),
        }
    }

    pub async fn sign_in_user<'e, C>(db: &C, params: &LoginUser<'_>) -> ModelResult<Self>
    where
        for<'a> &'a C: Executor<'e, Database = Postgres>,
    {
        let user: Self = Self::find_user_by_email(db, params.email())
            .await?
            .ok_or_else(|| ModelError::InvalidCredentials)?;

        user.verify_password(params.password().trim())?;

        Ok(user)
    }

    pub async fn find_user_by_id<'e, C>(db: &C, id: Uuid) -> ModelResult<Self>
    where
        for<'a> &'a C: Executor<'e, Database = Postgres>,
    {
        let user: Self = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM users WHERE id = $1
        ",
        )
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)?;

        Ok(user)
    }

    pub async fn find_user_by_claims_key<'e, C>(db: &C, key: &str) -> ModelResult<Self>
    where
        for<'a> &'a C: Executor<'e, Database = Postgres>,
    {
        let id = Uuid::parse_str(key).map_err(|_| ModelError::InvalidClaimsKey)?;
        let user: Self = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM users WHERE id = $1
        ",
        )
        .bind(id)
        .fetch_optional(db)
        .await?
        .ok_or_else(|| ModelError::EntityNotFound)?;

        Ok(user)
    }

    pub async fn find_user_by_email<'e, C>(db: &C, email: &str) -> ModelResult<Option<Self>>
    where
        for<'a> &'a C: Executor<'e, Database = Postgres>,
    {
        let user: Option<Self> = sqlx::query_as::<_, Self>(
            r"
            SELECT * FROM users WHERE email = $1
        ",
        )
        .bind(email)
        .fetch_optional(db)
        .await?;

        Ok(user)
    }

    fn hash_password(password: &str) -> ModelResult<String> {
        let argon: Argon2<'_> = Argon2::default();
        let salt: SaltString = SaltString::generate(&mut OsRng);

        Ok(argon
            .hash_password(password.as_bytes(), &salt)
            .map_err(ModelError::PasswordHash)?
            .to_string())
    }

    fn verify_password(&self, plain_password: &str) -> ModelResult<()> {
        let parded_hash = PasswordHash::new(&self.password_hash)?;

        Argon2::default().verify_password(plain_password.as_bytes(), &parded_hash)?;

        Ok(())
    }
}
