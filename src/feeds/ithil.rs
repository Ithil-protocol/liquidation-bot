use std::str::FromStr;
use std::time;

use web3::contract::Contract;
use web3::contract::tokens::Tokenizable;
use web3::ethabi::{Event, EventParam, LogParam, ParamType, RawLog};
use web3::futures::StreamExt;
use web3::types::{FilterBuilder, H160};

use crate::events;
use events::{PositionWasClosed, PositionWasLiquidated, PositionWasOpened};

pub struct Configuration {
    pub ethereum_provider_https_url: String,
    pub ethereum_provider_wss_url: String,
    pub margin_trading_strategy_address: String,
}

fn make_position_was_opened_event() -> Event {
    let position_was_opened_event_params = vec![
        EventParam {
            name: "id".to_string(),
            kind: ParamType::Uint(256),
            indexed: true,
        },
        EventParam {
            name: "owner".to_string(),
            kind: ParamType::Address,
            indexed: true,
        },
        EventParam {
            name: "owedToken".to_string(),
            kind: ParamType::Address,
            indexed: false,
        },
        EventParam {
            name: "heldToken".to_string(),
            kind: ParamType::Address,
            indexed: false,
        },
        EventParam {
            name: "collateralToken".to_string(),
            kind: ParamType::Address,
            indexed: false,
        },
        EventParam {
            name: "collateral".to_string(),
            kind: ParamType::Uint(256),
            indexed: false,
        },
        EventParam {
            name: "principal".to_string(),
            kind: ParamType::Uint(256),
            indexed: false,
        },
        EventParam {
            name: "allowance".to_string(),
            kind: ParamType::Uint(256),
            indexed: false,
        },
        EventParam {
            name: "fees".to_string(),
            kind: ParamType::Uint(256),
            indexed: false,
        },
        EventParam {
            name: "createdAt".to_string(),
            kind: ParamType::Uint(256),
            indexed: false,
        },
    ];

    let position_was_opened_event = Event {
        name: "PositionWasOpened".to_string(),
        inputs: position_was_opened_event_params,
        anonymous: false,
    };

    return position_was_opened_event;
}

fn make_position_was_closed_event() -> Event {
    let position_was_closed_event_params = vec![EventParam {
        name: "id".to_string(),
        kind: ParamType::Uint(256),
        indexed: true,
    }];

    let position_was_closed_event = Event {
        name: "PositionWasClosed".to_string(),
        inputs: position_was_closed_event_params,
        anonymous: false,
    };

    return position_was_closed_event;
}

fn make_position_was_liquidated_event() -> Event {
    let position_was_liquidated_event_params = vec![EventParam {
        name: "id".to_string(),
        kind: ParamType::Uint(256),
        indexed: true,
    }];

    let position_was_liquidated_event = Event {
        name: "PositionWasLiquidated".to_string(),
        inputs: position_was_liquidated_event_params,
        anonymous: false,
    };

    return position_was_liquidated_event;
}

fn parse_position_was_opened_event(log_params: &Vec<LogParam>) -> events::Event {
    return events::Event::PositionWasOpened(PositionWasOpened {
        id: log_params[0].value.clone().into_uint().unwrap(),
        owner: log_params[1].value.clone().into_address().unwrap(),
        owed_token: log_params[2].value.clone().into_address().unwrap(),
        held_token: log_params[3].value.clone().into_address().unwrap(),
        collateral_token: log_params[4].value.clone().into_address().unwrap(),
        collateral: log_params[5].value.clone().into_uint().unwrap(),
        principal: log_params[6].value.clone().into_uint().unwrap(),
        allowance: log_params[7].value.clone().into_uint().unwrap(),
        fees: log_params[8].value.clone().into_uint().unwrap(),
        created_at: log_params[9].value.clone().into_uint().unwrap(),
    });
}

fn parse_position_was_closed_event(log_params: &Vec<LogParam>) -> events::Event {
    return events::Event::PositionWasClosed(PositionWasClosed {
        id: log_params[0].value.clone().into_uint().unwrap(),
    });
}

fn parse_position_was_liquidated_event(log_params: &Vec<LogParam>) -> events::Event {
    return events::Event::PositionWasLiquidated(PositionWasLiquidated {
        id: log_params[0].value.clone().into_uint().unwrap(),
    });
}

