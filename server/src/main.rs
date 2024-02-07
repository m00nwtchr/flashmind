use tokio::net::TcpListener;

mod app;
mod handler;

#[tokio::main]
async fn main() {
	tracing_subscriber::fmt::init();

	let listener = TcpListener::bind("[::1]:3000").await.unwrap();
	axum::serve(listener, app::router()).await.unwrap();
}