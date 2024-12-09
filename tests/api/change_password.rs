use uuid::Uuid;

use crate::helpers::{assert_is_redirected_to, spawn_test_server};

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_change_password_form() {
    let test_app = spawn_test_server().await;

    let response = test_app.get_change_password().await;

    assert_is_redirected_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_change_password() {
    let test_app = spawn_test_server().await;
    let new_password = Uuid::new_v4().to_string();

    let response = test_app.post_change_password(&serde_json::json!(
       { "current_password" : Uuid::new_v4().to_string(), "new_password" : &new_password ,"new_password_check" : &new_password}
    )).await;

    assert_is_redirected_to(&response, "/login");
}

#[tokio::test]
async fn new_password_fields_must_match() {
    let test_app = spawn_test_server().await;
    let new_password = Uuid::new_v4().to_string();
    let another_new_password = Uuid::new_v4().to_string();

    // Login
    test_app
        .post_login(&serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password
        }))
        .await;

    // Try to change password (with non-matching new passwords)
    let response = test_app
        .post_change_password(&serde_json::json!({
        "current_password": &test_app.test_user.password,
        "new_password": &new_password,
        "new_password_check": &another_new_password,
        }))
        .await;
    assert_is_redirected_to(&response, "/admin/password");

    // Follow the redirect
    let html_page = test_app.get_change_password_html().await;
    assert!(html_page.contains(
        "<p><i>You entered two different new passwords - \
            the field values must match.</i></p>"
    ));
}

#[tokio::test]
async fn current_password_must_be_valid() {
    let test_app = spawn_test_server().await;
    let new_password = Uuid::new_v4().to_string();
    let wrong_password = Uuid::new_v4().to_string();

    // Login
    test_app
        .post_login(&serde_json::json!({
        "username": &test_app.test_user.username,
        "password": &test_app.test_user.password
        }))
        .await;

    // Try to change password (with wrong current password)
    let response = test_app
        .post_change_password(&serde_json::json!({
        "current_password": &wrong_password,
        "new_password": &new_password,
        "new_password_check": &new_password,
        }))
        .await;
    assert_is_redirected_to(&response, "/admin/password");

    // Follow the redirect
    let html_page = test_app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>The current password is incorrect.</i></p>"));
}

#[tokio::test]
async fn changing_password_works() {
    let test_app = spawn_test_server().await;
    let new_password = Uuid::new_v4().to_string();

    // Login
    let login_body = serde_json::json!({
    "username": &test_app.test_user.username,
    "password": &test_app.test_user.password
    });
    let response = test_app.post_login(&login_body).await;
    assert_is_redirected_to(&response, "/admin/dashboard");

    // Change password
    let response = test_app
        .post_change_password(&serde_json::json!({
        "current_password": &test_app.test_user.password,
        "new_password": &new_password,
        "new_password_check": &new_password,
        }))
        .await;
    assert_is_redirected_to(&response, "/admin/password");

    // Follow the redirect
    let html_page = test_app.get_change_password_html().await;
    assert!(html_page.contains("<p><i>Your password has been changed.</i></p>"));

    // Logout
    let response = test_app.post_logout().await;
    assert_is_redirected_to(&response, "/login");

    // Follow the redirect
    let html_page = test_app.get_login_html().await;
    assert!(html_page.contains("<p><i>You have successfully logged out.</i></p>"));

    // Login using the new password
    let login_body = serde_json::json!({
    "username": &test_app.test_user.username,
    "password": &new_password
    });
    let response = test_app.post_login(&login_body).await;
    assert_is_redirected_to(&response, "/admin/dashboard");
}
