use std::{fs::File, io::Read, path::Path, str::FromStr, time::Duration};

use log::info;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use web3::{
	contract::{Contract, Options},
	types::{H160, H256},
};

use serde_json::Value as JsonValue;

use crate::{contracts::ABI, error::Error::*};

use crate::{ethereum::WEB3, Result};

const CONTRACT_ABI_FORMAT: &str = "./src/contracts/{}.abi";
const CONTRACT_BIN_FORMAT: &str = "./src/contracts/{}.bin";

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
pub(crate) struct InvokeContractRequest<T = ()> {
	contract_name: String,
	contract_address: String,
	from_account: String,
	fn_name: String,
	fn_params: Option<T>,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
pub(crate) struct DeployContractRequest {
	from_account: String,
	contract_name: String,
	contract_params: JsonValue,
	confirmations: usize,
}

pub(crate) async fn deploy_sol_contract(request: DeployContractRequest) -> Result<H160> {
	let account = request.from_account.parse().map_err(|_| InvalidParam(request.from_account))?;

	let abi_url = CONTRACT_ABI_FORMAT.replace("{}", &request.contract_name);
	let bin_url = CONTRACT_BIN_FORMAT.replace("{}", &request.contract_name);

	let contract_abi = read_file(abi_url)?;
	let contract_bin = read_file(bin_url)?;
	let constructor = contract_abi.parse::<ABI>()?.constructor;
	let params = constructor.to_params(request.contract_params)?;

	let address = Contract::deploy(WEB3.eth(), contract_abi.as_bytes())
		.map_err(|e| AnyError(e.into()))?
		.confirmations(request.confirmations)
		.poll_interval(Duration::from_secs(10))
		.options(Options::with(|options| options.gas = Some(3_000_000.into())))
		.execute(contract_bin, params.as_slice(), account)
		.await
		.map_err(|e| AnyError(e.into()))?
		.address();

	info!("Deploy value storage contract, account: {}, addr: {}", account, address);
	Ok(address)
}

pub(crate) async fn call_sol_contract(request: InvokeContractRequest<u64>) -> Result<H256> {
	let from_account =
		request.from_account.parse().map_err(|_| InvalidParam(request.from_account))?;
	let address =
		request.contract_address.parse().map_err(|_| InvalidParam(request.contract_address))?;
	let abi_url = CONTRACT_ABI_FORMAT.replace("{}", &request.contract_name);

	let contract = Contract::from_json(WEB3.eth(), address, read_file(abi_url)?.as_bytes())
		.map_err(|e| Web3ContractError(e.into()))?;

	let tx_hash = contract
		.call(&request.fn_name, request.fn_params.unwrap(), from_account, Options::default())
		.await?;
	Ok(tx_hash)
}

pub(crate) async fn query_sol_contract(request: InvokeContractRequest<()>) -> Result<u32> {
	let address = H160::from_str(&request.contract_address)
		.map_err(|_| web3::Error::Decoder(request.contract_address))?;
	let abi_url = CONTRACT_ABI_FORMAT.replace("{}", &request.contract_name);

	let contract = Contract::from_json(WEB3.eth(), address, read_file(abi_url)?.as_bytes())
		.map_err(|e| Web3ContractError(e.into()))?;
	let value: u32 = contract.query(&request.fn_name, (), None, Options::default(), None).await?;
	Ok(value)
}

/// ----------------------------------------
/// ------------ private method ------------
/// ----------------------------------------
fn read_file(path: String) -> Result<String, anyhow::Error> {
	let mut file = File::open(Path::new(&path))?;
	let mut buf = String::new();
	let _ = file.read_to_string(&mut buf);
	Ok(buf)
}
