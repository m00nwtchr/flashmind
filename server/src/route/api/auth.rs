use axum::{
	http::{header::LOCATION, StatusCode},
	middleware,
	response::IntoResponse,
	routing::{get, post},
	Extension, Json, Router,
};

use crate::{
	app::AppState,
	session,
	session::{CurrentUser, Session},
};

async fn logout(session: Session) -> Result<impl IntoResponse, (StatusCode, String)> {
	session.destroy();
	Ok((StatusCode::SEE_OTHER, [(LOCATION, "/".to_string())]))
}

async fn user(Extension(user): Extension<CurrentUser>) -> Json<CurrentUser> {
	Json(user)
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/user", get(user))
		.route("/logout", post(logout))
		.route_layer(middleware::from_fn(session::auth))
}
