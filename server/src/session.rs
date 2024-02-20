use axum::extract::Request;
use axum::http::StatusCode;
use axum::middleware::Next;
use axum::response::Response;
use axum_session::SessionNullPool;
use serde::{Deserialize, Serialize};

pub const OIDC_CSRF_TOKEN: &str = "oidc_csrf_token";
pub const OIDC_NONCE: &str = "oidc_nonce";

pub const CURRENT_USER: &str = "current_user";

pub type Session = axum_session::Session<SessionNullPool>;

#[derive(Serialize, Deserialize, Clone)]
pub struct CurrentUser {
	pub user_id: u32,
	pub subject_id: String,
	pub provider: String,
	pub username: Option<String>,
	pub display: Option<String>,
	pub email: Option<String>,
}

pub(crate) async fn auth(
	session: Session,
	mut req: Request,
	next: Next,
) -> Result<Response, StatusCode> {
	if let Some(current_user) = session.get::<CurrentUser>(CURRENT_USER) {
		req.extensions_mut().insert(current_user);
		Ok(next.run(req).await)
	} else {
		Err(StatusCode::UNAUTHORIZED)
	}
}
