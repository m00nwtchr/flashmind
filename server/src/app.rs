use std::{ops::Deref, sync::Arc};

use axum::{
	extract::{FromRef, State},
	http::StatusCode,
	routing::get,
	Json, Router,
};
use axum_session::{Key, SessionConfig, SessionLayer, SessionNullPool, SessionStore};
use migration::MigratorTrait;
use sea_orm::DatabaseConnection;
use serde_json::json;

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
	let providers = oidc::get_oidc_providers(format!("{}/login", config.public_url)).await;
	let db = db(&config).await;

	migration::Migrator::up(&db, None)
		.await
		.expect("Migration failed");

	let state = AppState(Arc::new(AppStateInner { providers, db }));

	let session_config = SessionConfig::default()
		.with_key(Key::generate())
		.with_secure(true)
		.with_ip_and_user_agent(false);

	let session_store = SessionStore::<SessionNullPool>::new(None, session_config)
		.await
		.unwrap();

	Router::new()
		.nest("/api", route::api())
		.route("/", get(|| async { "Hello, World!".to_string() }))
		.with_state(state)
		.route("/.well-known/assetlinks.json", get(asset_links))
		.with_state(config)
		.layer(SessionLayer::new(session_store))
}

async fn asset_links(
	State(config): State<AppConfig>,
) -> Result<Json<serde_json::Value>, StatusCode> {
	if config.app_fingerprints.is_empty() {
		return Err(StatusCode::NOT_FOUND);
	}

	Ok(Json(json!([{
		"relation": ["delegate_permission/common.handle_all_urls"],
		"target": {
			"namespace": "android_app",
			"package_name": config.app_id,
			"sha256_cert_fingerprints": config.app_fingerprints
		}
	}])))
}
