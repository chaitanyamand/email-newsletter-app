use crate::{
    configuration::Settings, domain::SubscriberEmail, email_client::EmailClient,
    startup::get_connection_pool,
};
use sqlx::{PgPool, Postgres, Transaction};
use std::time::Duration;
use tracing::{field::display, Span};
use uuid::Uuid;

struct NewsletterIssue {
    title: String,
    text_content: String,
    html_content: String,
}

pub enum ExecutionOutcome {
    TaskCompleted,
    EmptyQueue,
}

#[tracing::instrument(skip_all, fields(newsletter_issue_id = tracing::field::Empty, subscriber_email = tracing::field::Empty),err)]
pub async fn try_execute_task(
    db_pool: &PgPool,
    email_client: &EmailClient,
) -> Result<ExecutionOutcome, anyhow::Error> {
    let task = dequeue_task(db_pool).await?;
    if task.is_none() {
        return Ok(ExecutionOutcome::EmptyQueue);
    }
    let (transaction, issue_id, email) = task.unwrap();

    Span::current()
        .record("newsletter_issue_id", &display(issue_id))
        .record("subscriber_email", &display(&email));
    match SubscriberEmail::parse(email.clone()) {
        Ok(email) => {
            let issue = get_issue(db_pool, issue_id).await?;
            if let Err(e) = email_client
                .send_email(
                    &email,
                    &issue.title,
                    &issue.html_content,
                    &issue.text_content,
                )
                .await
            {
                tracing::error!(error.cause_chain = ?e, error.message = %e, "Failed to deliver issue to a confirmed subscriber. Skipping.");
            }
        }
        Err(e) => {
            tracing::error!(error.cause_chain = ?e, error.message = %e, "Skiping a confirmed subscriber. \
                    Their stored contact details are invalid
                ")
        }
    }
    delete_task(transaction, issue_id, &email).await?;
    Ok(ExecutionOutcome::TaskCompleted)
}

#[tracing::instrument(skip_all)]
async fn get_issue(db_pool: &PgPool, issue_id: Uuid) -> Result<NewsletterIssue, anyhow::Error> {
    let issue = sqlx::query_as!(
        NewsletterIssue,
        r#"
        SELECT title, text_content, html_content
        FROM newsletter_issues 
        WHERE newsletter_issue_id = $1
        "#,
        issue_id
    )
    .fetch_one(db_pool)
    .await?;
    Ok(issue)
}

type PgTransaction = Transaction<'static, Postgres>;

#[tracing::instrument(skip_all)]
async fn dequeue_task(
    db_pool: &PgPool,
) -> Result<Option<(PgTransaction, Uuid, String)>, anyhow::Error> {
    let mut transaction = db_pool.begin().await?;
    let r = sqlx::query!(
        r#"
                                SELECT newsletter_issue_id, subscriber_email
                                FROM issue_delivery_queue FOR UPDATE SKIP LOCKED LIMIT 1"#
    )
    .fetch_optional(&mut transaction)
    .await?;
    if let Some(r) = r {
        Ok(Some((
            transaction,
            r.newsletter_issue_id,
            r.subscriber_email,
        )))
    } else {
        Ok(None)
    }
}

#[tracing::instrument(skip_all)]
async fn delete_task(
    mut transaction: PgTransaction,
    issue_id: Uuid,
    subscriber_email: &str,
) -> Result<(), anyhow::Error> {
    sqlx::query!(r#"DELETE FROM issue_delivery_queue WHERE newsletter_issue_id = $1 AND subscriber_email = $2"#,issue_id,subscriber_email).execute(&mut transaction).await?;
    let _ = transaction.commit().await;
    Ok(())
}

async fn worker_loop(db_pool: PgPool, email_client: EmailClient) -> Result<(), anyhow::Error> {
    loop {
        match try_execute_task(&db_pool, &email_client).await {
            Ok(ExecutionOutcome::EmptyQueue) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
            }
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(1)).await;
            }
            Ok(ExecutionOutcome::TaskCompleted) => {}
        }
    }
}

pub async fn run_worker_until_stopped(configuration: Settings) -> Result<(), anyhow::Error> {
    let connection_pool = get_connection_pool(&configuration.database);
    let email_client = configuration.email_client.client();
    worker_loop(connection_pool, email_client).await
}
