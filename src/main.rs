use emailnewsletter::configuration::get_configurations;
use emailnewsletter::email_client::EmailClient;
use emailnewsletter::startup::run;
use emailnewsletter::telemetry::{get_subscriber, init_subscriber};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("emailnewsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configurations().expect("Failed to read configuration");
    let db_pool = PgPoolOptions::new().connect_lazy_with(configuration.database.with_db());

    let sender_email = configuration
        .email_client
        .sender()
        .expect("Invalid sender email address");
    let timeout = configuration.email_client.timeout();
    let email_client = EmailClient::new(
        configuration.email_client.base_url,
        sender_email,
        configuration.email_client.authorization_token,
        timeout,
    );

    let listener = TcpListener::bind(format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    ))
    .expect("Failed to bind port 8000");
    run(listener, db_pool, email_client)
        .await
        .expect("Failed to start the server");
    Ok(())
}
