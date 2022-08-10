use std::collections::HashMap;
use std::str::FromStr;

use secp256k1::SecretKey;

use actix_web::{get, middleware, rt, web, App, HttpRequest, HttpResponse, HttpServer, Responder};

use web3::contract::tokens::Tokenize;
use web3::contract::Options;
use web3::ethabi;
use web3::signing::SecretKeyRef;
use web3::types::H160;

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use web3::types::Address;

use crate::events;
use crate::feeds;
use crate::liquidator;
use crate::types;
use crate::types::Token;
use events::Event;
use liquidator::Liquidator;
use types::Liquidation;

pub struct Configuration {
    pub liquidator_address: String,
    pub ethereum_feed_configuration: feeds::ethereum_blocks::Configuration,
    pub ithil_feed_configuration: feeds::ithil::Configuration,
    pub secret: String,
    pub tokens: Vec<Token>,
}

pub async fn run(configuration: Configuration) {
    let (tx, mut rx): (Sender<Event>, Receiver<Event>) = mpsc::channel(1024);

    let tokens: HashMap<Address, Token> = configuration
        .tokens
        .into_iter()
        .map(|token| (token.address, token))
        .collect();

    // 0. Get block events from Ethereum network
    // This feed helps to keep a synchronized clock with the blockchain.
    let ethereum_blocks_feed: feeds::EthereumBlocks =
        feeds::EthereumBlocks::new(&configuration.ethereum_feed_configuration);

    let latest_block = ethereum_blocks_feed.get_latest_block().await.unwrap();

    let tx_ethereum = tx.clone();
    tokio::spawn(async move {
        ethereum_blocks_feed.run(tx_ethereum).await.unwrap();
    });

    let margin_trading_strategy_address = Address::from_str(
        &configuration
            .ithil_feed_configuration
            .margin_trading_strategy_address,
    )
    .unwrap();
    let mut liquidator = Liquidator::new(latest_block, margin_trading_strategy_address, tokens);

    // 1. Set up Ithil Ethereum events feed from Ithil smart contract.
    //    This feed should be used to keep track of open positions and their state.
    let ithil_feed: feeds::Ithil = feeds::Ithil::new(&configuration.ithil_feed_configuration)
        .await
        .unwrap();

    // 2. Build current position from past events
    ithil_feed
        .bootstrap_positions_state()
        .await
        .unwrap()
        .into_iter()
        .for_each(|event| {
            println!("Event => {:?}", event);
            liquidator.run(&event);
        });

    // 3. Listen for new events
    let tx_ithil = tx.clone();
    tokio::spawn(async move {
        ithil_feed.run(tx_ithil).await.unwrap();
    });

    // 4. Set up Coinbase feed to get real time prices.
    //    Eventually we may use multiple exchanges, including DEXes, to make the bot more robust.
    println!("Setup Coinbase feed ...");
    let tx_coinbase = tx.clone();
    tokio::spawn(async move {
        feeds::coinbase::run(tx_coinbase).await;
    });

    // 5. Set up a thread to execute liquidation commands
    let (liquidation_tx, liquidation_rx): (Sender<Liquidation>, Receiver<Liquidation>) =
        mpsc::channel(1024);
    tokio::spawn(async move {
        liquidate_positions(
            liquidation_rx,
            &configuration
                .ithil_feed_configuration
                .ethereum_provider_wss_url,
            &configuration.liquidator_address,
            &configuration.secret,
        )
        .await
        .unwrap();
    });

    // 6. Read all incoming messages from the Ethereum network and price feeds from exchanges,
    //    keep an updated view on open positions and real time prices, trigger liquidation logic.
    println!("Listen for events ...");
    while let Some(event) = rx.recv().await {
        println!("{:?}", event);
        let liquidations = liquidator.run(&event);
        for liquidation in liquidations {
            liquidation_tx.send(liquidation).await.unwrap();
        }
    }
}

impl Tokenize for Liquidation {
    fn into_tokens(self) -> Vec<ethabi::Token> {
        vec![
            ethabi::Token::Address(self.strategy),
            ethabi::Token::Int(self.position_id),
        ]
    }
}

async fn liquidate_positions(
    mut liquidation_rx: Receiver<Liquidation>,
    ethereum_provider_wss_url: &String,
    liquidator_address: &String,
    secret: &String,
) -> web3::Result {
    let ws = web3::transports::WebSocket::new(ethereum_provider_wss_url).await?;
    let web3 = web3::Web3::new(ws.clone());

    let liquidator_contract_address = H160::from_str(&liquidator_address).unwrap();
    let liquidator_contract = web3::contract::Contract::from_json(
        web3.eth(),
        liquidator_contract_address,
        include_bytes!("../deployed/goerli/abi/Liquidator.json"),
    )
    .unwrap();

    while let Some(liquidation) = liquidation_rx.recv().await {
        println!("LIQUIDATION => {:?}", liquidation);
        let receipt = liquidator_contract
            .signed_call_with_confirmations(
                "liquidateSingle",
                liquidation,
                Options {
                    gas: None,
                    gas_price: None,
                    value: None,
                    nonce: None,
                    condition: None,
                    transaction_type: None,
                    access_list: None,
                    max_fee_per_gas: None,
                    max_priority_fee_per_gas: None,
                },
                3,
                SecretKeyRef::new(&SecretKey::from_str(secret).unwrap()),
            )
            .await
            .unwrap();
        println!("LIQUIDATION RECEIPT => {:?}", receipt);
    }

    Ok(())
}
