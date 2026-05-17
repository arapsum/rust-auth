#![allow(unused_imports)]
use std::borrow::Cow;

use auth::{
    App,
    repository::UserModel,
    validator::{RegisterUser, auth::ResetPassword},
};
use insta::{Settings, assert_debug_snapshot, with_settings};
use serial_test::serial;
use uuid::Uuid;

use crate::{
    boot_test,
    utils::{cleanup_date, cleanup_password, cleanup_uuid},
};

macro_rules! configure_insta {
    ($(expr;expr),*) => {
        let mut settings = Settings::clone_current();
        settings.set_prepend_module_to_snapshot(false);
        settings.set_snapshot_suffix("users");
        settings.set_snapshot_path("snapshots/users");
        let _guard = settings.bind_to_scope();
    };
}

#[tokio::test]
#[serial]
async fn can_create_user() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    let params = RegisterUser::new(
        Cow::Owned("test@example.com".to_string()),
        Cow::Owned("testuser".to_string()),
        Cow::Owned("password".to_string()),
        Cow::Owned("password".to_string()),
    );

    let result = UserModel::register_user(ctx.db(), &params).await;

    with_settings!({
        filters => {
            let mut filters= cleanup_uuid().to_vec();
            filters.extend(cleanup_date().to_vec());
            filters.extend(cleanup_password().to_vec());
            filters
        }
    }, {
        assert_debug_snapshot!(result);
    })
}

#[tokio::test]
#[serial]
async fn can_find_user_by_email() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    crate::seed_data(ctx.db()).await.unwrap();

    let result = UserModel::find_user_by_email(ctx.db(), "jane.smith@globex.com").await;

    assert_debug_snapshot!(result);
}

#[tokio::test]
#[serial]
async fn can_find_user_by_id() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let id = Uuid::parse_str("3c008e68-88fa-4072-808e-6888fa60724c").unwrap();

    let result = UserModel::find_user_by_id(ctx.db(), id).await;

    assert_debug_snapshot!(result);
}

#[tokio::test]
#[serial]
async fn can_find_user_by_claims_key() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let key = "3c008e68-88fa-4072-808e-6888fa60724c";

    let result = UserModel::find_user_by_claims_key(ctx.db(), key).await;

    assert_debug_snapshot!(result);
}

#[tokio::test]
#[serial]
async fn can_verify_user() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let token = "e761d8e3-fc3e-4a2e-a6c9-7c7a4f2130e8";

    let result = UserModel::verify_user(ctx.db(), token).await;

    with_settings!({
        filters => {
            cleanup_date().to_vec()
        }
    }, {
        assert_debug_snapshot!(result);
    })
}

#[tokio::test]
#[serial]
async fn can_set_reset_token() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let email = "jane.smith@globex.com";

    let result = UserModel::forgot_password(ctx.db(), email).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_date().to_vec();
            filters.extend(cleanup_uuid().to_vec());
            filters
        }
    }, {
        assert_debug_snapshot!(result);
    })
}

#[tokio::test]
#[serial]
async fn can_reset_password() {
    configure_insta!();

    let ctx = boot_test().await.unwrap();

    App::seed(ctx.db()).await.unwrap();

    let email = "jane.smith@globex.com";

    let user = UserModel::forgot_password(ctx.db(), email).await.unwrap();

    let reset_token = user.reset_token.unwrap();

    let params = ResetPassword::new(
        Cow::Borrowed(&reset_token),
        Cow::Borrowed("NewPassword123!"),
        Cow::Borrowed("NewPassword123"),
    );

    let result = UserModel::reset_password(ctx.db(), &params).await;

    with_settings!({
        filters => {
            let mut filters = cleanup_date().to_vec();
            filters.extend(cleanup_password().to_vec());
            filters
        }
    }, {
        assert_debug_snapshot!(result);
    })
}
