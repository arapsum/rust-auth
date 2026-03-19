use auth::App;
use serial_test::serial;

use crate::boot_test;

#[tokio::test]
#[serial]
async fn can_seed() {
    let ctx = boot_test().await.unwrap();

    let result = App::seed(ctx.db()).await;

    assert!(result.is_ok());
}
