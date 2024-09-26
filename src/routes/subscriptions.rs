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
    let request_id = Uuid::new_v4();

    log::info!(
        "Request Id - {} : Adding a user with email '{}' and name '{}' as a new subscriber.",
        request_id,
        form.email,
        form.name
    );
    log::info!(
        "Request Id - {} : Saving new subscriber details in the database.",
        request_id
    );

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
        Ok(_) => {
            log::info!(
                "Request Id - {} : New subscriber details have been saved.",
                request_id
            );
            return HttpResponse::Ok();
        }

        Err(e) => {
            log::error!(
                "Request Id - {} : Failed to execute query:{:?}",
                request_id,
                e
            );
            return HttpResponse::InternalServerError();
        }
    };
}
