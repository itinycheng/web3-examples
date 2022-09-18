use std::str::FromStr;

use secp256k1::SecretKey;
use serde::{Deserialize, Serialize};
use web3::types::{TransactionParameters, TransactionRequest, H256, U256};
use utoipa::ToSchema;

use super::WEB3;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct TxRequest {
	from: String,
	to: String,
	value: u64,
	gas: Option<u64>,
	nonce: Option<u128>,
	secret_key: Option<String>,
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

	WEB3.eth().send_transaction(request.build()).await
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

	let key = SecretKey::from_str(&tx_request.secret_key.unwrap()).unwrap();
	let signed = WEB3.accounts().sign_transaction(parameters, &key).await?;
	WEB3.eth().send_raw_transaction(signed.raw_transaction).await
}
