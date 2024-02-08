use axum::Router;

use crate::api;
use crate::db::db;

pub async fn app() -> Router {
	Router::new()
		.nest("/api/flashcard", api::flashcard())
		.with_state(db().await)
}
