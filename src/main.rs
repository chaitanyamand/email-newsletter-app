use std::fmt::{Debug, Display};

use emailnewsletter::configuration::get_configurations;
use emailnewsletter::issue_delivery_worker::run_worker_until_stopped;
use emailnewsletter::startup::Application;
use emailnewsletter::telemetry::{get_subscriber, init_subscriber};
use tokio::task::JoinError;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    let subscriber = get_subscriber("emailnewsletter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    let configuration = get_configurations().expect("Failed to read configuration");
    let application = Application::build(configuration.clone()).await?;
    let application_task = tokio::spawn(application.run_until_stopped());

    let worker_task = tokio::spawn(run_worker_until_stopped(configuration));

    tokio::select! {
        o = application_task => {report_exit("API",o)},
        o = worker_task => {report_exit("Background worker",o)}
    };
    Ok(())
}

fn report_exit(task_name: &str, outcome: Result<Result<(), impl Debug + Display>, JoinError>) {
    match outcome {
        Ok(Ok(())) => {
            tracing::info!("{} has exited", task_name)
        }
        Ok(Err(e)) => {
            tracing::error!(
            error.cause_chain = ?e,
            error.message = %e,
            "{} failed",
            task_name
            )
        }
        Err(e) => {
            tracing::error!(
            error.cause_chain = ?e,
            error.message = %e,
            "{}' task failed to complete",
            task_name
            )
        }
    }
}
