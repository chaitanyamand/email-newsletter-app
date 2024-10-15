use dotenv;
use emailnewsletter::{
    configuration::get_configurations,
    configuration::DatabaseSettings,
    startup::run,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use reqwest::Client;
use serde_json::Value;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use tokio::time::{sleep, Duration};
use uuid::Uuid;

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    }
});

async fn spawn_test_server() -> TestApp {
    Lazy::force(&TRACING);

    let listener: TcpListener =
        TcpListener::bind("127.0.0.1:0").expect("Failed to bind random address");
    let port = listener.local_addr().unwrap().port();

    let mut configuration = get_configurations().expect("Failed to read configuration");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let pool = configure_database(&configuration.database).await;

    let _server = run(listener, pool.clone());

    TestApp {
        address: format!("http://127.0.0.1:{}", port),
        db_pool: pool,
    }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres");
    connection
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let connection_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");
    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}

#[tokio::test]
async fn health_check_works() {
    dotenv::dotenv().ok();

    let test_app = spawn_test_server().await;
    let req_address = test_app.address;
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
