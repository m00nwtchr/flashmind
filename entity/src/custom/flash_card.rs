use std::collections::HashMap;

use sea_orm::FromJsonQueryResult;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::custom::lang::Language;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
// #[ts(export)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase")]
pub enum FlashCardItem {
	Title(String),
	Pronunciation {
		ipa: String,
		#[serde(skip_serializing_if = "Option::is_none")]
		audio_url: Option<String>,
	},
	Image(String),
	Example(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(tag = "type", content = "content")]
pub enum FlashCardSection {
	Separator,
	// #[serde(untagged)]
	Item(FlashCardItem),
	// #[serde(untagged)]
	FrontBack {
		front: FlashCardItem,
		back: FlashCardItem,
	},
	// #[serde(untagged)]
	Lang(HashMap<Language, FlashCardItem>),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, FromJsonQueryResult, TS)]
#[ts(export)]
#[serde(transparent)]
pub struct FlashCardContent(pub Vec<FlashCardSection>);
