use axum::{extract::Path, http::StatusCode, Json};

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use web3::types::{H160, H256};

use crate::ethereum::{
	account::{account_balance, accounts},
	storage_contract::{deploy_value_storage, retrieve_value, store_value, ContractRequest},
	transaction::{send_raw_transaction, send_transaction, TxRequest},
};

use super::ResultInfo;

pub(crate) async fn eth_accounts() -> Json<Value> {
	let result = match accounts().await {
		Ok(accounts) => (StatusCode::OK, accounts),
		Err(err) => {
			error!(target: "ethereum", "Get eth accounts error: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, vec![])
		}
	};

	build_json_value(result)
}

pub(crate) async fn eth_balance(Path(id): Path<String>) -> Json<Value> {
	let result = match account_balance(&id).await {
		Ok(balance) => (StatusCode::OK, balance),
		Err(err) => {
			error!(target: "ethereum", "Get account balance error: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, "0".to_string())
		}
	};

	build_json_value(result)
}

pub(crate) async fn eth_transaction(Json(payload): Json<TxRequest>) -> Json<Value> {
	let result = match send_transaction(payload).await {
		Ok(addr) => (StatusCode::OK, addr),
		Err(err) => {
			error!(target: "ethereum", "Send transaction error: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, H256::zero())
		}
	};

	build_json_value(result)
}

pub(crate) async fn eth_raw_transaction(Json(payload): Json<TxRequest>) -> Json<Value> {
	let result = match send_raw_transaction(payload).await {
		Ok(addr) => (StatusCode::OK, addr),
		Err(err) => {
			error!(target: "ethereum", "Send raw transaction error: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, H256::zero())
		}
	};

	build_json_value(result)
}

pub(crate) async fn deploy_contract_value_storage(Path(account_id): Path<String>) -> Json<Value> {
	let result = match deploy_value_storage(account_id).await {
		Ok(addr) => (StatusCode::OK, addr),
		Err(err) => {
			error!(target: "ethereum", "Deploy contract value storage error: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, H160::zero())
		}
	};

	build_json_value(result)
}

pub(crate) async fn call_store_of_value_storage_contract(
	Json(payload): Json<ContractRequest<u64>>,
) -> Json<Value> {
	let result = match store_value(payload).await {
		Ok(addr) => (StatusCode::OK, addr),
		Err(err) => {
			error!(target: "ethereum", "call store method of contract value storage error: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, H256::zero())
		}
	};

	build_json_value(result)
}

pub(crate) async fn call_retrieve_of_value_storage_contract(
	Json(payload): Json<ContractRequest<()>>,
) -> Json<Value> {
	let result = match retrieve_value(payload).await {
		Ok(addr) => (StatusCode::OK, addr),
		Err(err) => {
			error!(target: "ethereum", "call retrieve method of contract value storage error: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, 0)
		}
	};

	build_json_value(result)
}

fn build_json_value<T: Serialize>(result_tuple: (StatusCode, T)) -> Json<Value> {
	Json(json!(ResultInfo::new(
		result_tuple.0.as_u16(),
		result_tuple.0.canonical_reason().unwrap().to_string(),
		result_tuple.1
	)))
}
