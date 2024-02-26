use axum::{
	extract::{Query, State},
	http::{header::LOCATION, StatusCode},
	middleware,
	response::IntoResponse,
	routing::{get, post},
	Extension, Json, Router,
};
use openidconnect::{
	core::CoreResponseType, reqwest::async_http_client, AuthenticationFlow, AuthorizationCode,
	CsrfToken, Nonce, RequestTokenError, Scope, TokenResponse,
};
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::{
	app::AppState,
	internal_error,
	oidc::OIDCProvider,
	session,
	session::{CurrentUser, Session, CURRENT_USER, OIDC_CSRF_TOKEN, OIDC_NONCE},
};
use entity::{prelude::*, user};

async fn provider(
	provider: OIDCProvider,
	session: Session,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let (authorize_url, csrf_state, nonce) = provider
		.client
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

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
	code: AuthorizationCode,
	state: CsrfToken,
}

async fn provider_callback(
	provider: OIDCProvider,
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
	let token_response = provider
		.client
		.exchange_code(query.code)
		.request_async(async_http_client)
		.await
		.map_err(|err| match err {
			RequestTokenError::ServerResponse(err) => {
				(StatusCode::NOT_FOUND, format!("{}", err.error()))
			}
			_ => internal_error(err),
		})?;
	let id_token = token_response.id_token().unwrap();
	let claims = id_token
		.claims(&provider.client.id_token_verifier(), &nonce)
		.unwrap();
	let sub = claims.subject();

	let res = User::find()
		.filter(user::Column::Sub.eq(sub.as_str()))
		.filter(user::Column::Provider.eq(&provider.id))
		.one(&db)
		.await
		.map_err(internal_error)?;

	let user = match res {
		None => {
			let mut u = CurrentUser {
				user_id: 0,
				subject_id: sub.to_string(),
				provider: provider.id,
				username: claims.preferred_username().map(|c| c.to_string()),
				display: claims
					.name()
					.and_then(|c| c.get(None))
					.map(|c| c.to_string()),
				email: claims.email().map(|c| c.to_string()),
			};

			// Create new user
			u.user_id = User::insert(user::ActiveModel {
				sub: Set(sub.to_string()),
				provider: Set(u.provider.clone()),
				display: Set(u.display.clone()),
				email: Set(u.display.clone()),
				..Default::default()
			})
				.exec(&db)
				.await
				.map_err(internal_error)?
				.last_insert_id;

			u
		}
		Some(model) => {
			let user = CurrentUser {
				user_id: model.id,
				subject_id: model.sub,
				provider: provider.id,
				username: claims.preferred_username().map(|c| c.to_string()),
				display: claims
					.name()
					.and_then(|c| c.get(None))
					.map(|c| c.to_string()),
				email: claims.email().map(|c| c.to_string()),
			};

			// TODO: Consider updating user data on subsequent login
			// entity::User::update(user.update_model(model.into_active_model()))
			// 	.exec(&db)
			// 	.await
			// 	.map_err(internal_error)?;

			user
		}
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

async fn providers(State(state): State<AppState>) -> Json<Vec<OIDCProvider>> {
	Json(state.providers.values().cloned().collect())
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/user", get(user))
		.route("/logout", post(logout))
		.route_layer(middleware::from_fn(session::auth))
		.route("/oidc", get(providers))
		.route("/oidc/:provider", get(provider))
		.route("/oidc/:provider/callback", get(provider_callback))
}
