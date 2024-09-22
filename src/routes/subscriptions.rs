use actix_web::{post, web, Responder};
use serde::Deserialize;

#[derive(Deserialize)]
struct SubscribeRequest {
    name: String,
    email: String,
}

#[post("/subscribe")]
pub async fn subscribe(form: web::Form<SubscribeRequest>) -> impl Responder {
    let form_data = form.into_inner();
    format!("name:{} and email:{}", form_data.name, form_data.email)
}
