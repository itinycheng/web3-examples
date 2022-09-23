use std::fmt::Debug;

use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum Error {
	#[error("call web3 api error: {0}")]
	Web3Error(#[from] web3::error::Error),

	#[error("call web3 api error: {0:?}")]
	Web3ContractError(#[from] web3::contract::Error),

	#[error("input parameter is invalid, {0}")]
	InvalidParam(String),

	#[error(transparent)]
	AnyError(#[from] anyhow::Error),
}
