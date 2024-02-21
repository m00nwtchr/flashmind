use crate::sea_orm::{DbBackend, EnumIter, Iterable};
use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
	async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		let sqlite = matches!(manager.get_database_backend(), DbBackend::Sqlite);

		manager
			.create_table(
				Table::create()
					.table(User::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(User::Id)
							.integer()
							.unsigned()
							.not_null()
							.auto_increment()
							.primary_key(),
					)
					.col(ColumnDef::new(User::Sub).string_len(255).not_null())
					.col(ColumnDef::new(User::Provider).string().null())
					.col(ColumnDef::new(User::Display).string().null())
					.col(ColumnDef::new(User::Email).string().null())
					.to_owned(),
			)
			.await?;
		manager
			.create_table(
				Table::create()
					.table(FlashCard::Table)
					.if_not_exists()
					.col(
						ColumnDef::new(FlashCard::Uid)
							.uuid()
							.not_null()
							.primary_key(),
					)
					.col(ColumnDef::new(FlashCard::Creator).unsigned().not_null())
					.col(
						ColumnDef::new(FlashCard::Share)
							.enumeration(Alias::new("share"), Share::iter())
							.not_null(),
					)
					.col(ColumnDef::new(FlashCard::Content).json().not_null())
					.index(Index::create().col(FlashCard::Creator))
					.to_owned(),
			)
			.await?;

		manager
			.create_table(
				Table::create()
					.table(Deck::Table)
					.if_not_exists()
					.col(ColumnDef::new(Deck::Uid).uuid().not_null().primary_key())
					.col(ColumnDef::new(Deck::Name).text().not_null())
					.col(ColumnDef::new(Deck::Creator).unsigned().not_null())
					.col(
						ColumnDef::new(Deck::Kind)
							.enumeration(Alias::new("kind"), Kind::iter())
							.not_null(),
					)
					.col(
						ColumnDef::new(Deck::Share)
							.enumeration(Alias::new("share"), Share::iter())
							.not_null(),
					)
					.index(Index::create().col(Deck::Creator))
					.to_owned(),
			)
			.await?;

		let mut table = Table::create();
		table
			.table(DeckCards::Table)
			.if_not_exists()
			.col(ColumnDef::new(DeckCards::Deck).uuid().not_null())
			.col(ColumnDef::new(DeckCards::Card).uuid().not_null())
			.primary_key(Index::create().col(DeckCards::Deck).col(DeckCards::Card))
			.foreign_key(
				ForeignKey::create()
					.from(DeckCards::Table, DeckCards::Deck)
					.to(Deck::Table, Deck::Uid),
			)
			.foreign_key(
				ForeignKey::create()
					.from(DeckCards::Table, DeckCards::Card)
					.to(Deck::Table, FlashCard::Uid),
			);
		if !sqlite {
			table.index(
				Index::create()
					.name("card-deck")
					.col(DeckCards::Card)
					.col(DeckCards::Deck),
			);
		}

		manager.create_table(table.to_owned()).await?;

		let mut table = Table::create();
		table
			.table(FollowedDecks::Table)
			.if_not_exists()
			.col(ColumnDef::new(FollowedDecks::User).unsigned().not_null())
			.col(ColumnDef::new(FollowedDecks::Deck).uuid().not_null())
			.primary_key(
				Index::create()
					.col(FollowedDecks::User)
					.col(FollowedDecks::Deck),
			)
			.foreign_key(
				ForeignKey::create()
					.from(FollowedDecks::Table, FollowedDecks::User)
					.to(User::Table, User::Id),
			)
			.foreign_key(
				ForeignKey::create()
					.from(FollowedDecks::Table, FollowedDecks::Deck)
					.to(Deck::Table, Deck::Uid),
			);
		if !sqlite {
			table.index(
				Index::create()
					.name("deck-user")
					.col(FollowedDecks::Deck)
					.col(FollowedDecks::User),
			);
		}

		manager.create_table(table.to_owned()).await?;

		Ok(())
	}

	async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
		manager
			.drop_table(Table::drop().table(User::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(FlashCard::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(Deck::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(DeckCards::Table).to_owned())
			.await?;

		manager
			.drop_table(Table::drop().table(FollowedDecks::Table).to_owned())
			.await?;

		Ok(())
	}
}

#[derive(DeriveIden)]
enum User {
	Table,
	Id,
	Sub,
	Provider,
	Display,
	Email,
}

#[derive(DeriveIden)]
enum FlashCard {
	Table,
	Uid,
	Creator,
	Share,
	Content,
}

#[derive(DeriveIden)]
enum Deck {
	Table,
	Uid,
	Name,
	Creator,
	Kind,
	Share,
}

#[derive(Iden, EnumIter)]
pub enum Kind {
	#[iden = "Language"]
	Language,
	#[iden = "Other"]
	Other,
}

#[derive(Iden)]
enum DeckCards {
	Table,
	Deck,
	Card,
}

#[derive(Iden)]
enum FollowedDecks {
	Table,
	User,
	Deck,
}

#[derive(Iden, EnumIter)]
pub enum Share {
	#[iden = "Public"]
	Public,
	#[iden = "Private"]
	Private,
}
