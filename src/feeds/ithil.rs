use std::str::FromStr;
use std::time;

use web3::ethabi::{EventParam, LogParam, ParamType, RawLog};
use web3::futures::StreamExt;
use web3::types::{BlockNumber, Filter, FilterBuilder, Log, H160, H256, U64};

use crate::events;
use events::{PositionWasClosed, PositionWasLiquidated, PositionWasOpened, RiskFactorWasUpdated};

pub struct Configuration {
    pub ethereum_provider_https_url: String,
    pub ethereum_provider_wss_url: String,
    pub margin_trading_strategy_address: String,
}

fn make_position_was_opened_event() -> web3::ethabi::Event {
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

    web3::ethabi::Event {
        name: "PositionWasOpened".to_string(),
        inputs: position_was_opened_event_params,
        anonymous: false,
    }
}

fn make_position_was_closed_event() -> web3::ethabi::Event {
    let position_was_closed_event_params = vec![EventParam {
        name: "id".to_string(),
        kind: ParamType::Uint(256),
        indexed: true,
    }];

    web3::ethabi::Event {
        name: "PositionWasClosed".to_string(),
        inputs: position_was_closed_event_params,
        anonymous: false,
    }
}

fn make_position_was_liquidated_event() -> web3::ethabi::Event {
    let position_was_liquidated_event_params = vec![EventParam {
        name: "id".to_string(),
        kind: ParamType::Uint(256),
        indexed: true,
    }];

    web3::ethabi::Event {
        name: "PositionWasLiquidated".to_string(),
        inputs: position_was_liquidated_event_params,
        anonymous: false,
    }
}

fn make_risk_factor_was_updated_event() -> web3::ethabi::Event {
    let risk_factor_was_updated_event_params = vec![
        EventParam {
            name: "token".to_string(),
            kind: ParamType::Address,
            indexed: true,
        },
        EventParam {
            name: "newRiskFactor".to_string(),
            kind: ParamType::Uint(256),
            indexed: false,
        },
    ];

    web3::ethabi::Event {
        name: "RiskFactorWasUpdated".to_string(),
        inputs: risk_factor_was_updated_event_params,
        anonymous: false,
    }
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
    events::Event::PositionWasClosed(PositionWasClosed {
        id: log_params[0].value.clone().into_uint().unwrap(),
    })
}

fn parse_position_was_liquidated_event(log_params: &Vec<LogParam>) -> events::Event {
    events::Event::PositionWasLiquidated(PositionWasLiquidated {
        id: log_params[0].value.clone().into_uint().unwrap(),
    })
}

fn parse_risk_factor_was_updated_event(log_params: &Vec<LogParam>) -> events::Event {
    events::Event::RiskFactorWasUpdated(RiskFactorWasUpdated {
        token: log_params[0].value.clone().into_address().unwrap(),
        new_risk_factor: log_params[0].value.clone().into_uint().unwrap(),
    })
}

struct EventSignature {
    position_was_opened: H256,
    position_was_closed: H256,
    position_was_liquidated: H256,
    risk_factor_was_updated: H256,
}

pub struct Ithil {
    event_signature: EventSignature,
    events_filter: web3::types::Filter,
    margin_trading_strategy_contract: web3::contract::Contract<web3::transports::WebSocket>,
    web3: web3::Web3<web3::transports::WebSocket>,
}