async fn bootstrap_events_state(configuration: &Configuration) {
    let http_transport = web3::transports::Http::new(&configuration.ethereum_provider_https_url).unwrap();
    let web3 = web3::Web3::new(http_transport.clone());

    let margin_trading_strategy_contract_address = H160::from_str(&configuration.margin_trading_strategy_address).unwrap();
    let margin_trading_strategy_contract = Contract::from_json(
        web3.eth(),
        margin_trading_strategy_contract_address,
        include_bytes!("../../deployed/abi/Liquidator.json"),
    )
    .unwrap();

    let filter = FilterBuilder::default()
        .address(vec![margin_trading_strategy_contract.address()])
        .build();

    let filter = web3.eth_filter().create_logs_filter(filter).await.unwrap();

    ethabi::token::Token.String("");
    let events = margin_trading_strategy_contract.events::<str, i32, i32, i32>("PositionWasOpened", 12, 12, 12).await.unwrap();
    // let stream = filter.stream(time::Duration::from_secs(60 * 60 * 24 * 30));
    
    // stream
    //     .for_each(|event| {
    //         println!("Event => {:?}", event);
    //         futures_util::future::ready(())
    // })
    // .await;
}

pub async fn run(
    configuration: Configuration,
    events_queue: tokio::sync::mpsc::Sender<events::Event>,
) -> web3::Result {
    // let liquidator_address: [u8; 20] = [
    //     0x90, 0xb8, 0x80, 0x04, 0x68, 0xb3, 0xdd, 0x06, 0xf8, 0x24, 0xa5, 0x65, 0x89, 0xdE, 0xda,
    //     0x0A, 0x0b, 0x64, 0x38, 0x68,
    // ];
    println!("Connecting ...");

    bootstrap_events_state(&configuration).await;

    let ws = web3::transports::WebSocket::new(&configuration.ethereum_provider_wss_url).await?;
    let web3 = web3::Web3::new(ws.clone());

    println!("connected!");
    println!("Configuring contract ...");

    let margin_trading_strategy_contract_address = H160::from_str(&configuration.margin_trading_strategy_address).unwrap();
    let margin_trading_strategy_contract = Contract::from_json(
        web3.eth(),
        margin_trading_strategy_contract_address,
        include_bytes!("../../deployed/abi/Liquidator.json"),
    )
    .unwrap();

    println!("done!");

    let filter = FilterBuilder::default()
        .address(vec![margin_trading_strategy_contract.address()])
        .build();

    let mut sub = web3.eth_subscribe().subscribe_logs(filter).await?;

    println!("Got subscription id {:?}", sub.id());

    let position_was_opened_event = make_position_was_opened_event();
    let position_was_closed_event = make_position_was_closed_event();
    let position_was_liquidated_event = make_position_was_liquidated_event();

    (&mut sub)
        .take(6)
        .for_each(|msg| async {
            if msg.is_ok() {
                let log = msg.unwrap();
                println!("{:?}", log);
                let raw_log = RawLog {
                    topics: log.topics,
                    data: log.data.0,
                };
                if let Some(log_type) = log.log_type {
                    let parsed_event = match log_type.as_str() {
                        "PositionWasOpened" => {
                            let log_params =
                                position_was_opened_event.parse_log(raw_log).unwrap().params;
                            let position_was_opened = parse_position_was_opened_event(&log_params);
                            Some(position_was_opened)
                        }
                        "PositionWasClosed" => {
                            let log_params =
                                position_was_closed_event.parse_log(raw_log).unwrap().params;
                            let position_was_closed = parse_position_was_closed_event(&log_params);
                            Some(position_was_closed)
                        }
                        "PositionWasLiquidated" => {
                            let log_params = position_was_liquidated_event
                                .parse_log(raw_log)
                                .unwrap()
                                .params;
                            let position_was_liquidated =
                                parse_position_was_liquidated_event(&log_params);
                            Some(position_was_liquidated)
                        }
                        _ => {
                            // TODO Log unhandled event type
                            None
                        }
                    };

                    if let Some(event) = parsed_event {
                        let _ = events_queue.send(event);
                    }
                }
            }
        })
        .await;

    sub.unsubscribe().await?;

    Ok(())
}
