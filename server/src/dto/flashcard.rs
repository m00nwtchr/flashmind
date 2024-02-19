use crate::dto::lang::Language;
use crate::entities::sea_orm_active_enums::Share;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FlashCardItem {
	Title(String),
	#[serde(rename_all = "camelCase")]
	Pronunciation {
		ipa: String,
		audio_url: Option<String>,
	},
	Image(String),
	Example(String),
}

#[derive(Serialize, Deserialize)]
pub enum FlashCardSection {
	Separator,
	#[serde(untagged)]
	Item(FlashCardItem),
	#[serde(untagged)]
	FrontBack {
		front: FlashCardItem,
		back: FlashCardItem,
	},
	#[serde(untagged)]
	Lang(HashMap<Language, FlashCardItem>),
}

#[derive(Serialize, Deserialize)]
pub struct FlashCard {
	#[serde(default, skip_serializing_if = "Option::is_none")]
	pub id: Option<u32>,
	// #[serde(default, skip_serializing_if = "Option::is_none")]
	// pub creator: Option<u32>,
	pub share: Share,
	pub content: Vec<FlashCardSection>,
}
