use crate::helpers::{assert_is_redirected_to, spawn_test_server};

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    let test_app = spawn_test_server().await;

    let response = test_app.get_admin_dashboard().await;

    assert_is_redirected_to(&response, "/login");
}
