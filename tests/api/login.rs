use crate::helpers::{assert_is_redirected_to, spawn_test_server};

#[tokio::test]
async fn an_error_flash_message_is_set_on_failure() {
    let test_app = spawn_test_server().await;

    let login_body = serde_json::json!({
        "username" : "random-username",
        "password" : "random-password"
    });
    // Try To Login With Invalid Creds
    let response = test_app.post_login(&login_body).await;

    assert_is_redirected_to(&response, "/login".into());

    //Follow the redirect
    let html_page = test_app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));

    //Reload The Login Page
    let html_page = test_app.get_login_html().await;
    assert!(!html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
}

#[tokio::test]
async fn redirect_to_admin_dashboard_after_login_success() {
    let test_app = spawn_test_server().await;

    let login_body = serde_json::json!({
    "username": &test_app.test_user.username,
    "password": &test_app.test_user.password
    });
    let response = test_app.post_login(&login_body).await;
    assert_is_redirected_to(&response, "/admin/dashboard");

    let html_page = test_app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", test_app.test_user.username)));
}
