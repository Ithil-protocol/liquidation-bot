use std::collections::HashMap;

use web3::types::{Address, U256};

use crate::events;
use events::{
    Currency, Event, Exchange, Pair, PositionWasClosed, PositionWasLiquidated, PositionWasOpened,
    Ticker,
};

#[derive(Debug)]
pub struct Liquidation {}

#[derive(Debug)]
pub struct Position {
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

pub struct Liquidator {
    pub open_positions: HashMap<U256, Position>,
    prices: HashMap<Pair, f64>,
}

impl Liquidator {
    pub fn new() -> Self {
        Liquidator {
            open_positions: HashMap::new(),
            prices: HashMap::new(),
        }
    }

    pub fn run(&mut self, event: Event) -> Vec<Liquidation> {
        return match event {
            Event::PositionWasClosed(position_was_closed) => {
                self.on_position_closed(position_was_closed)
            }
            Event::PositionWasOpened(position_was_opened) => {
                self.on_position_opened(position_was_opened)
            }
            Event::PositionWasLiquidated(position_was_liquidated) => {
                self.on_position_liquidated(position_was_liquidated)
            }
            Event::Ticker(ticker) => self.on_price_ticker(ticker),
        };
    }

    fn on_position_opened(&mut self, position_opened: PositionWasOpened) -> Vec<Liquidation> {
        let position = Position {
            id: position_opened.id,
            owner: position_opened.owner,
            owed_token: position_opened.owed_token,
            held_token: position_opened.held_token,
            collateral_token: position_opened.collateral_token,
            collateral: position_opened.collateral,
            principal: position_opened.principal,
            allowance: position_opened.allowance,
            fees: position_opened.fees,
            created_at: position_opened.created_at,
        };

        self.open_positions.insert(position.id, position);

        return vec![];
    }

    fn on_position_closed(&mut self, position_closed: PositionWasClosed) -> Vec<Liquidation> {
        self.open_positions.remove(&position_closed.id);

        return vec![];
    }

    fn on_position_liquidated(
        &mut self,
        position_liquidated: PositionWasLiquidated,
    ) -> Vec<Liquidation> {
        self.open_positions.remove(&position_liquidated.id);

        return vec![];
    }

    fn on_price_ticker(&mut self, ticker: Ticker) -> Vec<Liquidation> {
        // TODO update currency prices
        // TODO check liquidation rules on all open positions

        return vec![];
    }
}
