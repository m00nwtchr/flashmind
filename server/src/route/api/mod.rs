use crate::app::AppState;
use axum::Router;

mod deck;
mod flash_card;

// #[utoipa::path(get, path = "/api/openapi.json",
// responses((status = 200, description = "JSON file", body = ()))
// )]
// async fn openapi() -> Json<utoipa::openapi::OpenApi> {
// 	Json(ApiDoc::openapi())
// }

// #[derive(OpenApi)]
// #[openapi(paths(
// 	openapi,
// 	flash_card::create,
// 	flash_card::get_one,
// 	flash_card::update,
// 	flash_card::delete_card
// ))]
// pub struct ApiDoc;

pub fn router() -> Router<AppState> {
	Router::new()
		// .route("/openapi.json", get(openapi))
		.nest("/deck", deck::router())
		.nest("/flashcard", flash_card::router())
}
