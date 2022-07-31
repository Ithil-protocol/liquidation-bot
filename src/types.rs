use std::str::FromStr;

use serde::{Deserialize, Serialize};
use web3::ethabi::Address;

#[derive(Debug)]
pub enum Exchange {
    Coinbase,
}

#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub enum CurrencyCode {
    BTC,
    DAI,
    ETH,
    USD,
    USDC,
    WETH,
}

impl FromStr for CurrencyCode {
    type Err = ();

    fn from_str(input: &str) -> Result<CurrencyCode, Self::Err> {
        match input {
            "BTC" => Ok(CurrencyCode::BTC),
            "DAI" => Ok(CurrencyCode::DAI),
            "ETH" => Ok(CurrencyCode::ETH),
            "USD" => Ok(CurrencyCode::USD),
            "USDC" => Ok(CurrencyCode::USDC),
            "WETH" => Ok(CurrencyCode::WETH),
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
    pub symbol: CurrencyCode,
}

#[derive(Debug)]
pub struct Liquidation {}
