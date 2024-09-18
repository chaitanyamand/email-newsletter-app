use reqwest::Client;
use serde_json::Value;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn health_check_works() {
    let client = Client::new();
    let url = "http://127.0.0.1:8080/health_check";
    let response = timeout(Duration::from_secs(5), client.get(url).send()).await;
    let response = match response {
        Ok(res) => res,
        Err(_) => panic!("Request to health_check endpoint timed out"),
    };

    let response = response
        .map_err(|err| panic!("Request failed: {}", err))
        .unwrap();

    assert_eq!(response.status().as_u16(), 200, "Status code was not 200");
    let json: Value = response.json().await.expect("Failed to parse JSON");
    assert_eq!(json["message"], "server is healthy");
    assert_eq!(json["status"], 200);
}
