use actix_web::{get, web, App, HttpServer, Responder};
use serde::Serialize;
use std::net::TcpListener;

#[derive(Serialize)]
struct MyResponse {
    message: String,
}

#[get("/")]
async fn index() -> impl Responder {
    return "Hello world!";
}

#[get("/health_check")]
async fn health_check() -> impl Responder {
    let response = MyResponse {
        message: String::from("server is healthy"),
    };
    return web::Json(response);
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

pub fn run(listener: TcpListener) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async {
        HttpServer::new(|| {
            App::new()
                .service(index)
                .service(health_check)
                .service(hello)
        })
        .listen(listener)
        .expect("Failed to bind address")
        .run()
        .await
        .expect("Server failed");
    })
}
