use crate::config::AppConfig;
use sea_orm::{Database, DatabaseConnection};

pub async fn db(config: &AppConfig) -> DatabaseConnection {
	Database::connect(&config.db_url).await.expect("")
}
