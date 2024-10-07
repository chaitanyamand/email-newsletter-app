use emailnewsletter::configuration::get_configurations;
use emailnewsletter::startup::run;
use emailnewsletter::telemetry::{get_subscriber, init_subscriber};
use secrecy::ExposeSecret;
use sqlx::PgPool;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("emailnewsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configurations().expect("Failed to read configuration");
    let db_pool = PgPool::connect_lazy(&configuration.database.connection_string().expose_secret())
        .expect("Failed to connect to postgres");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application.port))
        .expect("Failed to bind port 8000");
    run(listener, db_pool)
        .await
        .expect("Failed to start the server");
    Ok(())
}
