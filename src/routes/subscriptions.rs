use actix_web::{post, web, Responder};
use serde::Deserialize;
use sqlx::PgPool;

#[derive(Deserialize)]
struct SubscribeRequest {
    name: String,
    email: String,
}

#[post("/subscribe")]
pub async fn subscribe(
    form: web::Form<SubscribeRequest>,
    _connection: web::Data<PgPool>,
) -> impl Responder {
    format!("Received : name :{} and email:{}", form.name, form.email)
}
