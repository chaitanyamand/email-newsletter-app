use crate::helpers::spawn_test_server;
use dotenv;
use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};

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
