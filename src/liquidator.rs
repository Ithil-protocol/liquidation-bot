use std::{collections::HashMap, str::FromStr};

use bigdecimal::BigDecimal;
use num_bigfloat::BigFloat;
use num_bigint::BigInt;
use num_traits::FromPrimitive;
use web3::types::{Address, U256};

use crate::events;
use events::{
    BlockHeader, Event, PositionWasClosed, PositionWasLiquidated, PositionWasOpened,
    RiskFactorWasUpdated, Ticker,
};

use crate::types::{CurrencyCode, Liquidation, Pair, Token};

#[derive(Debug, PartialEq)]
pub enum PositionStatus {
    Opened,
    Closed,
    Liquidated,
    LiquidationRequested,
}

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

    pub status: PositionStatus,
}

pub struct Liquidator {
    latest_block: BlockHeader,
    open_positions: HashMap<U256, Position>,
    prices: HashMap<Pair, f64>,
    risk_factors: HashMap<CurrencyCode, web3::types::U256>,
    strategy_address: Address,
    tokens: HashMap<Address, Token>,
}

const VAULT_RESOLUTION: u32 = 10000;
const VAULT_TIME_FEE_PERIOD: u32 = 86400;

impl Liquidator {
    pub fn new(
        latest_block: BlockHeader,
        strategy_address: Address,
        tokens: HashMap<Address, Token>,
    ) -> Self {
        Liquidator {
            latest_block,
            open_positions: HashMap::new(),
            prices: HashMap::new(),
            risk_factors: HashMap::new(),
            strategy_address,
            tokens,
        }
    }

    pub fn run(&mut self, event: &Event) -> Vec<Liquidation> {
        println!("Positions => {:?}", self.open_positions);
        match event {
            Event::BlockHeader(block_header) => self.on_block_header(block_header),
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
        }
    }

    fn on_block_header(&mut self, block_header: &BlockHeader) -> Vec<Liquidation> {
        self.latest_block = block_header.clone();

        vec![]
    }

    fn on_position_opened(&mut self, position_opened: &PositionWasOpened) -> Vec<Liquidation> {
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
            status: PositionStatus::Opened,
        };

        self.open_positions.insert(position.id, position);

