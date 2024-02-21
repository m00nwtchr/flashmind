use rustc_hash::FxHashMap;

use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};

use crate::data::lang::Language;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum FlashCardItem {
	Title(String),
	Pronunciation {
		ipa: String,
		audio_url: Option<String>,
	},
	Image(String),
	Example(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
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
	Lang(FxHashMap<Language, FlashCardItem>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult)]
#[serde(transparent)]
pub struct FlashCardContent(pub Vec<FlashCardSection>);
