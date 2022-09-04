use std::time::Duration;

use axum::http::StatusCode;

use axum::{error_handling::HandleErrorLayer, routing::get, Router};
use chrono::Local;
use log::info;
use routes::eth_routes;
use tower::{BoxError, ServiceBuilder};

use tracing_subscriber::{
	fmt, fmt::time::FormatTime, prelude::__tracing_subscriber_SubscriberExt,
	util::SubscriberInitExt, EnvFilter, Layer,
};

mod ethereum;
mod routes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	tracing_subscriber::registry()
		.with(fmt::layer().with_timer(LogTimer).with_filter(
			EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
		))
		.init();

	info!("Starting up...");

	let app = Router::new()
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
