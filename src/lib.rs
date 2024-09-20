use actix_web::{get, post, web, App, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use std::net::TcpListener;

#[derive(Serialize)]
struct MyResponse {
    message: String,
}

#[derive(Deserialize)]
struct SubscribeRequest {
    name: String,
    email: String,
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

#[post("/subscribe")]
async fn subscibe(form: web::Form<SubscribeRequest>) -> impl Responder {
    let form_data = form.into_inner();
    format!("name:{} and email:{}", form_data.name, form_data.email)
}

pub fn run(listener: TcpListener) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async {
        HttpServer::new(|| {
            App::new()
                .service(index)
                .service(health_check)
                .service(hello)
                .service(subscibe)
        })
        .listen(listener)
        .expect("Failed to bind address")
        .run()
        .await
        .expect("Server failed");
    })
}
