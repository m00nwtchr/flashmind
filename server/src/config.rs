#[derive(Clone)]
pub struct AppConfig {
	pub public_url: String,
	pub listen_addr: String,
	pub db_url: String,
}

impl AppConfig {
	pub fn from_env() -> Self {
		AppConfig {
			public_url: std::env::var("FLASHMIND_PUBLIC_URL")
				.unwrap_or(String::from("http://localhost:3000")),
			listen_addr: std::env::var("FLASHMIND_LISTEN_ADDR")
				.unwrap_or(String::from("[::]:3000")),
			db_url: std::env::var("FLASHMIND_DB_URL").expect("You must provide a database url."),
		}
	}
}
