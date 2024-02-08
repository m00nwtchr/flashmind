use axum::async_trait;
use axum::extract::{FromRef, FromRequestParts, Path};
use axum::http::request::Parts;
use axum::http::StatusCode;

use mongodb::bson::oid::ObjectId;
use mongodb::options::ClientOptions;
use mongodb::{Client, Collection, Database};
use flashmind_schema::FlashCard;

use crate::schema::FlashCardPack;

pub async fn db() -> Database {
	let db_connection_str = std::env::var("DATABASE_URL").unwrap_or_else(|_| {
		"mongodb://root:root@localhost:27017/flashmind?authSource=admin".to_string()
	});

	let options = ClientOptions::parse(&db_connection_str).await.unwrap();
	let client = Client::with_options(options).unwrap();

	client.default_database().unwrap()
}

pub struct FlashCardCollection(pub Collection<FlashCard>);
pub struct PackCollection(pub Collection<FlashCardPack>);

#[async_trait]
impl<S> FromRequestParts<S> for FlashCardCollection
where
	Database: FromRef<S>,
	S: Send + Sync,
{
	type Rejection = (StatusCode, String);

	async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		Ok(Self(Database::from_ref(state).collection("flashcards")))
	}
}

#[async_trait]
impl<S> FromRequestParts<S> for PackCollection
where
	Database: FromRef<S>,
	S: Send + Sync,
{
	type Rejection = (StatusCode, String);

	async fn from_request_parts(_parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
		Ok(Self(Database::from_ref(state).collection("packs")))
	}
}
