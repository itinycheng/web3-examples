mod eth_api;

use axum::{
	routing::{get, post},
	Router,
};

use serde::{Deserialize, Serialize};

use self::eth_api::{
	call_retrieve_of_value_storage_contract, call_store_of_value_storage_contract,
	deploy_contract_value_storage, eth_accounts, eth_balance, eth_raw_transaction, eth_transaction,
};

#[derive(Debug, Serialize, Deserialize)]
struct ResultInfo<T> {
	code: u16,
	msg: String,
	data: T,
}

impl<T> ResultInfo<T> {
	fn new(code: u16, msg: String, data: T) -> ResultInfo<T> {
		ResultInfo { code, msg, data }
	}
}

pub fn eth_routes() -> Router {
	Router::new()
		.route("/accounts", get(eth_accounts))
		.route("/balance/:id", get(eth_balance))
		.route("/sendTransaction", post(eth_transaction))
		.route("/sendRawTransaction", post(eth_raw_transaction))
		.route("/contract/storage/deploy/:account_id", get(deploy_contract_value_storage))
		.route("/contract/storage/store", post(call_store_of_value_storage_contract))
		.route("/contract/storage/retrieve", post(call_retrieve_of_value_storage_contract))
}
