use std::collections::HashMap;

use web3::types::{Address, U256};

use crate::events;
use events::{
    Event, PositionWasClosed, PositionWasLiquidated, PositionWasOpened, RiskFactorWasUpdated,
    Ticker,
};

use crate::types::{CurrencyCode, Liquidation, Pair, Token};

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
    open_positions: HashMap<U256, Position>,
    prices: HashMap<Pair, f64>,
    risk_factors: HashMap<CurrencyCode, web3::types::U256>,
    tokens: HashMap<Address, Token>,
}

impl Liquidator {
    pub fn new(tokens: Vec<Token>) -> Self {
        Liquidator {
            open_positions: HashMap::new(),
            prices: HashMap::new(),
            risk_factors: HashMap::new(),
            tokens,
        }
    }

    pub fn run(&mut self, event: Event) -> Vec<Liquidation> {
        println!("Position => {:?}", self.open_positions);
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
            Event::RiskFactorWasUpdated(risk_factor_was_updated) => {
                self.on_risk_factor_updated(risk_factor_was_updated)
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

        vec![]
    }

    fn on_risk_factor_updated(
        &mut self,
        risk_factor_was_updated: RiskFactorWasUpdated,
    ) -> Vec<Liquidation> {
        let token = self.tokens.get(&risk_factor_was_updated.token).unwrap();

        self.risk_factors.insert(
            token.symbol.clone(),
            risk_factor_was_updated.new_risk_factor,
        );

        vec![]
    }

    fn on_price_ticker(&mut self, ticker: Ticker) -> Vec<Liquidation> {
        self.prices.insert(ticker.pair, ticker.price);

        //        self.open_positions.iter().filter(|(id, position)| Pair(position.held_token, position.owed_token) == ticker.pair).collect();

        return vec![];
    }

    fn compute_pair_risk_factor(&self, token0: &CurrencyCode, token1: &CurrencyCode) -> U256 {
        (self.risk_factors[token0] + self.risk_factors[token1]) / 2
    }

    fn compute_liquidation_score(&self, position: &Position) -> U256 {
        let collateral_in_owed_token = position.collateral_token == position.held_token;

        let held_token = self.tokens.get(&position.held_token).unwrap();
        let owed_token = self.tokens.get(&position.owed_token).unwrap();

        let pair_risk_factor = self.compute_pair_risk_factor(&held_token.symbol, &owed_token.symbol);

        let due_fees = position.fees *
            (position.interest_rate * (block.timestamp - position.created_at) * position.principal) /
            (uint32(VaultMath.TIME_FEE_PERIOD) * VaultMath.RESOLUTION);
    }
}
