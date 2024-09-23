use emailnewsletter::configuration::get_configurations;
use emailnewsletter::startup::run;
use sqlx::{Connection, PgConnection};
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configurations().expect("Failed to read configuration");
    let connection = PgConnection::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .expect("Failed to bind port 8080");
    run(listener, connection)
        .await
        .expect("Failed to start the server");
    Ok(())
}
