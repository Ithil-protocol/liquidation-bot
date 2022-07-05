use std::env;
use std::fs;

use crate::liquidation_bot::Configuration;
use crate::types::Token;

mod events;
mod feeds;
mod liquidation_bot;
mod liquidator;
pub mod types;

fn load_token_list() -> Result<Vec<Token>, ()> {
    let file = fs::File::open("deployed/latest/tokenlist.json").unwrap();
    let json: serde_json::Value = serde_json::from_reader(file).unwrap();
    if let Some(tokens_array) = json.get("tokens").unwrap().as_array() {
        let tokens: Vec<Token> = tokens_array.into_iter().map(|token| serde_json::from_value(token.clone()).unwrap()).collect();
        Ok(tokens)
    } else {
        Err(())
    }
}

fn load_config() -> Result<Configuration, ()> {
    let file = fs::File::open("deployed/latest/addresses.json").unwrap();
    let json: serde_json::Value = serde_json::from_reader(file).unwrap();
    let addresses = json.get("addresses").unwrap();

    let margin_trading_strategy_value = addresses.get("MarginTradingStrategy").unwrap();
    let margin_trading_strategy_address = margin_trading_strategy_value.as_str().unwrap();

    let infura_api_key = env::var("INFURA_API_KEY").unwrap();

    Ok(Configuration {
        ithil_feed_configuration: feeds::ithil::Configuration {
            ethereum_provider_https_url: format!("https://rinkeby.infura.io/v3/{}", infura_api_key),
            ethereum_provider_wss_url: format!("wss://rinkeby.infura.io/ws/v3/{}", infura_api_key),
            margin_trading_strategy_address: String::from(margin_trading_strategy_address),
        },
    })
}

#[tokio::main]
async fn main() {
    let config = load_config().unwrap();
    let tokens = load_token_list().unwrap();

    println!("Tokens => {:?}", tokens);

    liquidation_bot::run(config).await;
}
