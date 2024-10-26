use crate::{
    email_client::EmailClient,
    routes::{health_check, subscribe},
};
use actix_web::{web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;
use tracing_actix_web::TracingLogger;

pub fn run(
    listener: TcpListener,
    db_pool: PgPool,
    email_client: EmailClient,
) -> tokio::task::JoinHandle<()> {
    let db_pool = web::Data::new(db_pool);
    let email_client = web::Data::new(email_client);
    tokio::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .wrap(TracingLogger::default())
                .service(health_check)
                .service(subscribe)
                .app_data(db_pool.clone())
                .app_data(email_client.clone())
        })
        .listen(listener)
        .expect("Failed to bind address")
        .run()
        .await
        .expect("Server failed");
    })
}
