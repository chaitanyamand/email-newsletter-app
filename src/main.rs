use emailnewsletter::configuration::get_configurations;
use emailnewsletter::startup::run;
use std::net::TcpListener;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let configuration = get_configurations().expect("Failed to read configuration");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", configuration.application_port))
        .expect("Failed to bind port 8080");
    run(listener).await.expect("Failed to start the server");
    Ok(())
}
