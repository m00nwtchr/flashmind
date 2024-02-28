use axum::{
	extract::{Query, State},
	http::{header::LOCATION, StatusCode},
	response::IntoResponse,
	routing::{get, post},
	Json, Router,
};
use openidconnect::{
	reqwest::async_http_client, AuthorizationCode, Nonce, PkceCodeVerifier, RequestTokenError,
	TokenResponse,
};
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::Deserialize;

use crate::{
	app::AppState,
	internal_error,
	oidc::OIDCProvider,
	session::{CurrentUser, Session, CURRENT_USER},
};
use entity::{prelude::*, user};

#[derive(Debug, Deserialize)]
pub struct AuthRequest {
	code: AuthorizationCode,
	// state: CsrfToken,
	code_verifier: PkceCodeVerifier,
}

async fn exchange_code(
	provider: OIDCProvider,
	Query(query): Query<AuthRequest>,
	State(db): State<DatabaseConnection>,
	session: Session,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let token_response = provider
		.client
		.exchange_code(query.code)
		.set_pkce_verifier(query.code_verifier)
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
		.claims(
			&provider.client.id_token_verifier(),
			|nonce: Option<&Nonce>| Ok(()),
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

async fn providers(State(state): State<AppState>) -> Json<Vec<OIDCProvider>> {
	Json(state.providers.values().cloned().collect())
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", get(providers))
		// .route("/:provider", get(provider))
		.route("/:provider", post(exchange_code))
}
