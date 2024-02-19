use crate::app::AppState;
use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts, Path};
use axum::http::request::Parts;
use axum::http::StatusCode;
use futures::stream::StreamExt;
use openidconnect::core::{CoreClient, CoreProviderMetadata};
use openidconnect::reqwest::async_http_client;
use openidconnect::{ClientId, ClientSecret, IssuerUrl, RedirectUrl};
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Clone)]
pub struct OIDCProviders(HashMap<String, CoreClient>);

impl Deref for OIDCProviders {
	type Target = HashMap<String, CoreClient>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

pub struct OIDCProvider(pub String, pub CoreClient);

#[async_trait]
impl<S> FromRequestParts<S> for OIDCProvider
where
	AppState: FromRef<S>,
	S: Send + Sync,
{
	type Rejection = (StatusCode, String);

	async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		let Path(provider): Path<String> = Path::from_request_parts(parts, state)
			.await
			.map_err(|_| (StatusCode::BAD_REQUEST, String::new()))?;

		let state = AppState::from_ref(state);
		let client = state
			.providers
			.get(&provider)
			.ok_or_else(|| (StatusCode::NOT_FOUND, "No such OpenID Provider".to_string()))?;

		Ok(OIDCProvider(provider.clone(), client.clone()))
	}
}

pub async fn get_oidc_providers(base_url: String) -> OIDCProviders {
	let mut providers = HashMap::new();
	for (key, value) in std::env::vars() {
		if let Some(provider_name) = extract_provider_name(&key) {
			let (client_id, client_secret, issuer_url) = providers
				.entry(provider_name)
				.or_insert((String::new(), String::new(), String::new()));

			if key.ends_with("_OIDC_CLIENT_ID") {
				*client_id = value;
			} else if key.ends_with("_OIDC_CLIENT_SECRET") {
				*client_secret = value;
			} else if key.ends_with("_OIDC_ISSUER_URL") {
				*issuer_url = value;
			}
		}
	}

	let providers = futures::stream::iter(providers)
		.filter_map(|(provider_name, (client_id, client_secret, issuer_url))| {
			let base_url = base_url.clone();
			async move {
				if !client_id.is_empty() && !client_secret.is_empty() && !issuer_url.is_empty() {
					let redirect_url = format!("{base_url}/{provider_name}/callback");

					let provider_metadata = CoreProviderMetadata::discover_async(
						IssuerUrl::new(issuer_url).ok()?,
						async_http_client,
					)
					.await
					.ok()?;

					Some((
						provider_name,
						CoreClient::from_provider_metadata(
							provider_metadata,
							ClientId::new(client_id),
							Some(ClientSecret::new(client_secret)),
						)
						.set_redirect_uri(RedirectUrl::new(redirect_url).ok()?),
					))
				} else {
					None
				}
			}
		})
		.collect()
		.await;

	OIDCProviders(providers)
}

fn extract_provider_name(key: &str) -> Option<String> {
	if key.ends_with("_OIDC_CLIENT_ID") {
		Some(key.trim_end_matches("_OIDC_CLIENT_ID").to_lowercase())
	} else if key.ends_with("_OIDC_CLIENT_SECRET") {
		Some(key.trim_end_matches("_OIDC_CLIENT_SECRET").to_lowercase())
	} else if key.ends_with("_OIDC_ISSUER_URL") {
		Some(key.trim_end_matches("_OIDC_ISSUER_URL").to_lowercase())
	} else {
		None
	}
}
