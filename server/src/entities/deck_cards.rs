//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "deck_cards")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false, column_type = "custom(\"uuid\")")]
	#[serde(skip_deserializing)]
	pub deck: String,
	#[sea_orm(primary_key, auto_increment = false, column_type = "custom(\"uuid\")")]
	#[serde(skip_deserializing)]
	pub card: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(
		belongs_to = "super::deck::Entity",
		from = "Column::Deck",
		to = "super::deck::Column::Uid",
		on_update = "Restrict",
		on_delete = "Restrict"
	)]
	Deck,
	#[sea_orm(
		belongs_to = "super::flash_card::Entity",
		from = "Column::Card",
		to = "super::flash_card::Column::Uid",
		on_update = "Restrict",
		on_delete = "Restrict"
	)]
	FlashCard,
}

impl Related<super::deck::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Deck.def()
	}
}

impl Related<super::flash_card::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::FlashCard.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
