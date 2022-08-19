#![allow(dead_code)]

use std::str::FromStr;

use web3::transports::Http;
use web3::types::{H160, U256};
use web3::Web3;

pub(self) const WEB3_URL: &str = "http://localhost:8545";

pub async fn account_balance(account_str: &str) -> Result<U256, web3::Error> {
    let account =
        H160::from_str(account_str).map_err(|_| web3::Error::Decoder(account_str.to_string()))?;
    let http = Http::new(WEB3_URL)?;
    let web3 = Web3::new(http);
    let balance = web3.eth().balance(account, None).await?;
    Ok(balance)
}

pub async fn accounts() -> Result<Vec<H160>, web3::Error> {
    let http = Http::new(WEB3_URL)?;
    let web3 = Web3::new(http);
    let accounts = web3.eth().accounts().await?;
    Ok(accounts)
}
