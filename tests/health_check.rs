use emailnewsletter::run;
use reqwest::Client;
use serde_json::Value;
use tokio::time::{sleep, Duration};

#[tokio::test]
async fn health_check_works() {
    run();

    sleep(Duration::from_secs(2)).await;

    let client = Client::new();

    let url = "http://127.0.0.1:8080/health_check";

    let response = client
        .get(url)
        .send()
        .await
        .expect("Failed to send request");

    assert_eq!(response.status().as_u16(), 200, "Status code was not 200");

    let json: Value = response.json().await.expect("Failed to parse JSON");

    assert_eq!(json["message"], "server is healthy");
    assert_eq!(json["status"], 200);
}
