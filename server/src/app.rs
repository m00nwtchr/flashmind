use axum::Router;
use axum::routing::get;
use crate::handler::hello_world;

pub(crate) fn router() -> Router {
	Router::new().route("/", get(hello_world))
}