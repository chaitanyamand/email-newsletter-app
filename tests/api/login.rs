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
    let flash_cookie = response.cookies().find(|c| c.name() == "_flash").unwrap();
    assert_eq!(flash_cookie.value(), "Authentication failed");

    //Follow the redirect
    let html_page = test_app.get_login_html().await;
    assert!(html_page.contains(r#"<p><i>Authentication failed</i></p>"#));

    //Reload The Login Page
    let html_page = test_app.get_login_html().await;
    assert!(!html_page.contains(r#"<p><i>Authentication failed</i></p>"#));
}
