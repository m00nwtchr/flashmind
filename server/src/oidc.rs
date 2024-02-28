use std::ops::Deref;

use axum::{
	async_trait,
	extract::{FromRef, FromRequestParts, Path},
	http::{request::Parts, StatusCode},
};
use futures::stream::StreamExt;
use openidconnect::{
	core::{CoreClient, CoreProviderMetadata},
	reqwest::async_http_client,
	ClientId, ClientSecret, IssuerUrl, RedirectUrl,
};
use rustc_hash::FxHashMap;
use serde::Serialize;

use crate::app::AppState;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OIDCProvider {
	pub id: String,
	pub name: Option<String>,
	pub client_id: String,
	pub url: IssuerUrl,
	pub icon_url: Option<String>,
	#[serde(skip)]
	pub client: CoreClient,
}

#[derive(Clone)]
pub struct OIDCProviders(FxHashMap<String, OIDCProvider>);

impl Deref for OIDCProviders {
	type Target = FxHashMap<String, OIDCProvider>;

	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

// pub struct OIDCProvider(pub String, pub CoreClient);

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

		Ok(client.clone())
	}
}

pub async fn get_oidc_providers(base_url: String) -> OIDCProviders {
	let mut providers = FxHashMap::default();
	for (key, value) in std::env::vars() {
		if let Some(provider_name) = extract_provider_name(&key) {
			let (client_id, client_secret, issuer_url, name, icon_url) =
				providers.entry(provider_name).or_insert((
					String::new(),
					String::new(),
					String::new(),
					String::new(),
					String::new(),
				));

			if key.ends_with("_OIDC_CLIENT_ID") {
				*client_id = value;
			} else if key.ends_with("_OIDC_CLIENT_SECRET") {
				*client_secret = value;
			} else if key.ends_with("_OIDC_ISSUER_URL") {
				*issuer_url = value;
			} else if key.ends_with("_OIDC_NAME") {
				*name = value;
			} else if key.ends_with("_OIDC_ICON_URL") {
				*icon_url = value;
			}
		}
	}

	let providers = futures::stream::iter(providers)
		.filter_map(
			|(id, (client_id, client_secret, issuer_url, name, icon_url))| {
				let base_url = base_url.clone();
				async move {
					if !client_id.is_empty() && !client_secret.is_empty() && !issuer_url.is_empty()
					{
						let redirect_url = format!("{base_url}/{id}");

						let url = IssuerUrl::new(issuer_url).ok()?;
						let provider_metadata =
							CoreProviderMetadata::discover_async(url.clone(), async_http_client)
								.await
								.ok()?;

						Some((
							id.clone(),
							OIDCProvider {
								id: id.clone(),
								name: if name.is_empty() { None } else { Some(name) },
								client_id: client_id.clone(),
								url,
								icon_url: if icon_url.is_empty() {
									None
								} else {
									Some(icon_url)
								},
								client: CoreClient::from_provider_metadata(
									provider_metadata,
									ClientId::new(client_id),
									Some(ClientSecret::new(client_secret)),
								)
								.set_redirect_uri(RedirectUrl::new(redirect_url).ok()?),
							},
						))
					} else {
						None
					}
				}
			},
		)
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
	} else if key.ends_with("_OIDC_NAME") {
		Some(key.trim_end_matches("_OIDC_NAME").to_lowercase())
	} else if key.ends_with("_OIDC_ICON_URL") {
		Some(key.trim_end_matches("_OIDC_ICON_URL").to_lowercase())
	} else {
		None
	}
}
