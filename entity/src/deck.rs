//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use super::sea_orm_active_enums::{Kind, Share};
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ts_rs :: TS)]
#[sea_orm(table_name = "deck")]
#[ts(export)]
#[ts(rename = "Deck")]
pub struct Model {
	#[sea_orm(
		primary_key,
		auto_increment = false,
		column_type = "Binary(BlobSize::Blob(Some(16)))"
	)]
	#[serde(skip_deserializing)]
	pub uid: uuid::Uuid,
	#[sea_orm(column_type = "Text")]
	pub name: String,
	#[serde(skip_deserializing)]
	pub creator: u32,
	pub kind: Kind,
	pub share: Share,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::deck_cards::Entity")]
	DeckCards,
	#[sea_orm(has_many = "super::followed_decks::Entity")]
	FollowedDecks,
	#[sea_orm(
		belongs_to = "super::user::Entity",
		from = "Column::Creator",
		to = "super::user::Column::Id",
		on_update = "Restrict",
		on_delete = "Restrict"
	)]
	User,
}

impl Related<super::deck_cards::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::DeckCards.def()
	}
}

impl Related<super::followed_decks::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::FollowedDecks.def()
	}
}

impl Related<super::flash_card::Entity> for Entity {
	fn to() -> RelationDef {
		super::deck_cards::Relation::FlashCard.def()
	}
	fn via() -> Option<RelationDef> {
		Some(super::deck_cards::Relation::Deck.def().rev())
	}
}

impl Related<super::user::Entity> for Entity {
	fn to() -> RelationDef {
		super::followed_decks::Relation::User.def()
	}
	fn via() -> Option<RelationDef> {
		Some(super::followed_decks::Relation::Deck.def().rev())
	}
}

impl ActiveModelBehavior for ActiveModel {}
