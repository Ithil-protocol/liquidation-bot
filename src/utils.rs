use std::env;
use std::fs;

use crate::feeds;
use crate::liquidation_bot::Configuration;
use crate::types::Token;

pub fn load_token_list() -> Result<Vec<Token>, ()> {
    let file = fs::File::open("deployed/goerli/deployments/tokenlist.json").unwrap();
    let json: serde_json::Value = serde_json::from_reader(file).unwrap();
    if let Some(tokens_array) = json.get("tokens").unwrap().as_array() {
        let tokens: Vec<Token> = tokens_array
            .into_iter()
            .map(|token| serde_json::from_value(token.clone()).unwrap())
            .collect();
        Ok(tokens)
    } else {
        Err(())
    }
}

pub fn load_config() -> Result<Configuration, ()> {
    let file = fs::File::open("deployed/goerli/deployments/addresses.json").unwrap();
    let json: serde_json::Value = serde_json::from_reader(file).unwrap();
    let addresses = json.get("addresses").unwrap();

    let margin_trading_strategy_value = addresses.get("MarginTradingStrategy").unwrap();
    let margin_trading_strategy_address = margin_trading_strategy_value.as_str().unwrap();

    let infura_api_key = env::var("INFURA_API_KEY").unwrap();

    Ok(Configuration {
        ethereum_feed_configuration: feeds::ethereum::Configuration {
            ethereum_provider_wss_url: format!("wss://rinkeby.infura.io/ws/v3/{}", infura_api_key),
        },
        ithil_feed_configuration: feeds::ithil::Configuration {
            ethereum_provider_https_url: format!("https://rinkeby.infura.io/v3/{}", infura_api_key),
            ethereum_provider_wss_url: format!("wss://rinkeby.infura.io/ws/v3/{}", infura_api_key),
            margin_trading_strategy_address: String::from(margin_trading_strategy_address),
        },
        tokens: load_token_list().unwrap(),
    })
}
