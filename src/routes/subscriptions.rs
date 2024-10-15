use actix_web::{post, web, HttpResponse, Responder};
use chrono::Utc;
use serde::Deserialize;
use sqlx::PgPool;
use unicode_segmentation::UnicodeSegmentation;
use uuid::Uuid;

#[derive(Deserialize)]
struct SubscribeRequest {
    name: String,
    email: String,
}

fn is_valid_name(name: &str) -> bool {
    let is_empty = name.trim().is_empty();

    let is_too_long = name.graphemes(true).count() > 256;

    let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
    let contains_forbidden_characters = name.chars().any(|g| forbidden_characters.contains(&g));

    return !(is_empty || is_too_long || contains_forbidden_characters);
}

#[tracing::instrument(
    name = "Adding new subscriber",
    skip(form,db_pool),
    fields(
        subscriber_email = %form.email,
        subscriber_name = %form.name
    )
)]
#[post("/subscribe")]
pub async fn subscribe(
    form: web::Form<SubscribeRequest>,
    db_pool: web::Data<PgPool>,
) -> impl Responder {
    if !is_valid_name(&form.name) {
        return HttpResponse::BadRequest().finish();
    }

    match insert_subscriber(&form, &db_pool).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[tracing::instrument(name = "Saving new subscriber in the database", skip(form, db_pool))]
pub async fn insert_subscriber(
    form: &SubscribeRequest,
    db_pool: &PgPool,
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        INSERT INTO subscriptions(id,email,name,subscribed_at)
        VALUES($1,$2,$3,$4)
        "#,
        Uuid::new_v4(),
        form.email,
        form.name,
        Utc::now()
    )
    .execute(db_pool)
    .await
    .map_err(|e| {
        tracing::error!("Failed to execute query :{:?}", e);
        e
    })?;
    Ok(())
}
