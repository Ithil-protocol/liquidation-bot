use web3::futures::StreamExt;
use web3::types::{BlockId, BlockNumber};

use crate::events;

pub struct Configuration {
    pub ethereum_provider_wss_url: String,
}

pub struct EthereumBlocks {
    ethereum_provider_wss_url: String,
}

impl EthereumBlocks {
    pub fn new(configuration: &Configuration) -> Self {
        Self {
            ethereum_provider_wss_url: configuration.ethereum_provider_wss_url.clone(),
        }
    }

    pub async fn get_latest_block(&self) -> web3::Result<events::BlockHeader> {
        let websocket = web3::transports::WebSocket::new(&self.ethereum_provider_wss_url)
            .await
            .unwrap();
        let web3s = web3::Web3::new(websocket);
        let latest_block = web3s
            .eth()
            .block(BlockId::Number(BlockNumber::Latest))
            .await
            .unwrap()
            .unwrap();

        Ok(events::BlockHeader {
            timestamp: latest_block.timestamp,
        })
    }

    pub async fn run(
        &self,
        events_queue: tokio::sync::mpsc::Sender<events::Event>,
    ) -> web3::Result {
        let ws = web3::transports::WebSocket::new(&self.ethereum_provider_wss_url).await?;
        let web3 = web3::Web3::new(ws.clone());

        let feed = web3.eth_subscribe().subscribe_new_heads().await?;

        feed.for_each(|block_header| async {
            println!("BLOCK HEADER => {:?}", block_header);
            if let Ok(block_header) = block_header {
                let block_header_event = events::Event::BlockHeader(events::BlockHeader {
                    timestamp: block_header.timestamp,
                });
                events_queue.send(block_header_event).await.unwrap();
            } else {
                // TODO handle error case
            }

            ()
        })
        .await;

        Ok(())
    }
}
