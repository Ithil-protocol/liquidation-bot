use std::str::FromStr;

#[derive(Debug)]
pub enum Exchange {
    Coinbase,
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub enum Currency {
    BTC,
    DAI,
    ETH,
    USD,
    USDC,
    WETH,
}

impl FromStr for Currency {
    type Err = ();

    fn from_str(input: &str) -> Result<Currency, Self::Err> {
        match input {
            "BTC" => Ok(Currency::BTC),
            "DAI" => Ok(Currency::DAI),
            "ETH" => Ok(Currency::ETH),
            "USD" => Ok(Currency::USD),
            "USDC" => Ok(Currency::USDC),
            "WETH" => Ok(Currency::WETH),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Eq, Hash, PartialEq)]
pub struct Pair(pub Currency, pub Currency);
