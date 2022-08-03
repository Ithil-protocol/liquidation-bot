use std::str::FromStr;

use serde::{Deserialize, Serialize};
use web3::ethabi::Address;
use web3::types::U256;

#[derive(Debug)]
pub enum Exchange {
    Coinbase,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum CurrencyCode {
    DAI,
    USD,
    USDC,
    WBTC,
    WETH,
}

impl FromStr for CurrencyCode {
    type Err = ();

    fn from_str(input: &str) -> Result<CurrencyCode, Self::Err> {
        match input {
            "DAI" => Ok(CurrencyCode::DAI),
            "USD" => Ok(CurrencyCode::USD),
            "USDC" => Ok(CurrencyCode::USDC),
            "WBTC" => Ok(CurrencyCode::WBTC),
            "ETH" => Ok(CurrencyCode::WETH), // XXX Coinbase is very slow/buggy with WETH, so here we use ETH as a proxy for WETH prices.
            _ => Err(()),
        }
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct Pair(pub CurrencyCode, pub CurrencyCode);

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Token {
    pub name: String,
    pub address: Address,
    pub decimals: i32,
    pub symbol: CurrencyCode,
}

#[derive(Debug)]
pub struct Liquidation {
    pub strategy: Address,
    pub position_id: U256,
}
