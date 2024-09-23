use crate::routes::health_check;
use crate::routes::subscribe;
use actix_web::{web, App, HttpServer};
use sqlx::PgConnection;
use std::net::TcpListener;

pub fn run(listener: TcpListener, connection: PgConnection) -> tokio::task::JoinHandle<()> {
    let connection = web::Data::new(connection);

    tokio::spawn(async move {
        HttpServer::new(move || {
            App::new()
                .service(health_check)
                .service(subscribe)
                .app_data(connection.clone())
        })
        .listen(listener)
        .expect("Failed to bind address")
        .run()
        .await
        .expect("Server failed");
    })
}
