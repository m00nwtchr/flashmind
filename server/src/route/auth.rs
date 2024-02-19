use axum::routing::post;
use axum::{
	extract::Query,
	http::{header::LOCATION, StatusCode},
	response::IntoResponse,
	routing::get,
	Router,
};
use axum_session::{Session, SessionNullPool};
use openidconnect::{
	core::CoreResponseType, reqwest::async_http_client, AuthenticationFlow, AuthorizationCode,
	CsrfToken, Nonce, RequestTokenError, Scope, TokenResponse,
};
use serde::Deserialize;

use crate::{
	app::AppState,
	oidc::OIDCProvider,
	session::{AUTH_PROVIDER, CSRF_TOKEN, ID_TOKEN},
};

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
	code: AuthorizationCode,
	state: CsrfToken,
}

async fn provider(
	OIDCProvider(_, client): OIDCProvider,
	session: Session<SessionNullPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let (authorize_url, csrf_state, _nonce) = client
		.authorize_url(
			AuthenticationFlow::<CoreResponseType>::AuthorizationCode,
			CsrfToken::new_random,
			Nonce::new_random,
		)
		.add_scope(Scope::new("email".to_string()))
		.add_scope(Scope::new("profile".to_string()))
		.url();
	session.set(CSRF_TOKEN, csrf_state);

	Ok((StatusCode::FOUND, [(LOCATION, authorize_url.to_string())]))
}

async fn provider_callback(
	OIDCProvider(provider, client): OIDCProvider,
	Query(query): Query<AuthRequest>,
	session: Session<SessionNullPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let csrf_token: CsrfToken = session.get(CSRF_TOKEN).ok_or_else(|| {
		(
			StatusCode::BAD_REQUEST,
			"Missing state in session".to_string(),
		)
	})?;

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
			_ => (StatusCode::INTERNAL_SERVER_ERROR, "Other error".to_string()),
		})?;

	let id_token = token_response.id_token().unwrap();
	session.set(ID_TOKEN, id_token);
	session.set(AUTH_PROVIDER, provider);

	Ok((StatusCode::SEE_OTHER, [(LOCATION, "/")]))
}

async fn logout(
	session: Session<SessionNullPool>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	session.destroy();
	Ok((StatusCode::SEE_OTHER, [(LOCATION, "/".to_string())]))
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/oidc/:provider", get(provider))
		.route("/oidc/:provider/callback", get(provider_callback))
		.route("/logout", post(logout))
}
