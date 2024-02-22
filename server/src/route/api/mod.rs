use crate::app::AppState;
use axum::Router;

mod deck;
mod flash_card;

pub use deck::router as deck;
pub use flash_card::router as flashcard;
pub fn router() -> Router<AppState> {
	Router::new()
		// .route("/openapi.json", get(openapi))
		.nest("/deck", deck::router())
		.nest("/flashcard", flash_card::router())
}
