use std::str::FromStr;

use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use web3::{
	transports::Http,
	types::{TransactionParameters, TransactionRequest, H160, H256, U256},
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
	secret_key: Option<String>,
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

#[inline]
pub async fn send_raw_transaction(tx_request: TxRequest) -> Result<H256, web3::Error> {
	if tx_request.secret_key.is_none() {
		return Err(web3::Error::InvalidResponse("No secret key found.".to_string()));
	}

	let parameters = TransactionParameters {
		to: Some(tx_request.to.parse().map_err(|_| web3::Error::Decoder(tx_request.to))?),
		value: U256::exp10(18).overflowing_mul(U256::from(tx_request.value)).0,
		..TransactionParameters::default()
	};

	let http = Http::new(WEB3_URL)?;
	let web3 = Web3::new(http);

	let key = SecretKey::from_str(&tx_request.secret_key.unwrap()).unwrap();
	let signed = web3.accounts().sign_transaction(parameters, &key).await?;
	web3.eth().send_raw_transaction(signed.raw_transaction).await
}
