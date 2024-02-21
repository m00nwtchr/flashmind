use axum::http::StatusCode;

pub mod app;
pub mod config;
pub mod db;
pub mod oidc;
pub mod route;
pub mod session;

pub mod prelude {
	pub use crate::{app::app, config::AppConfig};
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
