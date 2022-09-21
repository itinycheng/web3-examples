use axum::{extract::Path, http::StatusCode, Json};

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use web3::types::{H160, H256};

use crate::ethereum::{
	account::{account_balance, accounts},
	contract::{
		call_sol_contract, deploy_sol_contract, query_sol_contract, DeployContractRequest,
		InvokeContractRequest,
	},
	transaction::{send_raw_transaction, send_transaction, TxRequest},
};

use super::ResultInfo;

#[utoipa::path(
	get,
	path = "/eth/accounts",
	responses(
		(status = 200, description = "List all accounts successfully"),
		(status = 500, description = "List all accounts failed"),
	)
)]
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

#[utoipa::path(
	get,
	path = "/eth/balance/{id}",
	responses(
		(status = 200, description = "Get account balance successfully"),
		(status = 500, description = "Get account balance failed"),
	),
	params(
		("id" = String, Path, description = "account id")
	),
)]
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

#[utoipa::path(
	post,
	path = "/eth/sendTransaction",
	request_body = TxRequest,
	responses(
		(status = 200, description = "Send transaction successfully"),
		(status = 500, description = "Send transaction failed")
	)
)]
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

#[utoipa::path(
	post,
	path = "/eth/sendRawTransaction",
	request_body = TxRequest,
	responses(
		(status = 200, description = "Send raw transaction successfully"),
		(status = 500, description = "Send raw transaction failed")
	)
)]
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

#[utoipa::path(
	post,
	path = "/eth/contract/deploy",
	request_body = DeployContractRequest,
	responses(
		(status = 200, description = "Deploy contract successfully"),
		(status = 500, description = "Deploy contract failed")
	)
)]
pub(crate) async fn deploy_contract(Json(payload): Json<DeployContractRequest>) -> Json<Value> {
	let result = match deploy_sol_contract(payload).await {
		Ok(addr) => (StatusCode::OK, addr),
		Err(err) => {
			error!(target: "ethereum", "Deploy contract error: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, H160::zero())
		}
	};

	build_json_value(result)
}

#[utoipa::path(
	post,
	path = "/eth/contract/call_fn",
	request_body = InvokeContractRequest,
	responses(
		(status = 200, description = "Call contract function successfully"),
		(status = 500, description = "Call contract function failed")
	)
)]
pub(crate) async fn call_contract(Json(payload): Json<InvokeContractRequest<u64>>) -> Json<Value> {
	let result = match call_sol_contract(payload).await {
		Ok(addr) => (StatusCode::OK, addr),
		Err(err) => {
			error!(target: "ethereum", "call function of contract error: {}", err);
			(StatusCode::INTERNAL_SERVER_ERROR, H256::zero())
		}
	};

	build_json_value(result)
}

#[utoipa::path(
	post,
	path = "/eth/contract/query_fn",
	request_body = InvokeContractRequest<()>,
	responses(
		(status = 200, description = "Query contract function successfully"),
		(status = 500, description = "Query contract function failed")
	)
)]
pub(crate) async fn query_contract(Json(payload): Json<InvokeContractRequest>) -> Json<Value> {
	let result = match query_sol_contract(payload).await {
		Ok(addr) => (StatusCode::OK, addr),
		Err(err) => {
			error!(target: "ethereum", "query function of contract error: {}", err);
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
