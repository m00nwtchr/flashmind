use axum::{extract::Request, http::StatusCode, middleware::Next, response::Response};
use axum_session::SessionNullPool;
use sea_orm::ActiveValue;
use serde::{Deserialize, Serialize};

use crate::entities::user;

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

impl CurrentUser {
	pub fn update_model(&self, mut model: user::ActiveModel) -> user::ActiveModel {
		model.email = ActiveValue::Set(self.email.clone());
		model.display = ActiveValue::Set(self.display.clone());
		model
	}
}

impl From<CurrentUser> for user::ActiveModel {
	fn from(value: CurrentUser) -> Self {
		Self {
			id: ActiveValue::NotSet,
			sub: ActiveValue::Unchanged(value.subject_id),
			provider: ActiveValue::Unchanged(value.provider),
			// username: ActiveValue::Set(value.username),
			display: ActiveValue::Set(value.display),
			email: ActiveValue::Set(value.email),
		}
	}
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
