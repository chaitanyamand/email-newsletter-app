use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use tracing::Instrument;
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

    let request_span = tracing::info_span!("Adding a new subscriber",%request_id,subscriber_email = %form.email,subscriber_name = %form.name);
    let query_span = tracing::info_span!("Saving new subscriber details in the database",%request_id,subscriber_email = %form.email,subscriber_name = %form.name);

    let _request_span_guard = request_span.enter();

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
    .instrument(query_span)
    .await
    {
        Ok(_) => {
            tracing::info!(
                "Request Id - {} : New subscriber details have been saved.",
                request_id
            );
            return HttpResponse::Ok();
        }

        Err(e) => {
            tracing::error!(
                "Request Id - {} : Failed to execute query:{:?}",
                request_id,
                e
            );
            return HttpResponse::InternalServerError();
        }
    };
}
