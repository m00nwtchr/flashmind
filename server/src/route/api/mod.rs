use crate::app::AppState;
use axum::Router;

mod auth;
mod deck;
mod flash_card;
mod oidc;

pub fn router() -> Router<AppState> {
	Router::new()
		// .route("/openapi.json", get(openapi))
		.nest("/deck", deck::router())
		.nest("/flashcard", flash_card::router())
		.nest("/auth", auth::router())
		.nest("/oidc", oidc::router())
}
