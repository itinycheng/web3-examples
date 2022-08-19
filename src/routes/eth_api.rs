use axum::extract::Path;
use axum::{http::StatusCode, Json};
use serde::Serialize;
use serde_json::{json, Value};

use crate::ethereum::transaction::account_balance;
use crate::ethereum::transaction::accounts;

use super::ResultInfo;

pub(crate) async fn eth_accounts() -> Json<Value> {
    let result_tuple = if let Ok(accounts) = accounts().await {
        (StatusCode::OK, accounts)
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, vec![])
    };

    build_json_value(result_tuple)
}

pub(crate) async fn eth_balance(Path(id): Path<String>) -> Json<Value> {
    let result_tuple = if let Ok(balance) = account_balance(&id).await {
        (StatusCode::OK, balance.as_u128().to_string())
    } else {
        (StatusCode::INTERNAL_SERVER_ERROR, "0".to_string())
    };

    build_json_value(result_tuple)
}

fn build_json_value<T: Serialize>(result_tuple: (StatusCode, T)) -> Json<Value> {
    Json(json!(ResultInfo::new(
        result_tuple.0.as_u16(),
        result_tuple.0.canonical_reason().unwrap().to_string(),
        result_tuple.1
    )))
}
