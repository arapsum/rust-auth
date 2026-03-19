#![allow(unused_imports)]
use std::borrow::Cow;

use auth::{repository::UserModel, validator::RegisterUser};
use insta::{Settings, assert_debug_snapshot, with_settings};
use serial_test::serial;

use crate::{boot_test, cleanup_date, cleanup_password, cleanup_uuid};

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

    let params = RegisterUser::new(
        Cow::Owned("test@example.com".to_string()),
        Cow::Owned("testuser".to_string()),
        Cow::Owned("password".to_string()),
        Cow::Owned("password".to_string()),
    );

    UserModel::register_user(ctx.db(), &params).await.unwrap();

    let result = UserModel::find_user_by_email(ctx.db(), "test@example.com").await;

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
