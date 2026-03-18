use std::sync::Arc;

use axum::{
    Json, Router,
    body::Body,
    debug_handler,
    extract::State,
    http::{
        HeaderValue, StatusCode,
        header::{AUTHORIZATION, SET_COOKIE},
    },
    response::{IntoResponse, Response},
    routing::post,
};
use axum_extra::extract::cookie;

use crate::{
    AppContext, Result,
    middlewares::AppJson,
    repository::UserModel,
    validator::{LoginUser, RegisterUser, Validator},
    views::{LoginResponse, UserResponse},
};

#[debug_handler]
async fn register(
    State(ctx): State<Arc<AppContext>>,
    AppJson(params): AppJson<RegisterUser<'static>>,
) -> Result<Response> {
    let validator = Validator::new(params);
    let validated = validator.validate()?;

    let user = UserModel::register_user(ctx.db(), validated).await?;

    Ok((StatusCode::OK, Json(UserResponse::new(&user))).into_response())
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

    let user_response = UserResponse::new(&user);

    let mut response = Response::builder().status(StatusCode::OK).body(Body::new(
        serde_json::json!(LoginResponse::new(&access_token, user_response)).to_string(),
    ))?;

    response
        .headers_mut()
        .append(AUTHORIZATION, HeaderValue::from_str(access_token.as_str())?);
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

pub fn router(ctx: &Arc<AppContext>) -> Router {
    Router::new()
        .route("/sign-up", post(register))
        .route("/sign-in", post(login))
        .with_state(ctx.clone())
}
