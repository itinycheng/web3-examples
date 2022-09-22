use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use web3::types::{TransactionParameters, TransactionRequest, H256, U256};

use crate::{error::Error::*, Result};

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
pub async fn send_transaction(tx_request: TxRequest) -> Result<H256> {
	let mut request = TransactionRequest::builder()
		.from(tx_request.from.parse().map_err(|_| InvalidParam(tx_request.from))?)
		.to(tx_request.to.parse().map_err(|_| InvalidParam(tx_request.to))?)
		.value(U256::exp10(18).overflowing_mul(U256::from(tx_request.value)).0);

	if tx_request.gas.is_some() {
		request = request.gas(U256::from(tx_request.gas.unwrap()));
	}

	if tx_request.nonce.is_some() {
		request = request.nonce(U256::from(tx_request.nonce.unwrap()));
	}

	let addr = WEB3.eth().send_transaction(request.build()).await?;
	Ok(addr)
}

#[inline]
pub async fn send_raw_transaction(tx_request: TxRequest) -> Result<H256> {
	if tx_request.secret_key.is_none() {
		return Err(InvalidParam("No secret key found.".to_string()));
	}

	let parameters = TransactionParameters {
		to: Some(tx_request.to.parse().map_err(|_| InvalidParam(tx_request.to))?),
		value: U256::exp10(18).overflowing_mul(U256::from(tx_request.value)).0,
		..TransactionParameters::default()
	};

	let secret_key = tx_request.secret_key.unwrap();
	let key = secret_key.parse().map_err(|_| InvalidParam(secret_key))?;
	let signed = WEB3.accounts().sign_transaction(parameters, &key).await?;
	let addr = WEB3.eth().send_raw_transaction(signed.raw_transaction).await?;
	Ok(addr)
}
