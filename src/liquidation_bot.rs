use tokio::sync::mpsc;

use tokio::sync::mpsc::{Receiver, Sender};

use crate::events;
use crate::feeds;
use crate::messages;
use messages::Message;

pub async fn run() {
    let (tx, mut rx): (Sender<Message>, Receiver<Message>) = mpsc::channel(32);

    let tx_ethereum_events = tx.clone();
    let tx_coinbase_feed = tx.clone();

    // 0. Set up Coinbase feed to get real time prices.
    //    Eventually we may use multiple exchanges, including DEXes, to make the bot more robust.
    tokio::spawn(async move {
        feeds::coinbase::run(tx_coinbase_feed).await;
    });

    // Read all Coinbase messages for debugging
    while let Some(message) = rx.recv().await {
        println!("{:?}", message);
    }

    // 1. Set up Ethereum events feed from Ithil smart contract.
    //    This feed should be used to keep track of open positions and their state.
    tokio::spawn(async move {
        events::ithil::run(tx_ethereum_events).await.unwrap();
    }).await.unwrap();

    // 3. Read all incoming messages from the Ethereum network and price feeds from exchanges,
    //    keep an updated view on open positions and real time prices, trigger liquidation logic.
}
