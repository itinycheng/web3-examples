#![feature(absolute_path)]

use std::time::Duration;

use axum::http::StatusCode;

use axum::{error_handling::HandleErrorLayer, routing::get, Router};
use chrono::Local;
use log::info;
use routes::eth_routes;

use tower::{BoxError, ServiceBuilder};

use self::ethereum::{
	contract::{DeployContractRequest, InvokeContractRequest},
	transaction::TxRequest,
};
use tracing_subscriber::{
	fmt, fmt::time::FormatTime, prelude::__tracing_subscriber_SubscriberExt,
	util::SubscriberInitExt, EnvFilter, Layer,
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

mod error;
mod ethereum;
mod routes;

pub type Result<T, E = crate::error::Error> = core::result::Result<T, E>;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::registry()
		.with(fmt::layer().with_timer(LogTimer).with_filter(
			EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
		))
		.init();

	info!("Starting up...");

	let app = Router::new()
		.merge(SwaggerUi::new("/swagger-ui/*tail").url("/api-doc/openapi.json", ApiDoc::openapi()))
		.route("/", get(|| async { "Hello, World!" }))
		.nest("/eth", eth_routes())
		.layer(
			ServiceBuilder::new()
				.layer(HandleErrorLayer::new(|error: BoxError| async move {
					if error.is::<tower::timeout::error::Elapsed>() {
						Ok(StatusCode::REQUEST_TIMEOUT)
					} else {
						Err((
							StatusCode::INTERNAL_SERVER_ERROR,
							format!("Unhandled internal error: {}", error),
						))
					}
				}))
				.timeout(Duration::from_secs(60))
				.into_inner(),
		);

	let addr = if cfg!(debug_assertions) { "127.0.0.1:8080" } else { "0.0.0.0:8080" };
	axum::Server::bind(&addr.parse().unwrap()).serve(app.into_make_service()).await?;

	Ok(())
}

struct LogTimer;

impl FormatTime for LogTimer {
	fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
		write!(w, "{}", Local::now().format("%Y-%m-%d %H:%M:%S%.3f"))
	}
}

#[derive(OpenApi)]
#[openapi(
	paths(
		self::routes::eth_api::eth_accounts,
		self::routes::eth_api::eth_balance,
		self::routes::eth_api::eth_transaction,
		self::routes::eth_api::eth_raw_transaction,
		self::routes::eth_api::deploy_contract,
		self::routes::eth_api::call_contract,
		self::routes::eth_api::query_contract,
	),
	components(schemas(TxRequest, DeployContractRequest, InvokeContractRequest<String>))
)]
struct ApiDoc;
