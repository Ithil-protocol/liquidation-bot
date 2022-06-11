use std::fs;

use crate::liquidation_bot::Configuration;

mod events;
mod feeds;
mod liquidation_bot;
mod liquidator;


fn load_config() -> Result<Configuration, ()> {
    let file = fs::File::open("deployed/latest/addresses.json").unwrap();
    let json: serde_json::Value = serde_json::from_reader(file).unwrap();
    let addresses = json.get("addresses").unwrap();

    let margin_trading_strategy_value = addresses.get("MarginTradingStrategy").unwrap();
    let margin_trading_strategy_address = margin_trading_strategy_value.as_str().unwrap();

    Ok(Configuration {
        ithil_feed_configuration: feeds::ithil::Configuration {
            ethereum_provider_https_url: format!("https://rinkeby.infura.io/v3/{}", INFURA_API_KEY),
            ethereum_provider_wss_url: format!("wss://rinkeby.infura.io/ws/v3/{}", INFURA_API_KEY),
            margin_trading_strategy_address: String::from(margin_trading_strategy_address),
        },
    })
}

#[tokio::main]
async fn main() {
    let config = load_config().unwrap();

    liquidation_bot::run(config).await;
}
