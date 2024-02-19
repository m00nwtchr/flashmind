use crate::dto::flashcard::FlashCard;
use crate::entities::sea_orm_active_enums::Share;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum DeckKind {
	Language,
	Other,
}

#[derive(Serialize, Deserialize)]
pub struct Deck {
	pub uid: String,
	pub name: String,
	pub kind: DeckKind,
	pub share: Share,
	pub flashcards: Vec<FlashCard>,
}
