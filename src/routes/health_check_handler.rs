use actix_web::{get, web, Responder};
use serde::Serialize;

#[derive(Serialize)]
pub struct MyResponse {
    pub message: String,
}

#[get("/health_check")]
pub async fn health_check() -> impl Responder {
    let response = MyResponse {
        message: String::from("server is healthy"),
    };
    return web::Json(response);
}
