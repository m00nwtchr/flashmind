use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

use flashmind_schema::FlashCardPackKind;

#[derive(Serialize, Deserialize)]
pub struct FlashCardPack {
	#[serde(rename = "_id")]
	oid: ObjectId,

	pub name: String,
	pub kind: FlashCardPackKind,
	pub flashcards: Vec<ObjectId>,
}
