use actix_web::{get, web, App, HttpServer, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct MyResponse {
    message: String,
    status: u32,
}

#[get("/")]
async fn index() -> impl Responder {
    return "Hello world!";
}

#[get("/health_check")]
async fn health_check() -> impl Responder {
    let response = MyResponse {
        message: "server is healthy".to_string(),
        status: 200,
    };
    return web::Json(response);
}

#[get("/{name}")]
async fn hello(name: web::Path<String>) -> impl Responder {
    format!("Hello {}!", &name)
}

pub fn run() -> tokio::task::JoinHandle<()> {
    tokio::spawn(async {
        HttpServer::new(|| {
            App::new()
                .service(index)
                .service(health_check)
                .service(hello)
        })
        .bind(("127.0.0.1", 8080))
        .expect("Failed to bind address")
        .run()
        .await
        .expect("Server failed");
    })
}
