use crate::helpers::{assert_is_redirected_to, spawn_test_server};

#[tokio::test]
async fn you_must_be_logged_in_to_access_the_admin_dashboard() {
    let test_app = spawn_test_server().await;

    let response = test_app.get_admin_dashboard().await;

    assert_is_redirected_to(&response, "/login");
}

#[tokio::test]
async fn logout_clears_session_state() {
    let test_app = spawn_test_server().await;

    // Login
    let login_body = serde_json::json!({
    "username": &test_app.test_user.username,
    "password": &test_app.test_user.password
    });
    let response = test_app.post_login(&login_body).await;
    assert_is_redirected_to(&response, "/admin/dashboard");

    // Follow the redirect
    let html_page = test_app.get_admin_dashboard_html().await;
    assert!(html_page.contains(&format!("Welcome {}", test_app.test_user.username)));

    // Logout
    let response = test_app.post_logout().await;
    assert_is_redirected_to(&response, "/login");

    // Follow the redirect
    let html_page = test_app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>You have successfully logged out.</i></p>"#));

    // Attempt to load admin panel
    let response = test_app.get_admin_dashboard().await;
    assert_is_redirected_to(&response, "/login");
}
