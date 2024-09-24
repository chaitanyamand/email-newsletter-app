use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Deserialize)]
struct SubscribeRequest {
    name: String,
    email: String,
}

#[post("/subscribe")]
pub async fn subscribe(
    form: web::Form<SubscribeRequest>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    match sqlx::query!(
        r#"
        INSERT INTO subscriptions(id,email,name,subscribed_at)
        VALUES($1,$2,$3,$4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool.get_ref())
    .await
    {
        Ok(_) => return HttpResponse::Ok(),

        Err(e) => {
            println!("Failed to execute query:{}", e);
            return HttpResponse::InternalServerError();
        }
    };
}
