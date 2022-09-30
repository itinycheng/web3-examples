use std::{fs::File, io::Read, path::Path, str::FromStr, time::Duration};

use log::info;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use utoipa::ToSchema;
use web3::{
	contract::{Contract, Options},
	ethabi::Token,
	types::{H160, H256},
};

use crate::{
	contracts::ABI,
	error::Error::{self, *},
};

use crate::{ethereum::WEB3, Result};

const CONTRACT_ABI_FORMAT: &str = "./src/contracts/{}.abi";
const CONTRACT_BIN_FORMAT: &str = "./src/contracts/{}.bin";

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
pub(crate) struct InvokeContractRequest {
	contract_name: String,
	contract_address: String,
	from_account: Option<String>,
	fn_name: String,
	#[serde(default)]
	fn_params: JsonValue,
	#[serde(default)]
	confirmations: usize,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
pub(crate) struct DeployContractRequest {
	from_account: String,
	contract_name: String,
	#[serde(default)]
	contract_params: JsonValue,
	#[serde(default)]
	confirmations: usize,
}

pub(crate) async fn deploy_sol_contract(request: DeployContractRequest) -> Result<H160> {
	let account = request.from_account.parse().map_err(|_| InvalidParam(request.from_account))?;

	let abi_url = CONTRACT_ABI_FORMAT.replace("{}", &request.contract_name);
	let bin_url = CONTRACT_BIN_FORMAT.replace("{}", &request.contract_name);

	let contract_abi = read_file(abi_url)?;
	let contract_bin = read_file(bin_url)?;
	let constructor = contract_abi.parse::<ABI>()?.constructor;
	let params = if constructor.is_some() {
		constructor.unwrap().to_params(&request.contract_params)?
	} else {
		vec![]
	};

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

pub(crate) async fn call_sol_contract(request: InvokeContractRequest) -> Result<H256> {
	let abi_url = CONTRACT_ABI_FORMAT.replace("{}", &request.contract_name);
	let contract_abi = read_file(abi_url)?;
	let params = parse_params(&contract_abi, &request)?;

	let address =
		request.contract_address.parse().map_err(|_| InvalidParam(request.contract_address))?;
	let from_account = request
		.from_account
		.as_ref()
		.unwrap()
		.parse()
		.map_err(|_| InvalidParam(request.from_account.unwrap()))?;

	let receipt = Contract::from_json(WEB3.eth(), address, contract_abi.as_bytes())
		.map_err(|e| Web3ContractError(e.into()))?
		.call_with_confirmations(
			&request.fn_name,
			params.as_slice(),
			from_account,
			Options::default(),
			request.confirmations,
		)
		.await?;
	Ok(receipt.transaction_hash)
}

pub(crate) async fn query_sol_contract(request: InvokeContractRequest) -> Result<Vec<String>> {
	let abi_url = CONTRACT_ABI_FORMAT.replace("{}", &request.contract_name);
	let contract_abi = read_file(abi_url)?;
	let params = parse_params(&contract_abi, &request)?;

	let address = H160::from_str(&request.contract_address)
		.map_err(|_| web3::Error::Decoder(request.contract_address))?;
	let from = if request.from_account.is_some() {
		Some(
			request
				.from_account
				.as_ref()
				.unwrap()
				.parse::<H160>()
				.map_err(|_| web3::Error::Decoder(request.from_account.unwrap()))?,
		)
	} else {
		None
	};

	let tokens: Vec<Token> = Contract::from_json(WEB3.eth(), address, contract_abi.as_bytes())
		.map_err(|e| Web3ContractError(e.into()))?
		.query(&request.fn_name, params.as_slice(), from, Options::default(), None)
		.await?;
	let results = tokens.iter().map(|token| format!("{:?}", token)).collect();
	Ok(results)
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

fn parse_params(contract_abi: &String, request: &InvokeContractRequest) -> Result<Vec<Token>> {
	let abi = contract_abi.parse::<ABI>()?;
	let tokens = abi
		.function_map
		.get(&request.fn_name)
		.ok_or(Error::InvalidParam("function not found in abi".to_string()))?
		.to_params(&request.fn_params)?;
	Ok(tokens)
}
