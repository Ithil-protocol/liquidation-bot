use crate::types::{Exchange, Pair};

use web3::ethabi::Address;
use web3::types::U256;

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
pub struct RiskFactorWasUpdated {
    pub token: Address,
    pub new_risk_factor: U256,
}

#[derive(Debug)]
pub struct Ticker {
    pub exchange: Exchange,
    pub pair: Pair,
    pub price: f64,
}

#[derive(Clone, Debug)]
pub struct BlockHeader {
    pub timestamp: U256,
}

#[derive(Debug)]
pub enum Event {
    BlockHeader(BlockHeader),
    PositionWasOpened(PositionWasOpened),
    PositionWasClosed(PositionWasClosed),
    PositionWasLiquidated(PositionWasLiquidated),
    RiskFactorWasUpdated(RiskFactorWasUpdated),
    Ticker(Ticker),
}
