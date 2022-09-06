use std::{ops::Div, str::FromStr};
use web3::types::{H160, U256};

use super::WEB3;

#[inline]
pub async fn account_balance(account_str: &str) -> Result<String, web3::Error> {
	let account =
		H160::from_str(account_str).map_err(|_| web3::Error::Decoder(account_str.to_string()))?;

	let balance = WEB3.eth().balance(account, None).await?;
	let eth_amt = balance.div(U256::exp10(18));
	Ok(eth_amt.to_string())
}

#[inline]
pub async fn accounts() -> Result<Vec<H160>, web3::Error> {
	let accounts = WEB3.eth().accounts().await?;
	Ok(accounts)
}
