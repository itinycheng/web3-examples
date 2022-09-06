use std::{str::FromStr, time::Duration};

use log::info;
use serde::{Deserialize, Serialize};
use web3::{
	contract::{Contract, Options},
	types::{H160, H256},
};

use crate::ethereum::WEB3;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ContractRequest<T> {
	address: String,
	from: String,
	params: T,
}

pub(crate) async fn deploy_value_storage(account: String) -> Result<H160, web3::contract::Error> {
	let account = H160::from_str(&account).map_err(|_| web3::Error::Decoder(account))?;

	let address =
		Contract::deploy(WEB3.eth(), include_bytes!("../contracts/res/ValueStorage.abi"))?
			.confirmations(1)
			.poll_interval(Duration::from_secs(10))
			.options(Options::with(|options| options.gas = Some(3_000_000.into())))
			.execute(include_str!("../contracts/res/ValueStorage.bin"), (), account)
			.await?
			.address();

	info!("Deploy value storage contract, account: {}, addr: {}", account, address);
	Ok(address)
}

pub(crate) async fn store_value(
	request: ContractRequest<u64>,
) -> Result<H256, web3::contract::Error> {
	let from_account =
		H160::from_str(&request.from).map_err(|_| web3::Error::Decoder(request.from))?;

	let address =
		H160::from_str(&request.address).map_err(|_| web3::Error::Decoder(request.address))?;
	let contract = Contract::from_json(
		WEB3.eth(),
		address,
		include_bytes!("../contracts/res/ValueStorage.abi"),
	)?;

	let tx_hash = contract.call("store", request.params, from_account, Options::default()).await?;
	Ok(tx_hash)
}

pub(crate) async fn retrieve_value(
	request: ContractRequest<()>,
) -> Result<u32, web3::contract::Error> {
	let address =
		H160::from_str(&request.address).map_err(|_| web3::Error::Decoder(request.address))?;
	let contract = Contract::from_json(
		WEB3.eth(),
		address,
		include_bytes!("../contracts/res/ValueStorage.abi"),
	)?;

	let value: u32 =
		contract.query("retrieve", request.params, None, Options::default(), None).await?;
	Ok(value)
}
