use actix_web::main;
use emailnewsletter::run;

#[main]
async fn main() -> std::io::Result<()> {
    return run().await;
}
