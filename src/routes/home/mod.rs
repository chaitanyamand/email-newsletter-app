use actix_web::{get, http::header::ContentType, HttpResponse};

#[get("/")]
pub async fn get_home() -> HttpResponse {
    HttpResponse::Ok()
        .content_type(ContentType::html())
        .body(include_str!("home.html"))
}
