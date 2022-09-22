use std::ops::Div;
use web3::types::{H160, U256};

use crate::{error::Error::InvalidParam, Result};

use super::WEB3;

#[inline]
pub async fn account_balance(account_str: &str) -> Result<String> {
	let account = account_str
		.parse()
		.map_err(|_| InvalidParam(format!("account: {} parse failed", account_str)))?;
	let balance = WEB3.eth().balance(account, None).await?;
	let eth_amt = balance.div(U256::exp10(18));
	Ok(eth_amt.to_string())
}

#[inline]
pub async fn accounts() -> Result<Vec<H160>> {
	let accounts = WEB3.eth().accounts().await?;
	Ok(accounts)
}
