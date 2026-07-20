use std::sync::Arc;

use apalis::prelude::Storage;
use axum::{
    Extension, Json, Router,
    body::Body,
    debug_handler,
    extract::{Path, State},
    http::{
        HeaderValue, StatusCode,
        header::{AUTHORIZATION, SET_COOKIE},
    },
    response::{IntoResponse, Response},
    routing::{get, post},
};
use axum_extra::extract::cookie;

use crate::{
    AppContext, Result,
    context::Claims,
    middlewares::AuthLayer,
    repository::UserModel,
    utils::AppJson,
    validator::{LoginUser, RegisterUser, Validator, auth::ForgotPassword},
    views::{AuthResponse, LoginResponse, UserResponse},
    workers::MailJob,
};

#[debug_handler]
async fn register(
    State(ctx): State<Arc<AppContext>>,
    AppJson(params): AppJson<RegisterUser<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let user = UserModel::register_user(ctx.db(), validated).await?;

    if let Some(queue) = ctx.queue() {
        let mut welcome = queue.welcome.clone();
        welcome.push(MailJob { user_id: user.id }).await?;
    }

    Ok((StatusCode::CREATED, Json(UserResponse::new(&user))).into_response())
}

#[debug_handler]
async fn verify(State(ctx): State<Arc<AppContext>>, Path(token): Path<String>) -> Result<Response> {
    let user = UserModel::verify_user(ctx.db(), &token).await?;

    Ok((StatusCode::OK, Json(UserResponse::new(&user))).into_response())
}

#[debug_handler]
async fn forgot(
    State(ctx): State<Arc<AppContext>>,
    AppJson(params): AppJson<ForgotPassword<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let user = UserModel::forgot_password(ctx.db(), validated.email()).await?;

    if let Some(queue) = ctx.queue() {
        let mut forgot = queue.forgot.clone();
        forgot.push(MailJob { user_id: user.id }).await?;
    }

    Ok((
        StatusCode::OK,
        Json(AuthResponse::new(
            "An email with a password reset link has been sent to your email inbox.",
        )),
    )
        .into_response())
}

#[debug_handler]
async fn login(
    State(ctx): State<Arc<AppContext>>,
    AppJson(params): AppJson<LoginUser<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let user = UserModel::sign_in_user(ctx.db(), validated).await?;
    let user_id = user.id.to_string();

    let access_token = ctx.auth().generate_access_token(&user_id)?;
    let refresh_token = ctx.auth().generate_refresh_token(&user_id)?;

    let access_cookie = cookie::Cookie::build(("access_token", &access_token))
        .path("/")
        .http_only(false)
        .max_age(time::Duration::seconds(ctx.auth().access().maxage()))
        .same_site(cookie::SameSite::Lax)
        .secure(false);

    let refresh_cookie = cookie::Cookie::build(("refresh_token", &refresh_token))
        .path("/")
        .http_only(true)
        .max_age(time::Duration::seconds(ctx.auth().refresh().maxage()))
        .same_site(cookie::SameSite::Lax)
        .secure(false);

    let mut response = Response::builder().status(StatusCode::OK).body(Body::new(
        serde_json::json!(LoginResponse::new(&access_token, UserResponse::new(&user))).to_string(),
    ))?;

    response.headers_mut().append(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", access_token.as_str()))?,
    );
    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(access_cookie.to_string().as_str())?,
    );
    response.headers_mut().append(
        SET_COOKIE,
        HeaderValue::from_str(refresh_cookie.to_string().as_str())?,
    );

    Ok(response)
}

#[debug_handler]
#[tracing::instrument(skip(ctx))]
async fn current(
    State(ctx): State<Arc<AppContext>>,
    Extension(claims): Extension<Claims>,
) -> Result<Response> {
    let user = UserModel::find_user_by_claims_key(ctx.db(), claims.sub()).await?;

    Ok((StatusCode::OK, Json(UserResponse::new(&user))).into_response())
}

pub fn router(ctx: &Arc<AppContext>) -> Router {
    Router::new()
        .route("/sign-up", post(register))
        .route("/sign-in", post(login))
        .route("/forgot-password", post(forgot))
        .route("/me", get(current).layer(AuthLayer::new(ctx.clone())))
        .route("/verify/{token}", get(verify))
        .with_state(ctx.clone())
}
