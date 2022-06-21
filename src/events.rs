use std::str::FromStr;

use web3::{ethabi::Address, types::U256};

#[derive(Debug)]
pub struct PositionWasOpened {
    pub id: U256,
    pub owner: Address,
    pub owed_token: Address,
    pub held_token: Address,
    pub collateral_token: Address,
    pub collateral: U256,
    pub principal: U256,
    pub allowance: U256,
    pub fees: U256,
    pub created_at: U256,
}

#[derive(Debug)]
pub struct PositionWasClosed {
    pub id: U256,
}

#[derive(Debug)]
pub struct PositionWasLiquidated {
    pub id: U256,
}

#[derive(Debug)]
pub enum Exchange {
    Coinbase,
}

#[derive(Debug)]
pub enum Currency {
    BTC,
    DAI,
    ETH,
    USD,
    USDC,
    WETH,
}

impl FromStr for Currency {
    type Err = ();

    fn from_str(input: &str) -> Result<Currency, Self::Err> {
        match input {
            "BTC" => Ok(Currency::BTC),
            "DAI" => Ok(Currency::DAI),
            "ETH" => Ok(Currency::ETH),
            "USD" => Ok(Currency::USD),
            "USDC" => Ok(Currency::USDC),
            "WETH" => Ok(Currency::WETH),
            _ => Err(()),
        }
    }
}

#[derive(Debug)]
pub struct Pair(pub Currency, pub Currency);

#[derive(Debug)]
pub struct Ticker {
    pub exchange: Exchange,
    pub pair: Pair,
    pub price: f64,
}

#[derive(Debug)]
pub enum Event {
    PositionWasOpened(PositionWasOpened),
    PositionWasClosed(PositionWasClosed),
    PositionWasLiquidated(PositionWasLiquidated),
    Ticker(Ticker),
}
