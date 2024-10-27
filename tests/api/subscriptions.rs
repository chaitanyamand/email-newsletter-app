use crate::helpers::spawn_test_server;
use reqwest::Client;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn subscribe_returns_200_when_data_is_valid() {
    let test_app = spawn_test_server().await;
    let req_address = test_app.address;
    sleep(Duration::from_secs(2)).await;

    let db_pool = test_app.db_pool;

    let client = Client::new();
    let url = format!("{}/subscribe", req_address);
    let body = "name=chaitanya%20mandale&email=chaitanyam187%40gmail.com";
    let response = client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to send POST request");

    assert_eq!(response.status().as_u16(), 200, "Status code was not 200");

    let saved = sqlx::query!("SELECT email,name FROM subscriptions")
        .fetch_one(&db_pool)
        .await
        .expect("Failed to fetch saved subscription");

    assert_eq!(saved.email, "chaitanyam187@gmail.com");
    assert_eq!(saved.name, "chaitanya mandale");
}

#[tokio::test]
async fn subscribe_returns_400_when_data_is_invalid() {
    let test_app = spawn_test_server().await;
    let req_address = test_app.address;
    sleep(Duration::from_secs(2)).await;

    let client = Client::new();
    let url = format!("{}/subscribe", req_address);
    let test_cases = vec![
        ("name=chaitanya%20mandale", "email not sent"),
        ("email=chaitanyam187%40gmail.com", "name not sent"),
        ("", "no data sent at all"),
    ];

    for (body, error_message) in test_cases {
        let response = client
            .post(url.clone())
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to send post request");

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
    let req_address = test_app.address;
    sleep(Duration::from_secs(2)).await;

    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=&email=chaitanyam187%40gmail.com", "empty name"),
        ("name=chaitanya%20mandale&email=", "empty email"),
        (
            "name=chaitanya%20mandale&email=definitely-not-an-email",
            "invalid email",
        ),
    ];

    for (body, description) in test_cases {
        let response = client
            .post(&format!("{}/subscribe", req_address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request");

        assert_eq!(
            400,
            response.status().as_u16(),
            "The API did not return a 400 OK when the payload was {}",
            description
        );
    }
}
