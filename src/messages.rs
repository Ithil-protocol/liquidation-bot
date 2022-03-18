use web3::{types::U256, ethabi::Address};

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
pub enum Event {
    PositionWasOpened(PositionWasOpened),
    PositionWasClosed(PositionWasClosed),
    PositionWasLiquidated(PositionWasLiquidated),
}

#[derive(Debug)]
pub enum Exchange {
    Coinbase,
}

#[derive(Debug)]
pub enum Currency {
    DAI,
    USD,
    USDC,
    WETH,
}

#[derive(Debug)]
pub struct Pair(pub Currency, pub Currency);

#[derive(Debug)]
pub enum FeedMessage {
    Ticker {
        exchange: Exchange,
        pair: Pair,
        price: f64,
    },
}

#[derive(Debug)]
pub enum Message {
    Event(Event),
    FeedMessage(FeedMessage),
}
