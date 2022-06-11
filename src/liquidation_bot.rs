use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::events;
use crate::feeds;
use crate::liquidator;
use events::Event;
use liquidator::Liquidator;

pub struct Configuration {
    pub ithil_feed_configuration: crate::feeds::ithil::Configuration,
}

pub async fn run(configuration: Configuration) {
    let (tx, mut rx): (Sender<Event>, Receiver<Event>) = mpsc::channel(32);

    let tx_ithil_feed = tx.clone();
    let tx_coinbase_feed = tx.clone();

    // 0. Set up Coinbase feed to get real time prices.
    //    Eventually we may use multiple exchanges, including DEXes, to make the bot more robust.
    // tokio::spawn(async move {
    //     feeds::coinbase::run(tx_coinbase_feed).await;
    // });

    // Read all Coinbase messages for debugging.
    // XXX dead code below this block!
    // while let Some(event) = rx.recv().await {
    //     println!("{:?}", event);
    // }

    // 1. Set up Ithil Ethereum events feed from Ithil smart contract.
    //    This feed should be used to keep track of open positions and their state.
    tokio::spawn(async move {
        feeds::ithil::run(configuration.ithil_feed_configuration, tx_ithil_feed)
            .await
            .unwrap();
    })
    .await
    .unwrap();

    // 2. Read all incoming messages from the Ethereum network and price feeds from exchanges,
    //    keep an updated view on open positions and real time prices, trigger liquidation logic.
    let mut liquidator = Liquidator::new();
    while let Some(event) = rx.recv().await {
        // TODO implement position state, price state, liquidation logic.
        println!("{:?}", event);
        let liquidations = liquidator.run(event);
        println!("{:?}", liquidations);
        // TODO execute liquidations
    }
}
