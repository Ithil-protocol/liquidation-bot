mod feeds;
mod events;
mod liquidation_bot;
mod messages;

#[tokio::main]
async fn main() {
    liquidation_bot::run().await;
}
