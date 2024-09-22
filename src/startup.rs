use crate::routes::health_check;
use crate::routes::subscribe;
use actix_web::{App, HttpServer};
use std::net::TcpListener;

pub fn run(listener: TcpListener) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async {
        HttpServer::new(|| App::new().service(health_check).service(subscribe))
            .listen(listener)
            .expect("Failed to bind address")
            .run()
            .await
            .expect("Server failed");
    })
}
