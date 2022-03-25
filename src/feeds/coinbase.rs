use futures_util::sink::SinkExt;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_aux::prelude::deserialize_number_from_string;
use serde_json::Value;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::protocol;

use crate::events;
use events::{
    Currency,
    Event,
    Exchange,
    Pair,
};

const URL: &str = "wss://ws-feed.exchange.coinbase.com";

#[derive(Debug, Serialize)] struct Channel {
    name: String,
    product_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename = "subscribe")]
struct SubscribeRequest {
    channels: Vec<Channel>,
}

#[derive(Debug, Deserialize, Serialize)]
struct Ticker {
    #[serde(deserialize_with = "deserialize_number_from_string")]
    best_ask: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    best_bid: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    high_24h: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    last_size: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    low_24h: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    open_24h: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    price: f64,
    product_id: String,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    sequence: u64,
    side: String,
    time: String,
    trade_id: i64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    volume_24h: f64,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    volume_30d: f64,
}

#[derive(Debug, Deserialize, Serialize)]
struct Heartbeat {
    last_trade_id: u64,
    product_id: String,
    sequence: u64,
    time: String,
}

pub async fn run(events_queue: tokio::sync::mpsc::Sender<Event>) {
    let url = url::Url::parse(URL).unwrap();

    let (ws_stream, _) = connect_async(url).await.unwrap();

    let (mut ws_write, mut ws_read) = ws_stream.split();

    let subscribe_request = SubscribeRequest {
        channels: vec![
            Channel {
                name: String::from("heartbeat"),
                product_ids: vec![String::from("ETH-USD"), String::from("BTC-USD")]
            },
            Channel {
                name: String::from("ticker"),
                product_ids: vec![String::from("ETH-USD"), String::from("BTC-USD")]
            },
        ],
    };
    let subscribe_request_json = serde_json::to_string(&subscribe_request).unwrap();
    let subscribe_request_result = ws_write.send(
        protocol::Message::text(subscribe_request_json)
    ).await;
    if let Err(result) = subscribe_request_result {
        // TODO write a retry mechanism
        panic!("Error subscribing to Coinbase ws channels: {}", result);
    }
    let subscribe_response = ws_read.next().await.unwrap();
    let res = match subscribe_response {
        Ok(protocol::Message::Text(payload)) => {
            let response: serde_json::Value = serde_json::from_str(&payload).unwrap();
            // TODO parse coinbase subscribe response
            Ok(())
        },
        Ok(_) => Err(()),
        Err(_) => Err(()),
    };

    ws_read.for_each(|message| async {
        match message {
            Ok(protocol::Message::Text(payload)) => {
                let msg: serde_json::Value = serde_json::from_str(&payload).unwrap();
                let msg_type = &msg["type"];
                if let Value::String(t) = msg_type {
                    match t.as_str() {
                        "heartbeat" => {
                            let heartbeat: Heartbeat = serde_json::from_str(&payload).unwrap();
                            println!("HEARTBEAT => {:?}", heartbeat);
                        },
                        "ticker" => {
                            let coinbase_ticker: Ticker = serde_json::from_str(&payload).unwrap();
                            println!("TICKER => {:?}", coinbase_ticker);
                            let ticker = events::Ticker {
                                exchange: Exchange::Coinbase,
                                pair: Pair(Currency::WETH, Currency::USD),
                                price: 12.3,
                            };
                            let event = Event::Ticker(ticker);
                            events_queue.send(event).await;
                        },
                        _ => (),
                    }
                }
            },
            Ok(_) => (),
            Err(_) => (),
        }
    }).await;
}
