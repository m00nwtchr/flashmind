#![warn(clippy::pedantic)]
use axum::http::StatusCode;
use tokio::net::TcpListener;

mod api;
mod app;
mod db;
mod schema;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();

	let listener = TcpListener::bind("[::1]:3000").await.unwrap();
	axum::serve(listener, app::app().await).await.unwrap();
}

fn internal_error<E>(err: E) -> (StatusCode, String)
where
	E: std::error::Error,
{
	(StatusCode::INTERNAL_SERVER_ERROR, err.to_string())
}
