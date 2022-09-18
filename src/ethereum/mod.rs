use once_cell::sync::Lazy;
use web3::{transports::Http, Web3};

pub(crate) mod account;
pub(crate) mod contract;
pub(crate) mod transaction;

pub(crate) const WEB3_URL: &str = "http://localhost:8545";

static WEB3: Lazy<Web3<Http>> = Lazy::new(|| {
	let http = Http::new(WEB3_URL).unwrap();
	Web3::new(http)
});
