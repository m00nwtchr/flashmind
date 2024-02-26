use std::env::{var, vars};

#[derive(Clone)]
pub struct AppConfig {
	pub public_url: String,
	pub listen_addr: String,
	pub db_url: String,
	pub app_id: String,
	pub app_fingerprints: Vec<String>,
}

impl AppConfig {
	pub fn from_env() -> Self {
		Self {
			public_url: var("FLASHMIND_PUBLIC_URL")
				.unwrap_or(String::from("http://localhost:3000")),
			listen_addr: var("FLASHMIND_LISTEN_ADDR").unwrap_or(String::from("[::]:3000")),
			db_url: var("FLASHMIND_DB_URL").expect("You must provide a database url."),
			app_id: var("FLASHMIND_APP_ID").unwrap_or("io.github.m00nwtchr.flashmind".to_string()),
			app_fingerprints: vars()
				.filter(|(k, _)| k.starts_with("FLASHMIND_APP_FINGERPRINT"))
				.map(|(_, v)| v)
				.collect(),
		}
	}
}

impl Default for AppConfig {
	fn default() -> Self {
		Self {
			public_url: String::from("http://localhost:3000"),
			listen_addr: String::from("[::]:3000"),
			db_url: String::from("sqlite::memory:"),
			app_id: "io.github.m00nwtchr.flashmind".to_string(),
			app_fingerprints: Vec::new(),
		}
	}
}
