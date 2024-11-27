use crate::helpers::spawn_test_server;
use tokio::time::{sleep, Duration};
use wiremock::matchers::{method, path};
use wiremock::{Mock, ResponseTemplate};

#[tokio::test]
async fn subscribe_returns_200_when_data_is_valid() {
    let test_app = spawn_test_server().await;
    sleep(Duration::from_secs(2)).await;

    let body = "name=chaitanya%20mandale&email=chaitanyam187%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let response = test_app.post_subscriptions(body.into()).await;

    assert_eq!(response.status().as_u16(), 200, "Status code was not 200");
}

#[tokio::test]
async fn subscribe_persists_the_new_subscriber() {
    let test_app = spawn_test_server().await;
    sleep(Duration::from_secs(2)).await;

    let body = "name=chaitanya%20mandale&email=chaitanyam187%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .mount(&test_app.email_server)
        .await;

    let _ = test_app.post_subscriptions(body.into()).await;

    let saved = sqlx::query!("SELECT email,name,status FROM subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "chaitanyam187@gmail.com");
    assert_eq!(saved.name, "chaitanya mandale");
    assert_eq!(saved.status, "pending_confirmation");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_invalid() {
    let test_app = spawn_test_server().await;
    sleep(Duration::from_secs(2)).await;

    let test_cases = vec![
        ("name=chaitanya%20mandale", "email not sent"),
        ("email=chaitanyam187%40gmail.com", "name not sent"),
        ("", "no data sent at all"),
    ];

    for (body, error_message) in test_cases {
        let response = test_app.post_subscriptions(body.into()).await;

        assert_eq!(
            response.status().as_u16(),
            400,
            "API did not return error code 400 for {}",
            error_message
        );
    }
}

#[tokio::test]
async fn subscribe_returns_a_400_when_fields_are_present_but_empty() {
    let test_app = spawn_test_server().await;
    sleep(Duration::from_secs(2)).await;

    let test_cases = vec![
        ("name=&email=chaitanyam187%40gmail.com", "empty name"),
        ("name=chaitanya%20mandale&email=", "empty email"),
        (
            "name=chaitanya%20mandale&email=definitely-not-an-email",
            "invalid email",
        ),
    ];

    for (body, description) in test_cases {
        let response = test_app.post_subscriptions(body.into()).await;

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 OK when the payload was {}",
            description
        );
    }
}

#[tokio::test]
async fn subscribe_sends_a_confirmation_email_for_valid_data() {
    let test_app = spawn_test_server().await;
    let body = "name=chaitanya%20mandale&email=chaitanyam187%40gmail.com";

    Mock::given(path("/email"))
        .and(method("POST"))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&test_app.email_server)
        .await;

    test_app.post_subscriptions(body.into()).await;

    let email_request = &test_app.email_server.received_requests().await.unwrap()[0];
    let confirmation_links = test_app.get_confirmation_links(&email_request);

    assert_eq!(confirmation_links.html, confirmation_links.plain_text);
}

#[tokio::test]
async fn subscribe_fails_if_there_is_a_fatal_database_error() {
    let test_app = spawn_test_server().await;
    let body = "name=chaitanya%20mandale&email=chaitanyam187%40gmail.com";

    sqlx::query!("ALTER TABLE subscription_tokens DROP COLUMN subscription_token;",)
        .execute(&test_app.db_pool)
        .await
        .unwrap();

    let response = test_app.post_subscriptions(body.into()).await;

    assert_eq!(response.status().as_u16(), 500);
}
