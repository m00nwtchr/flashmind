use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use axum_session::SessionNullPool;

use entity::user;

pub const CURRENT_USER: &str = "current_user";

pub type Session = axum_session::Session<SessionNullPool>;

#[cfg(not(feature = "mock-user"))]
pub(crate) async fn auth(
	session: Session,
	mut req: Request,
	next: Next,
) -> Result<Response, StatusCode> {
	if let Some(current_user) = session.get::<user::Model>(CURRENT_USER) {
		req.extensions_mut().insert(current_user);
		Ok(next.run(req).await)
	} else {
		Err(StatusCode::UNAUTHORIZED)
	}
}

#[cfg(feature = "mock-user")]
pub(crate) async fn auth(
	_session: Session,
	mut req: Request,
	next: Next,
) -> Result<Response, StatusCode> {
	req.extensions_mut().insert(user::Model {
		id: 0,
		sub: "".to_string(),
		provider: "".to_string(),
		display: None,
		email: None,
	});
	Ok(next.run(req).await)
}
