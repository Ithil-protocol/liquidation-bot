use std::collections::HashMap;

use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use web3::types::Address;

use crate::events;
use crate::feeds;
use crate::liquidator;
use crate::types::Token;
use events::Event;
use liquidator::Liquidator;

pub struct Configuration {
    pub ethereum_feed_configuration: feeds::ethereum_blocks::Configuration,
    pub ithil_feed_configuration: feeds::ithil::Configuration,
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

    let mut liquidator = Liquidator::new(latest_block, tokens);

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

    // 5. Read all incoming messages from the Ethereum network and price feeds from exchanges,
    //    keep an updated view on open positions and real time prices, trigger liquidation logic.
    println!("Listen for events ...");
    while let Some(event) = rx.recv().await {
        println!("{:?}", event);
        let liquidations = liquidator.run(&event);
        println!("{:?}", liquidations);
        // TODO execute liquidations
    }
}
