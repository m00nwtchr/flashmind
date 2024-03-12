use axum::{
	extract::State,
	http::StatusCode,
	response::IntoResponse,
	routing::{get, post},
	Json, Router,
};
use openidconnect::{
	reqwest::async_http_client, AuthorizationCode, Nonce, PkceCodeVerifier, RequestTokenError,
	TokenResponse,
};
use sea_orm::{ColumnTrait, DatabaseConnection, EntityTrait, IntoActiveModel, QueryFilter};
use serde::{Deserialize, Serialize};

use crate::{
	app::AppState,
	internal_error,
	oidc::OIDCProvider,
	session::{Session, CURRENT_USER},
};
use entity::{prelude::*, user};

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthRequest {
	code: AuthorizationCode,
	// state: CsrfToken,
	code_verifier: PkceCodeVerifier,
}

async fn exchange_code(
	provider: OIDCProvider,
	State(db): State<DatabaseConnection>,
	session: Session,
	Json(req): Json<AuthRequest>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let token_response = provider
		.client
		.exchange_code(req.code)
		.set_pkce_verifier(req.code_verifier)
		.request_async(async_http_client)
		.await
		.map_err(|err| match err {
			RequestTokenError::ServerResponse(err) => {
				(StatusCode::BAD_REQUEST, format!("{}", err.error()))
			}
			_ => internal_error(err),
		})?;

	let id_token = token_response.id_token().unwrap();
	let claims = id_token
		.claims(
			&provider.client.id_token_verifier(),
			|_nonce: Option<&Nonce>| Ok(()),
		)
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
			let mut user = user::Model {
				id: 0,
				sub: sub.to_string(),
				provider: provider.id,
				// 	username: claims.preferred_username().map(|c| c.to_string()),
				display: claims
					.name()
					.and_then(|c| c.get(None))
					.map(|c| c.to_string()),
				email: claims.email().map(|c| c.to_string()),
			};

			// Create new user
			user.id = User::insert(user.clone().into_active_model())
				.exec(&db)
				.await
				.map_err(internal_error)?
				.last_insert_id;

			user
		}
		Some(mut model) => {
			// model.username = claims.preferred_username().map(|c| c.to_string());
			model.display = claims
				.name()
				.and_then(|c| c.get(None))
				.map(|c| c.to_string());
			model.email = claims.email().map(|c| c.to_string());

			// TODO: Consider updating user data on subsequent login
			// User::update(model.clone().into_active_model())
			// 	.exec(&db)
			// 	.await
			// 	.map_err(internal_error)?;

			model
		}
	};

	session.set(CURRENT_USER, user.clone());
	Ok((StatusCode::OK, Json(user)))
}

async fn providers(State(state): State<AppState>) -> Json<Vec<OIDCProvider>> {
	Json(state.providers.values().cloned().collect())
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(providers))
		// .route("/:provider", get(provider))
		.route("/:provider", post(exchange_code))
}
