use std::{fs::File, io::Read, path::Path, str::FromStr, time::Duration};

use log::info;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use web3::{
	contract::{Contract, Options},
	types::{H160, H256},
};

use crate::ethereum::WEB3;

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
pub(crate) struct InvokeContractRequest<T> {
	contract_name: String,
	contract_address: String,
	from_account: String,
	fn_name: String,
	fn_params: T,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
pub(crate) struct DeployContractRequest {
	from_account: String,
	contract_name: String,
	confirmations: usize,
}

pub(crate) async fn deploy_sol_contract(
	request: DeployContractRequest,
) -> Result<H160, web3::contract::Error> {
	let account = H160::from_str(&request.from_account)
		.map_err(|_| web3::Error::Decoder(request.from_account))?;

	let abi_url = format!("./src/contracts/res/{}.abi", request.contract_name);
	let bin_url = format!("./src/contracts/res/{}.bin", request.contract_name);

	let contract_abi = read_file(abi_url)?;
	let contract_bin = read_file(bin_url)?;

	let address = Contract::deploy(WEB3.eth(), contract_abi.as_bytes())?
		.confirmations(0)
		.poll_interval(Duration::from_secs(10))
		.options(Options::with(|options| options.gas = Some(3_000_000.into())))
		.execute(contract_bin, (), account)
		.await?
		.address();

	info!("Deploy value storage contract, account: {}, addr: {}", account, address);
	Ok(address)
}

pub(crate) async fn call_sol_contract(
	request: InvokeContractRequest<u64>,
) -> Result<H256, web3::contract::Error> {
	let from_account = H160::from_str(&request.from_account)
		.map_err(|_| web3::Error::Decoder(request.from_account))?;
	let address = H160::from_str(&request.contract_address)
		.map_err(|_| web3::Error::Decoder(request.contract_address))?;
	let abi_url = format!("./src/contracts/res/{}.abi", request.contract_name);

	let contract = Contract::from_json(WEB3.eth(), address, read_file(abi_url)?.as_bytes())?;

	let tx_hash = contract
		.call(&request.fn_name, request.fn_params, from_account, Options::default())
		.await?;
	Ok(tx_hash)
}

pub(crate) async fn query_sol_contract(
	request: InvokeContractRequest<()>,
) -> Result<u32, web3::contract::Error> {
	let address = H160::from_str(&request.contract_address)
		.map_err(|_| web3::Error::Decoder(request.contract_address))?;
	let abi_url = format!("./src/contracts/res/{}.abi", request.contract_name);

	let contract = Contract::from_json(WEB3.eth(), address, read_file(abi_url)?.as_bytes())?;

	let value: u32 =
		contract.query(&request.fn_name, request.fn_params, None, Options::default(), None).await?;
	Ok(value)
}

/// ----------------------------------------
/// ------------ private method ------------
/// ----------------------------------------

#[allow(unused)]
fn read_file(path: String) -> Result<String, web3::Error> {
	let contract_string = File::open(Path::new(&path))
		.map(|mut file| {
			let mut buf = String::new();
			file.read_to_string(&mut buf);
			buf
		})
		.map_err(|_| web3::Error::Decoder("err".to_string()))?;

	Ok(contract_string)
}
