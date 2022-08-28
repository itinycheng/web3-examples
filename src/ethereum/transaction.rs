use std::str::FromStr;

use serde::{Deserialize, Serialize};
use web3::{
	transports::Http,
	types::{TransactionRequest, H160, H256, U256},
	Web3,
};

pub(self) const WEB3_URL: &str = "http://localhost:8545";

#[derive(Debug, Serialize, Deserialize)]
pub struct TxRequest {
	from: String,
	to: String,
	value: u64,
	gas: Option<u64>,
	nonce: Option<u128>,
}

#[inline]
pub async fn account_balance(account_str: &str) -> Result<U256, web3::Error> {
	let account =
		H160::from_str(account_str).map_err(|_| web3::Error::Decoder(account_str.to_string()))?;
	let http = Http::new(WEB3_URL)?;
	let web3 = Web3::new(http);
	let balance = web3.eth().balance(account, None).await?;
	Ok(balance)
}

#[inline]
pub async fn accounts() -> Result<Vec<H160>, web3::Error> {
	let http = Http::new(WEB3_URL)?;
	let web3 = Web3::new(http);
	let accounts = web3.eth().accounts().await?;
	Ok(accounts)
}

#[inline]
pub async fn send_transaction(tx_request: TxRequest) -> Result<H256, web3::Error> {
	let mut request = TransactionRequest::builder()
		.from(tx_request.from.parse().map_err(|_| web3::Error::Decoder(tx_request.from))?)
		.to(tx_request.to.parse().map_err(|_| web3::Error::Decoder(tx_request.to))?)
		.value(U256::exp10(18).overflowing_mul(U256::from(tx_request.value)).0);

	if tx_request.gas.is_some() {
		request = request.gas(U256::from(tx_request.gas.unwrap()));
	}

	if tx_request.nonce.is_some() {
		request = request.nonce(U256::from(tx_request.nonce.unwrap()));
	}

	let http = Http::new(WEB3_URL)?;
	let web3 = Web3::new(http);
	web3.eth().send_transaction(request.build()).await
}
