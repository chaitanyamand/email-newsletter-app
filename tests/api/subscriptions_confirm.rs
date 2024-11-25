use crate::helpers::spawn_test_server;

#[tokio::test]
async fn confirmations_without_tokens_are_rejected_with_a_400() {
    let test_app = spawn_test_server().await;

    let response = reqwest::get(&format!("{}/subscriptions/confirm", test_app.address))
        .await
        .unwrap();

    assert_eq!(response.status().as_u16(), 400);
}
