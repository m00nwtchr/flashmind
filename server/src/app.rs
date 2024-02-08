use axum::routing::{delete, get, post, MethodRouter};
use axum::Router;

use crate::db::db;
use crate::api;

pub async fn app() -> Router {
	Router::new()
		.nest("/api/flashcard", api::flashcard())
		.with_state(db().await)
}
