use std::collections::HashMap;

use bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

pub mod lang;
use crate::lang::Language;

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
	#[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
	pub id: Option<ObjectId>,

	pub content: Vec<FlashCardSection>,
}

#[derive(Serialize, Deserialize)]
pub enum FlashCardPackKind {
	Language,
	Other,
}

#[derive(Serialize, Deserialize)]
pub struct FlashCardPack {
	pub name: String,
	pub kind: FlashCardPackKind,
	pub flashcards: Vec<FlashCard>,
}

// pub const A: FlashCard = FlashCard {
// 	id: "informacja_information".to_string(),
// 	layout: vec![
// 		FlashCardSection::FrontBack {
// 			front: FlashCardItem::Title("informacja".to_string()),
// 			back: FlashCardItem::Title("information".to_string())
// 		},
// 		FlashCardSection::FrontBack {
// 			front: FlashCardItem::Pronunciation {
// 				ipa: "in.fɔrˈmat͡s.ja".to_string(),
// 				audio_url: Some("https://upload.wikimedia.org/wikipedia/commons/8/89/Pl-informacja.ogg".to_string()),
// 			},
// 			back: FlashCardItem::Pronunciation {
// 				ipa: "ˌɪn.fəˈmeɪ.ʃən".to_string(),
// 				audio_url: Some("https://upload.wikimedia.org/wikipedia/commons/8/89/Pl-informacja.ogg".to_string()),
// 			},
// 		},
// 		FlashCardSection::Item(FlashCardItem::Image("https://www.freeiconspng.com/uploads/-20-mb-format-psd-color-theme-blue-white-keywords-information-icon-2.jpg".to_string())),
// 		FlashCardSection::FrontBack {
// 			front: FlashCardItem::Example("Potrzebuję informacji o tym hotelu.".to_string()),
// 			back: FlashCardItem::Example("I need some information on this hotel.".to_string()),
// 		},
// 		FlashCardSection::Separator,
// 		FlashCardSection::FrontBack {
// 			front: FlashCardItem::Example("Gdzie jest punkt informacji turystycznej?".to_string()),
// 			back: FlashCardItem::Example("Where is the tourist information office?".to_string()),
// 		},
// 		FlashCardSection::FrontBack {
// 			front: FlashCardItem::Example("Gdzie jest punkt informacji turystycznej?".to_string()),
// 			back: FlashCardItem::Example("Where is the tourist information office?".to_string()),
// 		},
// 		FlashCardSection::FrontBack {
// 			front: FlashCardItem::Example("Gdzie jest punkt informacji turystycznej?".to_string()),
// 			back: FlashCardItem::Example("Where is the tourist information office?".to_string()),
// 		},
// 	]
// };
