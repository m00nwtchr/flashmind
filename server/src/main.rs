#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

use axum::http::StatusCode;
use tokio::net::TcpListener;

use crate::config::AppConfig;

mod app;
mod config;
mod data;
mod db;
mod entities;
mod oidc;
mod route;
mod session;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();

	let config = AppConfig::from_env();

	let listener = TcpListener::bind(&config.listen_addr).await.unwrap();
	axum::serve(listener, app::app(config).await).await.unwrap();
}

fn status_code<E>(status_code: StatusCode) -> impl FnOnce(E) -> (StatusCode, String)
where
	E: std::error::Error,
{
	move |err| (status_code, err.to_string())
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
	E: std::error::Error,
{
	(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
