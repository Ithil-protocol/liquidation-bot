use actix_web::{get, web, App, HttpServer, Responder};
use actix_rt::spawn;

use liquidation_bot::utils;

#[get("/status")]
async fn status() -> impl Responder {
    format!("Ok")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config = utils::load_config().unwrap();

    println!("Tokens => {:?}", config.tokens);

    // Start liquidation bot
    actix_rt::spawn(async {
        liquidation_bot::liquidation_bot::run(config).await;
    });

    // Start local webserver for monitoring
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(|| async { "ok" }))
            .service(status)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
