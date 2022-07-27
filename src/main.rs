use liquidation_bot::utils;

#[tokio::main]
async fn main() {
    let config = utils::load_config().unwrap();

    println!("Tokens => {:?}", config.tokens);

    liquidation_bot::liquidation_bot::run(config).await;
}
