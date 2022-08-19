mod eth_api;

use crate::routes::eth_api::{eth_accounts, eth_balance};
use axum::routing::get;
use axum::Router;
use serde::{Deserialize, Serialize};

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
}
