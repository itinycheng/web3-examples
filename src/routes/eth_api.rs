use axum::{extract::Path, http::StatusCode, Json};

use log::error;
use serde::Serialize;
use serde_json::{json, Value};
use web3::types::{H256, U256};

use crate::ethereum::transaction::{account_balance, accounts, send_transaction, TxRequest};

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
			(StatusCode::INTERNAL_SERVER_ERROR, U256::zero())
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

fn build_json_value<T: Serialize>(result_tuple: (StatusCode, T)) -> Json<Value> {
	Json(json!(ResultInfo::new(
		result_tuple.0.as_u16(),
		result_tuple.0.canonical_reason().unwrap().to_string(),
		result_tuple.1
	)))
}
