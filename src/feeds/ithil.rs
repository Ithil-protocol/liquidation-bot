use web3::contract::Contract;
use web3::ethabi::{
    Event,
    EventParam,
    ParamType,
    RawLog,
};
use web3::futures::StreamExt;
use web3::types::{
    FilterBuilder,
    H160,
};

use crate::messages;
use messages::{
    PositionWasClosed,
    PositionWasLiquidated,
    PositionWasOpened,
    Message,
};


fn make_position_was_opened_event() -> Event {
    let position_was_opened_event_params = vec![
        EventParam {
            name: "id".to_string(),
            kind: ParamType::Uint(256),
            indexed: true
        },
        EventParam {
            name: "owner".to_string(),
            kind: ParamType::Address,
            indexed: true
        },
        EventParam {
            name: "owedToken".to_string(),
            kind: ParamType::Address,
            indexed: false
        },
        EventParam {
            name: "heldToken".to_string(),
            kind: ParamType::Address,
            indexed: false
        },
        EventParam {
            name: "collateralToken".to_string(),
            kind: ParamType::Address,
            indexed: false
        },
        EventParam {
            name: "collateral".to_string(),
            kind: ParamType::Uint(256),
            indexed: false
        },
        EventParam {
            name: "principal".to_string(),
            kind: ParamType::Uint(256),
            indexed: false
        },
        EventParam {
            name: "allowance".to_string(),
            kind: ParamType::Uint(256),
            indexed: false
        },
        EventParam {
            name: "fees".to_string(),
            kind: ParamType::Uint(256),
            indexed: false
        },
        EventParam {
            name: "createdAt".to_string(),
            kind: ParamType::Uint(256),
            indexed: false
        },
    ];

    let position_was_opened_event = Event {
        name: "PositionWasOpened".to_string(),
        inputs: position_was_opened_event_params,
        anonymous: false
    };
    
    return position_was_opened_event;
}

fn make_position_was_closed_event() -> Event {
    let position_was_closed_event_params = vec![
        EventParam {
            name: "id".to_string(),
            kind: ParamType::Uint(256),
            indexed: true
        },
    ];

    let position_was_closed_event = Event {
        name: "PositionWasClosed".to_string(),
        inputs: position_was_closed_event_params,
        anonymous: false
    };

    return position_was_closed_event;
}

fn make_position_was_liquidated_event() -> Event {
    let position_was_liquidated_event_params = vec![
        EventParam {
            name: "id".to_string(),
            kind: ParamType::Uint(256),
            indexed: true
        },
    ];

    let position_was_liquidated_event = Event {
        name: "PositionWasLiquidated".to_string(),
        inputs: position_was_liquidated_event_params,
        anonymous: false
    };

    return position_was_liquidated_event;
}

pub async fn run(message_queue: tokio::sync::mpsc::Sender<Message>) -> web3::Result {
    let liquidator_address: [u8; 20] = [0x90, 0xb8, 0x80, 0x04, 0x68, 0xb3, 0xdd, 0x06, 0xf8, 0x24, 0xa5, 0x65, 0x89, 0xdE, 0xda, 0x0A, 0x0b, 0x64, 0x38, 0x68];

    println!("Connecting ...");

    let ws = web3::transports::WebSocket::new("ws://localhost:8545").await?;
    let web3 = web3::Web3::new(ws.clone());

    println!("connected!");
    println!("Configuring contract ...");

    let liquidator_contract = Contract::from_json(
        web3.eth(),
        H160(liquidator_address),
        include_bytes!("../../abi/Liquidator.json"),
    ).unwrap();

    println!("done!");

    let filter = FilterBuilder::default()
        .address(vec![liquidator_contract.address()])
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
                    data: log.data.0
                };
                if let Some(log_type) = log.log_type {
                    match log_type.as_str() {
                        "PositionWasOpened" => {
                            let event_params = position_was_opened_event.parse_log(raw_log).unwrap().params;
                            println!("{:?}", event_params);
                            let message = Message::Event(messages::Event::PositionWasOpened(
                                PositionWasOpened {
                                    id: event_params[0].value.clone().into_uint().unwrap(),
                                    owner: event_params[1].value.clone().into_address().unwrap(),
                                    owed_token: event_params[2].value.clone().into_address().unwrap(),
                                    held_token: event_params[3].value.clone().into_address().unwrap(),
                                    collateral_token: event_params[4].value.clone().into_address().unwrap(),
                                    collateral: event_params[5].value.clone().into_uint().unwrap(),
                                    principal: event_params[6].value.clone().into_uint().unwrap(),
                                    allowance: event_params[7].value.clone().into_uint().unwrap(),
                                    fees: event_params[8].value.clone().into_uint().unwrap(),
                                    created_at: event_params[9].value.clone().into_uint().unwrap(),
                                }
                            ));
                            message_queue.send(message).await;
                        },
                        "PositionWasClosed" => {
                            let event_params = position_was_closed_event.parse_log(raw_log).unwrap().params;
                            println!("{:?}", event_params);
                        },
                        "PositionWasLiquidated" => {
                            let event_params = position_was_liquidated_event.parse_log(raw_log).unwrap().params;
                            println!("{:?}", event_params);
                        },
                        _ => (),
                    }
                }
            }
        }).await;

    sub.unsubscribe().await?;

    Ok(())
}
