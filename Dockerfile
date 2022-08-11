FROM rust:1.62

WORKDIR /usr/src/liquidation-bot

COPY . .

RUN cargo install --profile release --path .

CMD ["liquidation-bot"]
