use axum::{
	extract::Path,
	http::{header::LOCATION, HeaderMap, StatusCode},
	routing::{delete, get, post, put},
	Json, Router,
};
use futures::stream::TryStreamExt;
use mongodb::{
	bson::{doc, oid::ObjectId},
	Database,
};

use crate::{db::FlashCardCollection, internal_error};
use flashmind_schema::FlashCard;

async fn create(
	FlashCardCollection(collection): FlashCardCollection,
	Json(mut flashcard): Json<FlashCard>,
) -> Result<(StatusCode, HeaderMap, Json<FlashCard>), (StatusCode, String)> {
	flashcard.id = None;
	let object_id: ObjectId = collection
		.insert_one(&flashcard, None)
		.await
		.map_err(internal_error)?
		.inserted_id
		.as_object_id()
		.expect("Object Id");

	flashcard.id = Some(object_id);
	let hex = object_id.to_hex();

	let mut headers = HeaderMap::new();
	headers.insert(LOCATION, hex.as_str().parse().unwrap());

	Ok((StatusCode::CREATED, headers, flashcard.into()))
}

async fn all(
	FlashCardCollection(collection): FlashCardCollection,
) -> Result<Json<Vec<FlashCard>>, (StatusCode, String)> {
	let vec: Vec<FlashCard> = collection
		.find(None, None)
		.await
		.map_err(internal_error)?
		.try_collect()
		.await
		.map_err(internal_error)?;

	Ok(vec.into())
}

async fn get_one(
	FlashCardCollection(collection): FlashCardCollection,
	Path(id): Path<ObjectId>,
) -> Result<Json<FlashCard>, (StatusCode, String)> {
	collection
		.find_one(doc! { "_id": id }, None)
		.await
		.map_err(internal_error)?
		.ok_or((StatusCode::NOT_FOUND, "Not found".to_string()))
		.map(Into::into)
}

async fn update(
	FlashCardCollection(collection): FlashCardCollection,
	Path(id): Path<ObjectId>,
	Json(mut flashcard): Json<FlashCard>,
) -> Result<StatusCode, (StatusCode, String)> {
	flashcard.id = None;
	collection
		.replace_one(doc! { "_id": id }, flashcard, None)
		.await
		.map_err(internal_error)?;

	Ok(StatusCode::NO_CONTENT)
}

async fn del(
	FlashCardCollection(collection): FlashCardCollection,
	Path(id): Path<ObjectId>,
) -> Result<StatusCode, (StatusCode, String)> {
	collection
		.delete_one(doc! { "_id": id }, None)
		.await
		.map_err(internal_error)?;

	Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<Database> {
	Router::new()
		.route("/", post(create))
		.route("/", get(all))
		.route("/:id", get(get_one))
		.route("/:id", put(update))
		.route("/:id", delete(del))
}
