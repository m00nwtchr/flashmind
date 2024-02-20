//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.14

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "user")]
pub struct Model {
	#[sea_orm(primary_key)]
	pub id: u32,
	pub sub: String,
	#[sea_orm(column_type = "Text")]
	pub provider: String,
	#[sea_orm(column_type = "Text", nullable)]
	pub display: Option<String>,
	#[sea_orm(column_type = "Text", nullable)]
	pub email: Option<String>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
	#[sea_orm(has_many = "super::deck::Entity")]
	Deck,
	#[sea_orm(has_many = "super::flash_card::Entity")]
	FlashCard,
	#[sea_orm(has_many = "super::followed_decks::Entity")]
	FollowedDecks,
}

impl Related<super::flash_card::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::FlashCard.def()
	}
}

impl Related<super::followed_decks::Entity> for Entity {
	fn to() -> RelationDef {
		Relation::FollowedDecks.def()
	}
}

impl Related<super::deck::Entity> for Entity {
	fn to() -> RelationDef {
		super::followed_decks::Relation::Deck.def()
	}
	fn via() -> Option<RelationDef> {
		Some(super::followed_decks::Relation::User.def().rev())
	}
}

impl ActiveModelBehavior for ActiveModel {}
