use std::collections::HashMap;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

use liquidation_bot::events::{
    BlockHeader, Event, PositionWasOpened, RiskFactorWasUpdated, Ticker,
};
use liquidation_bot::liquidator::Liquidator;
use liquidation_bot::types::{CurrencyCode, Exchange, Pair, Token};

use web3::types::{Address, U256};

const MARGIN_TRADING_STRATEGY_ADDRESS: &str = "0x09A37C94DF2b68831F0e56b943A416a00E5FA154";

#[test]
fn test_position_is_liquidated_after_loss() {
    let dai_token = Token {
        name: "DAI Stablecoin".to_string(),
        address: "0x4315D935947bf9430152b5e90E0A5675e888Be90"
            .parse()
            .unwrap(),
        decimals: 18,
        symbol: CurrencyCode::DAI,
    };
    let weth_token = Token {
        name: "Wrapped Ether".to_string(),
        address: "0x26CB03b59858dCD2b12F9309de5d1e8269e16F61"
            .parse()
            .unwrap(),
        decimals: 18,
        symbol: CurrencyCode::WETH,
    };
    let wbtc_token = Token {
        name: "Wrapped Bitcoin".to_string(),
        address: "0xc9EA4189848A3518B12808D98bFAD92eF48427A7"
            .parse()
            .unwrap(),
        decimals: 8,
        symbol: CurrencyCode::WBTC,
    };

    let tokens: HashMap<Address, Token> = vec![
        (dai_token.address, dai_token.clone()),
        (weth_token.address, weth_token.clone()),
        (wbtc_token.address, wbtc_token.clone()),
    ]
    .into_iter()
    .collect();

    let latest_block = BlockHeader {
        timestamp: U256::from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        ),
    };

    let margin_trading_strategy_address =
        Address::from_str(MARGIN_TRADING_STRATEGY_ADDRESS).unwrap();
    let mut liquidator = Liquidator::new(latest_block, margin_trading_strategy_address, tokens);

    let events: Vec<Event> = vec![
        Event::RiskFactorWasUpdated(RiskFactorWasUpdated {
            token: weth_token.address.clone(),
            new_risk_factor: U256::from(3000),
        }),
        Event::RiskFactorWasUpdated(RiskFactorWasUpdated {
            token: wbtc_token.address.clone(),
            new_risk_factor: U256::from(2000),
        }),
        Event::RiskFactorWasUpdated(RiskFactorWasUpdated {
            token: dai_token.address.clone(),
            new_risk_factor: U256::from(1000),
        }),
        Event::Ticker(Ticker {
            exchange: Exchange::Coinbase,
            pair: Pair(CurrencyCode::WBTC, CurrencyCode::USD),
            price: 20000.0,
        }),
        Event::Ticker(Ticker {
            exchange: Exchange::Coinbase,
            pair: Pair(CurrencyCode::WETH, CurrencyCode::USD),
            price: 1000.0,
        }),
        Event::Ticker(Ticker {
            exchange: Exchange::Coinbase,
            pair: Pair(CurrencyCode::DAI, CurrencyCode::USD),
            price: 1.0,
        }),
        Event::PositionWasOpened(PositionWasOpened {
            id: U256::from(1),
            owner: "0x643969a6ad1638e646Eda63961E1b54c198d15E3"
                .parse()
                .unwrap(),
            owed_token: dai_token.address.clone(),
            held_token: wbtc_token.address.clone(),
            collateral_token: dai_token.address.clone(),
            collateral: U256::from(100).saturating_mul(U256::from(10).pow(U256::from(18))), // 100 DAI
            principal: U256::from(900).saturating_mul(U256::from(10).pow(U256::from(18))), // 900 DAI
            allowance: U256::from(5000000), // 0.05 WBTC
            fees: U256::from(0),
            created_at: U256::from(1024), // Random block number.
        }),
        // The liquidation price is calculated as follows (ignoring time fees and rounding, therefore it's just an approximation)
        // liquidationPrice = (principal +- collateral*(riskFactor/VaultMath.RESOLUTION) ) * 10^heldDecimals/ (allowance * 10^owedDecimals)
        // in the above, + is for the long case (collateralToken = owedToken) and - for the short case (collateralToken = heldToken)
        // typical RESOLUTION is 10000 but it could change
        Event::Ticker(Ticker {
            exchange: Exchange::Coinbase,
            pair: Pair(CurrencyCode::WBTC, CurrencyCode::USD),
            price: 18300.0,
        }),
    ];

    let liquidations = events.into_iter().fold(vec![], |mut liquidations, event| {
        let mut new_liquidations = liquidator.run(&event);
        liquidations.append(&mut new_liquidations);
        liquidations
    });

    assert_eq!(liquidations.len(), 1);
}
