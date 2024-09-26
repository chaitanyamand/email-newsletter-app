use crate::routes::{health_check, subscribe};
use actix_web::{middleware::Logger, web, App, HttpServer};
use sqlx::PgPool;
use std::net::TcpListener;

pub fn run(listener: TcpListener, db_pool: PgPool) -> tokio::task::JoinHandle<()> {
    let db_pool = web::Data::new(db_pool);

    tokio::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .wrap(Logger::default())
                .service(health_check)
                .service(subscribe)
                .app_data(db_pool.clone())
        })
        .listen(listener)
        .expect("Failed to bind address")
        .run()
        .await
        .expect("Server failed");
    })
}
