use dotenv;
use emailnewsletter::{configuration::get_configurations, startup::run};
use reqwest::Client;
use serde_json::Value;
use sqlx::PgPool;
use std::net::TcpListener;
use tokio::time::{sleep, Duration};

async fn spawn_test_server() -> String {
    let listener: TcpListener =
        TcpListener::bind("127.0.0.1:0").expect("Failed to bind random address");
    let port = listener.local_addr().unwrap().port();
    let configuration = get_configurations().expect("Failed to read configuration");
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres");

    let _server = run(listener, db_pool);
    format!("http://127.0.0.1:{}", port)
}

#[tokio::test]
async fn health_check_works() {
    dotenv::dotenv().ok();

    let req_address = spawn_test_server().await;
    sleep(Duration::from_secs(2)).await;

    let client = Client::new();
    let url = format!("{}/health_check", req_address);
    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status().as_u16(), 200, "Status code was not 200");
    let json: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(json["message"], "server is healthy");
}

#[tokio::test]
async fn subscribe_returns_200_when_data_is_valid() {
    let req_address = spawn_test_server().await;
    sleep(Duration::from_secs(2)).await;

    let configuration = get_configurations().expect("Failed to retreive configurations");
    let connection_string = configuration.database.connection_string();

    let db_pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to Postgres");

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
    let req_address = spawn_test_server().await;
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
