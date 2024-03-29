//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize, ts_rs::TS)]
#[sea_orm(table_name = "followed_decks")]
pub struct Model {
	#[sea_orm(primary_key, auto_increment = false)]
	#[serde(skip_deserializing)]
	pub user: u32,
	#[sea_orm(
		primary_key,
		auto_increment = false,
		column_type = "Binary(BlobSize::Blob(Some(16)))"
	)]
	#[serde(skip_deserializing)]
	pub deck: uuid::Uuid,
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
		belongs_to = "super::user::Entity",
		from = "Column::User",
		to = "super::user::Column::Id",
		on_update = "Restrict",
		on_delete = "Restrict"
	)]
	User,
}

impl Related<super::deck::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::Deck.def()
	}
}

impl Related<super::user::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::User.def()
	}
}

impl ActiveModelBehavior for ActiveModel {}
