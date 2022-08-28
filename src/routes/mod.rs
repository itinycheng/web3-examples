mod eth_api;

use axum::{
	routing::{get, post},
	Router,
};

use serde::{Deserialize, Serialize};

use self::eth_api::{eth_accounts, eth_balance, eth_transaction};

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
		.route("/transaction", post(eth_transaction))
}
