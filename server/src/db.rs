use sea_orm::{Database, DatabaseConnection};

use crate::config::AppConfig;

pub async fn db(config: &AppConfig) -> DatabaseConnection {
	Database::connect(&config.db_url).await.expect("")
}
