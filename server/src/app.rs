use std::{ops::Deref, sync::Arc};

use axum::{extract::FromRef, routing::get, Router};
use axum_session::{Key, SessionConfig, SessionLayer, SessionNullPool, SessionStore};
use sea_orm::DatabaseConnection;

use crate::{config::AppConfig, db::db, oidc, oidc::OIDCProviders, route};

pub struct AppStateInner {
	pub providers: OIDCProviders,
	pub db: DatabaseConnection,
}

#[derive(Clone)]
pub struct AppState(Arc<AppStateInner>);

impl Deref for AppState {
	type Target = AppStateInner;

	#[allow(clippy::explicit_deref_methods)]
	fn deref(&self) -> &Self::Target {
		self.0.deref()
	}
}

impl FromRef<AppState> for DatabaseConnection {
	fn from_ref(input: &AppState) -> Self {
		input.db.clone()
	}
}

pub async fn app(config: AppConfig) -> Router {
	let providers = oidc::get_oidc_providers(format!("{}/auth/oidc", config.public_url)).await;
	let db = db(&config).await;

	let state = AppState(Arc::new(AppStateInner { providers, db }));

	let session_config = SessionConfig::default()
		.with_key(Key::generate())
		.with_secure(true);

	let session_store = SessionStore::<SessionNullPool>::new(None, session_config)
		.await
		.unwrap();

	Router::new()
		.nest("/api/flashcard", route::api::flashcard())
		.nest("/auth", route::auth())
		.route("/", get(|| async { "Hello World!".to_string() }))
		.with_state(state)
		.layer(SessionLayer::new(session_store))
}
