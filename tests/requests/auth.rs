use auth::{repository::UserModel, views::UserResponse};
use insta::{Settings, assert_debug_snapshot, with_settings};
use rstest::rstest;
use serial_test::serial;

use crate::utils;

macro_rules! configure_insta {
    ($(expr:expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_path("snapshots/auth");
        settings.set_snapshot_suffix("auth");
        let _guard = settings.bind_to_scope();
    };
}

#[rstest]
#[case(
    "can_successfully_register_user",
    serde_json::json!({
        "email":  "test1@example.com",
        "username": "test_user1",
        "password": "SafePassWord11!",
        "confirmPassword": "SafePassWord11!"
    })
)]
#[case(
    "when_passwords_do_not_match_registration_fails",
    serde_json::json!({
        "email":  "test1@example.com",
        "username": "test_user1",
        "password": "SafePassWord11!",
        "confirmPassword": "SafePassWord11"
    })
)]
#[case(
    "when_password_is_under_eight_chars_registration_fails",
    serde_json::json!({
        "email":  "test1@example.com",
        "username": "test_user1",
        "password": "Safe",
        "confirmPassword": "Safe"
    })
)]
#[case(
    "when_password_contains_whitespace_char_registration_fails",
    serde_json::json!({
        "email":  "test1@example.com",
        "username": "test_user1",
        "password": "ChangeMe 123!",
        "confirmPassword": "ChangeMe 123!"
    })
)]
#[case(
    "when_password_contains_comma_char_registration_fails",
    serde_json::json!({
        "email":  "test1@example.com",
        "username": "test_user1",
        "password": "ChangeMe,123!",
        "confirmPassword": "ChangeMe,123!"
    })
)]
#[case(
    "when_email_is_not_an_email_registration_fails",
    serde_json::json!({
        "email":  "test1:example.com",
        "username": "test_user1",
        "password": "Password123",
        "confirmPassword": "Password123"
    })
)]
#[case(
    "when_username_is_under_six_chars_registration_fails",
    serde_json::json!({
        "email":  "test1@example.com",
        "username": "test",
        "password": "Password123",
        "confirmPassword": "Password123"
    })
)]
#[case(
    "when_username_contains_non_alphanumeric_chars_registration_fails",
    serde_json::json!({
        "email":  "test1@example.com",
        "username": "test user;+",
        "password": "Password123",
        "confirmPassword": "Password123"
    })
)]
#[tokio::test]
#[serial]
async fn can_register_user(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server.post("/auth/sign-up").json(&params).await;

        with_settings!({
            filters => {
                let mut filters = utils::cleanup_date().to_vec();
                filters.extend(utils::cleanup_uuid().to_vec());
                filters
            }
        },  {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()))
        })
    })
    .await;
}

#[rstest]
#[case(
    "can_set_reset_token_when_user_exists",
    serde_json::json!({
        "email":  "james.moriaty@continental.org",
    })
)]
#[case(
    "when_user_does_not_exist_forgot_password_fails",
    serde_json::json!({
        "email":  "test1@example.com",
    })
)]
#[tokio::test]
#[serial]
async fn can_forgot_password(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server.post("/auth/forgot-password").json(&params).await;

        with_settings!({
            filters => {
                let mut filters = utils::cleanup_date().to_vec();
                filters.extend(utils::cleanup_uuid().to_vec());
                filters
            }
        },  {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()))
        })
    })
    .await;
}

#[rstest]
#[case("can_successfully_login_user", serde_json::json!({
    "email": "john.doe@acme.com",
    "password": "Password"
}))]
#[case("when_password_is_wrong_login_fails", serde_json::json!({
    "email": "john.doe@acme.com",
    "password": "Password1"
}))]
#[case("when_email_is_wrong_login_fails", serde_json::json!({
    "email": "james.doe@acme.com",
    "password": "Password1"
}))]
#[case("when_email_is_not_email_login_fails", serde_json::json!({
    "email": "james.doe@acme.com",
    "password": "Password1"
}))]
#[case("when_password_is_too_short_login_fails", serde_json::json!({
    "email": "james.doe@acme.com",
    "password": "Pas"
}))]
#[tokio::test]
#[serial]
async fn can_login_user(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let response = server.post("/auth/sign-in").json(&params).await;

        with_settings!({
            filters => {
                let mut filters = utils::cleanup_date().to_vec();
                filters.extend(utils::cleanup_uuid().to_vec());
                filters.extend(utils::cleanup_jwt().to_vec());
                filters.extend(utils::cleanup_headers());
                filters
            }
        },  {
            assert_debug_snapshot!(test_name, (response.status_code(), response.headers(),response.text()))
        })
    })
    .await;
}

#[rstest]
#[case("can_get_current_user", serde_json::json!({
    "email": "john.doe@acme.com",
    "password": "Password"
}))]
#[tokio::test]
#[serial]
async fn can_get_current_user(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, context| async move {
        configure_insta!();

        crate::seed_data(context.db())
            .await
            .expect("Failed to seed data");

        let user: utils::LoggedInUser = utils::login_users(&server, &params).await;
        let (auth_header, auth_value) = utils::auth_header(user.access_token);

        let response = server
            .get("/auth/me")
            .add_header(auth_header, auth_value)
            .add_cookie(user.access_cookie)
            .add_cookie(user.refresh_cookie)
            .await;

        with_settings!({
            filters => {
                let mut filters = utils::cleanup_date().to_vec();
                filters.extend(utils::cleanup_uuid().to_vec());
                filters
            }
        },  {
            assert_debug_snapshot!(test_name, (response.status_code(), response.text()))
        })
    })
    .await;
}

#[tokio::test]
#[serial]
async fn can_verify_user(#[case] test_name: &str, #[case] params: serde_json::Value) {
    crate::request(|server, ctx| async move {
        configure_insta!();

        crate::seed_data(ctx.db())
            .await
            .expect("Failed to seed data");

        let params = serde_json::json!({
        "email":  "test1@example.com",
        "username": "test_user1",
        "password": "SafePassWord11!",
        "confirmPassword": "SafePassWord11!"
        });

        let register_response = server.post("/auth/sign-up").json(&params).await;

        let user_response = register_response.json::<UserResponse>();

        let user = UserModel::find_user_by_id(ctx.db(), user_response.id)
            .await
            .unwrap();

        let response = server
            .get(&format!("/auth/verify/{}", user.verification_token))
            .await;

        with_settings!({
            filters => {
                let mut filters = utils::cleanup_date().to_vec();
                filters.extend(utils::cleanup_uuid().to_vec());
                filters
            }
        },  {
            assert_debug_snapshot!((response.status_code(), response.text()))
        })
    })
    .await;
}
