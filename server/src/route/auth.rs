use axum::extract::State;
use axum::routing::post;
use axum::{
	extract::Query,
	http::{header::LOCATION, StatusCode},
	middleware,
	response::IntoResponse,
	routing::get,
	Extension, Json, Router,
};
use openidconnect::{
	core::CoreResponseType, reqwest::async_http_client, AuthenticationFlow, AuthorizationCode,
	CsrfToken, Nonce, RequestTokenError, Scope, TokenResponse,
};
use sea_orm::ActiveValue::Set;
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter};
use serde::Deserialize;

use crate::entities::{prelude as entity, user};
use crate::session::{CurrentUser, Session, CURRENT_USER, OIDC_NONCE};
use crate::{app::AppState, internal_error, oidc::OIDCProvider, session, session::OIDC_CSRF_TOKEN};

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
	code: AuthorizationCode,
	state: CsrfToken,
}

async fn provider(
	OIDCProvider(_, client): OIDCProvider,
	session: Session,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let (authorize_url, csrf_state, nonce) = client
		.authorize_url(
			AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
			CsrfToken::new_random,
			Nonce::new_random,
		)
		.add_scope(Scope::new("email".to_string()))
		.add_scope(Scope::new("profile".to_string()))
		.url();
	session.set(OIDC_CSRF_TOKEN, csrf_state);
	session.set(OIDC_NONCE, nonce);

	Ok((StatusCode::FOUND, [(LOCATION, authorize_url.to_string())]))
}

fn if_empty(str: String) -> Option<String> {
	if str.is_empty() {
		None
	} else {
		Some(str)
	}
}

async fn provider_callback(
	OIDCProvider(provider, client): OIDCProvider,
	Query(query): Query<AuthRequest>,
	session: Session,
	State(db): State<DatabaseConnection>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let mis_st = || {
		(
			StatusCode::BAD_REQUEST,
			"Missing state in session".to_string(),
		)
	};
	let csrf_token: CsrfToken = session.get(OIDC_CSRF_TOKEN).ok_or_else(mis_st)?;
	let nonce: Nonce = session.get(OIDC_NONCE).ok_or_else(mis_st)?;

	if !query.state.secret().eq(csrf_token.secret()) {
		return Err((StatusCode::FORBIDDEN, "Forbidden".to_string()));
	};
	let token_response = client
		.exchange_code(query.code)
		.request_async(async_http_client)
		.await
		.map_err(|err| match err {
			RequestTokenError::ServerResponse(_) => (
				StatusCode::BAD_REQUEST,
				"Token endpoint returned error response".to_string(),
			),
			RequestTokenError::Request(_) => (
				StatusCode::GATEWAY_TIMEOUT,
				"Failed to contact token endpoint".to_string(),
			),
			_ => internal_error(err),
		})?;
	let id_token = token_response.id_token().unwrap();
	let claims = id_token
		.claims(&client.id_token_verifier(), &nonce)
		.unwrap();
	let sub = claims.subject();

	let res = entity::User::find()
		.filter(user::Column::Sub.eq(sub.as_str()))
		.filter(user::Column::Provider.eq(&provider))
		.one(&db)
		.await
		.map_err(internal_error)?;

	let user = match res {
		None => {
			let mut u = CurrentUser {
				user_id: 0,
				subject_id: sub.to_string(),
				provider,
				username: claims.preferred_username().map(|c| c.to_string()),
				display: claims
					.name()
					.and_then(|c| c.get(None))
					.map(|c| c.to_string()),
				email: claims.email().map(|c| c.to_string()),
			};

			// Create new user
			u.user_id = entity::User::insert(user::ActiveModel {
				sub: Set(u.subject_id.clone()),
				provider: Set(u.provider.clone()),
				email: Set(u.email.clone()),
				display: Set(u.display.clone()),
				..Default::default()
			})
			.exec(&db)
			.await
			.map_err(internal_error)?
			.last_insert_id;

			u
		}
		Some(user) => CurrentUser {
			user_id: user.id,
			subject_id: user.sub,
			provider: user.provider,
			username: None,
			display: user.display,
			email: user.email,
		},
	};

	session.set(CURRENT_USER, user);
	Ok((StatusCode::SEE_OTHER, [(LOCATION, "/")]))
}

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
		.route_layer(middleware::from_fn(session::auth))
		.route("/oidc/:provider", get(provider))
		.route("/oidc/:provider/callback", get(provider_callback))
		.route("/logout", post(logout))
}
