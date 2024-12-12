use std::time::Duration;

use crate::helpers::{assert_is_redirected_to, spawn_test_server, ConfirmationLinks, TestApp};
use wiremock::matchers::{any, method, path};
use wiremock::{Mock, ResponseTemplate};

async fn create_unconfirmed_subscriber(app: &TestApp) -> ConfirmationLinks {
    let body = "name=chaitanya%20mandale&email=chaitanyam187%40gmail.com";

    let _mock_guard = Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .named("Create unconfirmed subscriber")
        .expect(1)
        .mount_as_scoped(&app.email_server)
        .await;
    app.post_subscriptions(body.into())
        .await
        .error_for_status()
        .unwrap();

    let email_request = &app
        .email_server
        .received_requests()
        .await
        .unwrap()
        .pop()
        .unwrap();
    app.get_confirmation_links(email_request)
}

async fn create_confirmed_subscriber(app: &TestApp) {
    let confirmation_link = create_unconfirmed_subscriber(app).await.html;
    reqwest::get(confirmation_link)
        .await
        .unwrap()
        .error_for_status()
        .unwrap();
}

#[tokio::test]
async fn newsletters_are_not_delivered_to_unconfirmed_subscribers() {
    let test_app = spawn_test_server().await;
    create_unconfirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(any())
        .respond_with(ResponseTemplate::new(200))
        .expect(0)
        .mount(&test_app.email_server)
        .await;

    // Submit newsletter form
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content_text": "Newsletter body as plain text",
        "content_html": "<p>Newsletter body as HTML</p>",
        "idempotency_key" : uuid::Uuid::new_v4().to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirected_to(&response, "/admin/newsletters");

    //  Follow the redirect
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));
}

#[tokio::test]
async fn newsletters_are_delivered_to_confirmed_subscribers() {
    let test_app = spawn_test_server().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Submit newsletter form
    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "content_text": "Newsletter body as plain text",
        "content_html": "<p>Newsletter body as HTML</p>",
        "idempotency_key" : uuid::Uuid::new_v4().to_string()

    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirected_to(&response, "/admin/newsletters");

    // Follow the redirect
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));
}

#[tokio::test]
async fn you_must_be_logged_in_to_see_the_newsletter_form() {
    let test_app = spawn_test_server().await;

    let response = test_app.get_publish_newsletter().await;

    assert_is_redirected_to(&response, "/login");
}

#[tokio::test]
async fn you_must_be_logged_in_to_publish_a_newsletter() {
    let test_app = spawn_test_server().await;

    let newsletter_request_body = serde_json::json!({
        "title": "Newsletter title",
        "text_content": "Newsletter body as plain text",
        "html_content": "<p>Newsletter body as HTML</p>",
        "idempotency_key" : uuid::Uuid::new_v4().to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;

    assert_is_redirected_to(&response, "/login");
}

#[tokio::test]
async fn newsletter_creation_is_idempotent() {
    let test_app = spawn_test_server().await;
    create_confirmed_subscriber(&test_app).await;
    test_app.test_user.login(&test_app).await;
    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Submit newsletter form
    let newsletter_request_body = serde_json::json!({
    "title": "Newsletter title",
    "content_text": "Newsletter body as plain text",
    "content_html": "<p>Newsletter body as HTML</p>",
    "idempotency_key": uuid::Uuid::new_v4().to_string()
    });
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirected_to(&response, "/admin/newsletters");

    // Follow the redirect
    let html_page = test_app.get_publish_newsletter_html().await;
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));

    // Submit newsletter form **again**
    let response = test_app
        .post_publish_newsletter(&newsletter_request_body)
        .await;
    assert_is_redirected_to(&response, "/admin/newsletters");
    assert!(html_page.contains("<p><i>The newsletter issue has been published!</i></p>"));
}

#[tokio::test]
async fn concurrent_form_submission_is_handled_gracefully() {
    let test_app = spawn_test_server().await;

    create_confirmed_subscriber(&test_app).await;

    test_app.test_user.login(&test_app).await;

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200).set_delay(Duration::from_secs(2)))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    // Submit two newsletter forms concurrently
    let newsletter_request_body = serde_json::json!({
    "title": "Newsletter title",
    "content_text": "Newsletter body as plain text",
    "content_html": "<p>Newsletter body as HTML</p>",
    "idempotency_key": uuid::Uuid::new_v4().to_string()
    });

    let response1 = test_app.post_publish_newsletter(&newsletter_request_body);
    let response2 = test_app.post_publish_newsletter(&newsletter_request_body);

    let (response1, response2) = tokio::join!(response1, response2);

    assert_eq!(response1.status(), response2.status());
    assert_eq!(
        response1.text().await.unwrap(),
        response2.text().await.unwrap()
    );
}
