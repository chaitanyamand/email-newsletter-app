use crate::configuration::{DatabaseSettings, Settings};
use crate::routes::{confirm, publish_newsletter};
use crate::{
    email_client::EmailClient,
    routes::{health_check, subscribe},
};
use actix_web::dev::Server;
use actix_web::{web, App, HttpServer};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::io;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub struct Application {
    port: u16,
    server: Server,
}

pub fn run(
    listener: std::net::TcpListener,
    db_pool: sqlx::PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Result<Server, io::Error> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    let base_url = web::Data::new(ApplicationBaseUrl(base_url));
    let server = HttpServer::new(move || {
        App::new()
            .wrap(TracingLogger::default())
            .service(health_check)
            .service(subscribe)
            .service(confirm)
            .service(publish_newsletter)
            .app_data(db_pool.clone())
            .app_data(email_client.clone())
            .app_data(base_url.clone())
    })
    .listen(listener)?
    .run();

    Ok(server)
}

impl Application {
    pub fn build(configuration: Settings) -> Result<Self, std::io::Error> {
        let db_pool = get_connection_pool(&configuration.database);

        let sender_email = configuration
            .email_client
            .sender()
            .expect("Invalid sender email address");
        let email_client = EmailClient::new(
            configuration.email_client.base_url.clone(),
            sender_email,
            configuration.email_client.authorization_token.clone(),
            configuration.email_client.timeout(),
        );

        let address = format!(
            "{}:{}",
            configuration.application.host, configuration.application.port
        );
        let listener = TcpListener::bind(&address)?;
        let port = listener.local_addr().unwrap().port();
        let server = run(
            listener,
            db_pool,
            email_client,
            configuration.application.base_url,
        )?;

        Ok(Self { port, server })
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn run_until_stopped(self) -> Result<(), std::io::Error> {
        self.server.await
    }
}

#[derive(Debug)]
pub struct ApplicationBaseUrl(pub String);

pub fn get_connection_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new().connect_lazy_with(configuration.with_db())
}
