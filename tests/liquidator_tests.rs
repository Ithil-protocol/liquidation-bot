use liquidation_bot::events::{Event, PositionWasOpened, RiskFactorWasUpdated, Ticker};
use liquidation_bot::liquidator::Liquidator;
use liquidation_bot::types::{CurrencyCode, Exchange, Liquidation, Pair, Token};

use web3::types::U256;

#[test]
fn test_position_is_liquidated_after_loss() {
    let dai_token = Token {
        name: "DAI Stablecoin".to_string(),
        address: "0x4315D935947bf9430152b5e90E0A5675e888Be90"
            .parse()
            .unwrap(),
        symbol: CurrencyCode::DAI,
    };
    let weth_token = Token {
        name: "Wrapped Ether".to_string(),
        address: "0x26CB03b59858dCD2b12F9309de5d1e8269e16F61"
            .parse()
            .unwrap(),
        symbol: CurrencyCode::WETH,
    };

    let tokens: Vec<Token> = vec![dai_token.clone(), weth_token.clone()];

    let mut liquidator = Liquidator::new(tokens);

    let events: Vec<Event> = vec![
        Event::RiskFactorWasUpdated(RiskFactorWasUpdated {
            token: weth_token.address.clone(),
            new_risk_factor: U256::from(30),
        }),
        Event::Ticker(Ticker {
            exchange: Exchange::Coinbase,
            pair: Pair(CurrencyCode::WETH, CurrencyCode::USDC),
            price: 1000.0,
        }),
        Event::PositionWasOpened(PositionWasOpened {
            id: U256::from(1),
            owner: "0x643969a6ad1638e646Eda63961E1b54c198d15E3"
                .parse()
                .unwrap(),
            owed_token: dai_token.address.clone(),
            held_token: weth_token.address.clone(),
            collateral_token: dai_token.address.clone(),
            collateral: U256::from(1000),
            principal: U256::from(9000), // XXX convert principal to weth currency amount.
            allowance: U256::from(9000), // XXX same as above.
            fees: U256::from(0),
            created_at: U256::from(1024), // Random block number.
        }),
        Event::Ticker(Ticker {
            exchange: Exchange::Coinbase,
            pair: Pair(CurrencyCode::WETH, CurrencyCode::USDC),
            price: 500.0, // XXX price goes down 50% !!! Need to check price direction here.
        }),
    ];

    let liquidations = events.into_iter().fold(vec![], |mut liquidations, event| {
        let mut new_liquidations = liquidator.run(event);
        liquidations.append(&mut new_liquidations);
        liquidations
    });

    assert_eq!(liquidations.len(), 1);

    // TODO add assertions on liquidation data
}
