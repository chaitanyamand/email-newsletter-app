use emailnewsletter::run;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let _ = run().await.expect("Failed to start the server");
    Ok(())
}