impl Ithil {
    pub async fn new(configuration: &Configuration) -> Result<Self, web3::Error> {
        let ws = web3::transports::WebSocket::new(&configuration.ethereum_provider_wss_url).await?;
        let web3 = web3::Web3::new(ws.clone());

        println!("Connected!");
        println!("Configuring contract ...");

        let margin_trading_strategy_contract_address =
            H160::from_str(&configuration.margin_trading_strategy_address).unwrap();
        let margin_trading_strategy_contract = web3::contract::Contract::from_json(
            web3.eth(),
            margin_trading_strategy_contract_address,
            include_bytes!("../../deployed/goerli/abi/MarginTradingStrategy.json"),
        )
        .unwrap();

        let position_was_opened_signature = margin_trading_strategy_contract
            .abi()
            .event("PositionWasOpened")
            .unwrap()
            .signature();
        let position_was_closed_signature = margin_trading_strategy_contract
            .abi()
            .event("PositionWasClosed")
            .unwrap()
            .signature();
        let position_was_liquidated_signature = margin_trading_strategy_contract
            .abi()
            .event("PositionWasLiquidated")
            .unwrap()
            .signature();
        let risk_factor_was_updated_signature = margin_trading_strategy_contract
            .abi()
            .event("RiskFactorWasUpdated")
            .unwrap()
            .signature();

        let events_filter = FilterBuilder::default()
            .address(vec![margin_trading_strategy_contract.address()])
            .from_block(BlockNumber::Number(U64::from(10742373 as i32)))
            .to_block(BlockNumber::Latest)
            .topics(
                Some(vec![
                    position_was_opened_signature,
                    position_was_closed_signature,
                    position_was_liquidated_signature,
                    risk_factor_was_updated_signature,
                ]),
                None,
                None,
                None,
            )
            .build();

        Ok(Self {
            event_signature: EventSignature {
                position_was_opened: position_was_opened_signature,
                position_was_closed: position_was_closed_signature,
                position_was_liquidated: position_was_liquidated_signature,
                risk_factor_was_updated: risk_factor_was_updated_signature,
            },
            events_filter,
            margin_trading_strategy_contract,
            web3,
        })
    }

    fn parse_event(&self, log: &Log) -> Option<events::Event> {
        let raw_log = RawLog {
            topics: log.topics.clone(),
            data: log.data.0.clone(),
        };

        match log.topics[0] {
            s if s == self.event_signature.position_was_opened => {
                let log_params = make_position_was_opened_event()
                    .parse_log(raw_log)
                    .unwrap()
                    .params;
                let position_was_opened = parse_position_was_opened_event(&log_params);
                Some(position_was_opened)
            }
            s if s == self.event_signature.position_was_closed => {
                let log_params = make_position_was_closed_event()
                    .parse_log(raw_log)
                    .unwrap()
                    .params;
                let position_was_closed = parse_position_was_closed_event(&log_params);
                Some(position_was_closed)
            }
            s if s == self.event_signature.position_was_liquidated => {
                let log_params = make_position_was_liquidated_event()
                    .parse_log(raw_log)
                    .unwrap()
                    .params;
                let position_was_liquidated = parse_position_was_liquidated_event(&log_params);
                Some(position_was_liquidated)
            }
            s if s == self.event_signature.risk_factor_was_updated => {
                let log_params = make_risk_factor_was_updated_event()
                    .parse_log(raw_log)
                    .unwrap()
                    .params;
                let risk_factor_was_updated = parse_risk_factor_was_updated_event(&log_params);
                Some(risk_factor_was_updated)
            }
            _ => {
                // TODO handle unknown event
                println!("Unparsed data");
                None
            }
        }
    }

    pub async fn bootstrap_positions_state(&self) -> web3::Result<Vec<events::Event>> {
        let logs_filter = self
            .web3
            .eth_filter()
            .create_logs_filter(self.events_filter.clone())
            .await?;

        println!("Polling ...");

        let logs = logs_filter.logs().await?;

        // logs_filter.stream(time::Duration::from_secs(1)).take(4).for_each(|msg| {
        //     println!("msg -> {:?}", msg);
        //     futures_util::future::ready(())
        // }).await;

        println!("Got logs => {:?}", logs);

        let events = logs
            .into_iter()
            .filter_map(|log| self.parse_event(&log))
            .collect();

        Ok(events)
    }

    pub async fn run(
        &self,
        events_queue: tokio::sync::mpsc::Sender<events::Event>,
    ) -> web3::Result {
        let mut sub = self
            .web3
            .eth_subscribe()
            .subscribe_logs(self.events_filter.clone())
            .await?;

        println!("Got subscription id {:?}", sub.id());

        (&mut sub)
            .take(6)
            .for_each(|msg| async {
                if msg.is_ok() {
                    let log = msg.unwrap();
                    println!("{:?}", log);
                    let parsed_event = self.parse_event(&log);
                    if let Some(event) = parsed_event {
                        let _ = events_queue.send(event);
                    }
                }
            })
            .await;

        sub.unsubscribe().await?;

        Ok(())
    }
}
