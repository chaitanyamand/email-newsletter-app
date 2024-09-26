use emailnewsletter::configuration::get_configurations;
use emailnewsletter::startup::run;
use env_logger::Env;
use sqlx::PgPool;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info"))
        .filter_module("sqlx", log::LevelFilter::Off)
        .init();

    let configuration = get_configurations().expect("Failed to read configuration");
    let db_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to postgres");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .expect("Failed to bind port 8080");
    run(listener, db_pool)
        .await
        .expect("Failed to start the server");
    Ok(())
}