        return vec![];
    }

    fn on_position_closed(&mut self, position_closed: &PositionWasClosed) -> Vec<Liquidation> {
        self.open_positions.remove(&position_closed.id);

        return vec![];
    }

    fn on_position_liquidated(
        &mut self,
        position_liquidated: &PositionWasLiquidated,
    ) -> Vec<Liquidation> {
        self.open_positions.remove(&position_liquidated.id);

        vec![]
    }

    fn on_risk_factor_updated(
        &mut self,
        risk_factor_was_updated: &RiskFactorWasUpdated,
    ) -> Vec<Liquidation> {
        let token = self.tokens.get(&risk_factor_was_updated.token).unwrap();

        self.risk_factors.insert(
            token.symbol.clone(),
            risk_factor_was_updated.new_risk_factor,
        );

        vec![]
    }

    fn on_price_ticker(&mut self, ticker: &Ticker) -> Vec<Liquidation> {
        self.prices.insert(ticker.pair.clone(), ticker.price);

        // XXX we assume pairs have the form WBTC-USD
        // We assume all pairs are relative to USD
        let token = self
            .tokens
            .iter()
            .find(|(_, token)| token.symbol == ticker.pair.0)
            .unwrap()
            .0
            .clone();

        let liquidations: Vec<Liquidation> = self
            .open_positions
            .iter()
            .filter(|(_, position)| position.status == PositionStatus::Opened)
            .filter(|(_, position)| position.held_token == token || position.owed_token == token)
            .filter(
                |(_, position)| match self.compute_liquidation_score(position) {
                    Some(liquidation_score) => liquidation_score > BigInt::from(0),
                    None => false,
                },
            )
            .map(|(id, _)| Liquidation {
                strategy: self.strategy_address,
                position_id: id.clone(),
            })
            .collect();

        // Set the position status to liquidation in progress to avoid multiple liquidation
        // attempts on the same position.
        liquidations.iter().for_each(|liquidation| {
            let position = self.open_positions.get(&liquidation.position_id).unwrap();

            self.open_positions
                .insert(
                    position.id,
                    Position {
                        status: PositionStatus::LiquidationRequested,
                        ..*position
                    },
                )
                .unwrap();
        });

        liquidations
    }

    fn compute_pair_risk_factor(
        &self,
        token0: &CurrencyCode,
        token1: &CurrencyCode,
    ) -> Option<U256> {
        let maybe_token_0_risk_factor = self.risk_factors.get(token0);
        let maybe_token_1_risk_factor = self.risk_factors.get(token1);

        match (maybe_token_0_risk_factor, maybe_token_1_risk_factor) {
            (Some(token_0_risk_factor), Some(token_1_risk_factor)) => {
                Some((token_0_risk_factor + token_1_risk_factor) / 2)
            }
            _ => None,
        }
    }

    fn quote(&self, src: &Token, dst: &Token, amount: U256) -> Option<U256> {
        // Returns amount * src_price * 10^(dst_decimals) / (dst_price * 10^(src_decimals))
        // if all prices are present, None otherwise.
        // We also convert prices from f64 to U256 so this computation is not exact.
        let src_token_to_usd = Pair(src.symbol.clone(), CurrencyCode::USD);
        let dst_token_to_usd = Pair(dst.symbol.clone(), CurrencyCode::USD);

        let maybe_src_price = self.prices.get(&src_token_to_usd);
        let maybe_dst_price = self.prices.get(&dst_token_to_usd);

        let quote = match (maybe_src_price, maybe_dst_price) {
            (Some(src_price), Some(dst_price)) => {
                // src_price and dst_price are &f64
                // float64 can go until 2^1023, while the following one goes maximum until 2^(256 * 3) = 2^768
                // therefore, no overflow occurs
                let numerator = (U256::low_u64(&amount) as f64)
                    * src_price
                    * ((10 as i64).pow(dst.decimals as u32) as f64);
                let denominator = dst_price * ((10 as i64).pow(src.decimals as u32) as f64);
                // unfortunately, the maximum precision integer primitive in Rust seems to be i128
                // we cast to that int to reduce overflows (which can occur for very high numerators and low denominators)
                Some(U256::from((numerator / denominator) as i128))
                // let scaled_src_price_float = BigDecimal::from_str(&src_price.to_string()).unwrap()
                //     * BigDecimal::from_str(&BigInt::from(10).pow(dst.decimals as u32).to_string())
                //         .unwrap();
                // println!(
                //     "scaled_src_price_float => {}",
                //     scaled_src_price_float.to_string()
                // );
                // // TODO convert scaled_src_price_float to hex string before U256
                // let scaled_src_price = U256::from_str(&scaled_src_price_float.to_string()).unwrap();
                // // let scaled_src_price = U256::from_str(
                // //     &BigInt::from_str(&(
                // //         BigFloat::from(*src_price)
                // //             * BigFloat::from(10.0).pow(&BigFloat::from(dst.decimals))).int().to_string()
                // //     )
                // //     .unwrap()
                // //     .to_string(),
                // // )
                // // .unwrap();
                // println!("scaled_src_price => {:?}", scaled_src_price);

                // let scaled_dst_price = U256::from_str(
                //     &BigInt::from_f64(dst_price * 10_f64.powf(src.decimals as f64))
                //         .unwrap()
                //         .to_string(),
                // )
                // .unwrap();

                // println!("scaled_src_price => {:?}", scaled_src_price);
                // println!("scaled_dst_price => {:?}", scaled_dst_price);

                // let raw_rate = src_price / dst_price;
                // let scaled_raw_rate = raw_rate * VAULT_RESOLUTION as f64;
                // println!("raw_rate: {}", raw_rate);
                // let rate = if src.decimals == dst.decimals {
                //     U256::from(scaled_raw_rate as u64)
                // } else {
                //     U256::from(scaled_raw_rate as u64).saturating_mul(U256::from(10).pow(U256::from(src.decimals))) / U256::from(10).pow(U256::from(dst.decimals))
                // };

                // println!("amount: {}; rate: {}", amount, rate);

                // Some(amount * rate / VAULT_RESOLUTION)

                // Some(amount * scaled_src_price / scaled_dst_price)
            }
            _ => None,
        };

        quote
    }

    fn compute_liquidation_score(&self, position: &Position) -> Option<BigInt> {
        let collateral_in_owed_token = position.collateral_token != position.held_token;

        let held_token = self.tokens.get(&position.held_token).unwrap();
        let owed_token = self.tokens.get(&position.owed_token).unwrap();

        let pair_risk_factor =
            match self.compute_pair_risk_factor(&held_token.symbol, &owed_token.symbol) {
                Some(pair_risk_factor) => pair_risk_factor,
                None => return None,
            };

        // let position_fees = position.principal * fixedFees;
        // XXX use fake hardcoded value while we wait for this data to be added to token
        // whitelisting events.
        let position_fees = U256::from_str("1").unwrap();

        // XXX field position.fees should be ranamed to position.interest_rate
        let due_fees = position_fees
            + (position.fees
                * (self.latest_block.timestamp - position.created_at)
                * position.principal)
                / (VAULT_TIME_FEE_PERIOD * VAULT_RESOLUTION);

        let held_token = self.tokens.get(&position.held_token).unwrap();
        let owed_token = self.tokens.get(&position.owed_token).unwrap();

        match collateral_in_owed_token {
            true => self
                .quote(held_token, owed_token, position.allowance)
                .map(|expected_tokens| {
                    BigInt::from_str(&expected_tokens.to_string()).unwrap()
                        - (BigInt::from_str(&(position.principal + due_fees).to_string())).unwrap()
                }),
            false => self
                .quote(owed_token, held_token, position.principal + due_fees)
                .map(|expected_tokens| {
                    BigInt::from_str(&position.allowance.to_string()).unwrap()
                        - BigInt::from_str(&expected_tokens.to_string()).unwrap()
                }),
        }
        .map(|pl| {
            BigInt::from_str(&(position.collateral * pair_risk_factor).to_string()).unwrap()
                - pl * VAULT_RESOLUTION
        })
    }
}
