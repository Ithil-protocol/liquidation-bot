use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::events;
use crate::feeds;
use crate::feeds::ithil::Ithil;
use crate::liquidator;
use events::Event;
use liquidator::Liquidator;

pub struct Configuration {
    pub ithil_feed_configuration: crate::feeds::ithil::Configuration,
}

pub async fn run(configuration: Configuration) {
    let (tx, mut rx): (Sender<Event>, Receiver<Event>) = mpsc::channel(32);

    // 0. Set up Ithil Ethereum events feed from Ithil smart contract.
    //    This feed should be used to keep track of open positions and their state.
    let mut liquidator = Liquidator::new();
    let ithil_feed: Ithil = Ithil::new(&configuration.ithil_feed_configuration)
        .await
        .unwrap();

    // 1. Build current position from past events
    ithil_feed.bootstrap_positions_state().await.unwrap().into_iter().for_each(|event| {
        println!("Event => {:?}", event);
        liquidator.run(event);
    });

    println!("Position => {:?}", liquidator.open_positions);

    // 2. Listen for new events
    let tx_ithil = tx.clone();
    tokio::spawn(async move {
        ithil_feed.run(tx_ithil).await.unwrap();
    })
    .await
    .unwrap();

    // 3. Set up Coinbase feed to get real time prices.
    //    Eventually we may use multiple exchanges, including DEXes, to make the bot more robust.
    println!("Setup Coinbase feed ...");
    let tx_coinbase = tx.clone();
    tokio::spawn(async move {
        feeds::coinbase::run(tx_coinbase).await;
    }).await.unwrap();

    // 4. Read all incoming messages from the Ethereum network and price feeds from exchanges,
    //    keep an updated view on open positions and real time prices, trigger liquidation logic.
    println!("Listen for events ...");
    while let Some(event) = rx.recv().await {
        println!("{:?}", event);
        let liquidations = liquidator.run(event);
        println!("{:?}", liquidations);
        // TODO execute liquidations
    }
}
