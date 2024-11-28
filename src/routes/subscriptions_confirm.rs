use actix_web::{get, web, HttpResponse, ResponseError};
use anyhow::Context;
use sqlx::PgPool;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct Parameters {
    subscription_token: String,
}

#[tracing::instrument(name = "Confirm a pending subscriber", skip(parameters))]
#[get("/subscriptions/confirm")]
pub async fn confirm(
    parameters: web::Query<Parameters>,
    pool: web::Data<PgPool>,
) -> Result<HttpResponse, ConfirmError> {
    let id = get_subscriber_id_from_token(&pool, &parameters.subscription_token)
        .await
        .context("Failed to get subscriber id from subscription token")?;
    match id {
        None => return Ok(HttpResponse::Unauthorized().finish()),
        Some(subscriber_id) => {
            confirm_subscriber(&pool, subscriber_id)
                .await
                .context("Failed to check confirmation token")?;
            Ok(HttpResponse::Ok().finish())
        }
    }
}

#[derive(thiserror::Error)]
pub enum ConfirmError {
    #[error(transparent)]
    UnexpectedError(#[from] anyhow::Error),
}

impl ResponseError for ConfirmError {}

impl std::fmt::Debug for ConfirmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error_chain_fmt(self, f)
    }
}

fn error_chain_fmt(
    e: &impl std::error::Error,
    f: &mut std::fmt::Formatter<'_>,
) -> std::fmt::Result {
    writeln!(f, "{}\n", e)?;
    let mut current = e.source();
    while let Some(cause) = current {
        writeln!(f, "Caused by:\n\t{}", cause)?;
        current = cause.source();
    }
    Ok(())
}

#[tracing::instrument(name = "Mark subscriber as confirmed", skip(subscriber_id, db_pool))]
pub async fn confirm_subscriber(db_pool: &PgPool, subscriber_id: Uuid) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"UPDATE subscriptions SET status = 'confirmed' WHERE id = $1"#,
        subscriber_id,
    )
    .execute(db_pool)
    .await
    .map_err(|e| e)?;
    Ok(())
}

#[tracing::instrument(
    name = "Get subscriber_id from token",
    skip(subscription_token, db_pool)
)]
pub async fn get_subscriber_id_from_token(
    db_pool: &PgPool,
    subscription_token: &str,
) -> Result<Option<Uuid>, sqlx::Error> {
    let result = sqlx::query!(
        r#"SELECT subscriber_id FROM subscription_tokens WHERE subscription_token = $1"#,
        subscription_token,
    )
    .fetch_optional(db_pool)
    .await
    .map_err(|e| e)?;
    Ok(result.map(|r| r.subscriber_id))
}
