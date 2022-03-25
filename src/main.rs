mod feeds;
mod liquidation_bot;
mod liquidator;
mod events;

#[tokio::main]
async fn main() {
    liquidation_bot::run().await;
}
