#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use tokio::net::TcpListener;

use flashmind_server::prelude::*;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();

	let config = AppConfig::from_env();

	let listener = TcpListener::bind(&config.listen_addr).await.unwrap();
	axum::serve(listener, app(config).await).await.unwrap();
}
