use axum::{
	extract::{Path, State},
	http::{header::LOCATION, StatusCode},
	middleware,
	response::IntoResponse,
	routing::{delete, get, patch, post, put},
	Extension, Json, Router,
};
use sea_orm::{
	ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait, QueryFilter,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{app::AppState, internal_error, session};
use entity::{deck, deck_cards, flash_card, prelude::*, sea_orm_active_enums::Share, user};

async fn create(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<user::Model>,
	Json(mut deck): Json<deck::Model>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	deck.uid = Uuid::new_v4();
	deck.creator = user.id;

	Deck::insert(deck::ActiveModel {
		uid: Set(deck.uid),
		name: Set(deck.name.clone()),
		creator: Set(deck.creator),
		kind: Set(deck.kind),
		share: Set(deck.share),
	})
	.exec(&conn)
	.await
	.map_err(internal_error)?;

	Ok((
		StatusCode::CREATED,
		[(LOCATION, deck.uid.to_string())],
		Json(deck),
	))
}

async fn all(
	State(db): State<DatabaseConnection>,
	Extension(user): Extension<user::Model>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let decks = Deck::find()
		.filter(deck::Column::Creator.eq(user.id))
		.all(&db)
		.await
		.map_err(internal_error)?;

	Ok(Json(decks))
}

async fn get_one(
	State(db): State<DatabaseConnection>,
	Extension(user): Extension<user::Model>,
	Path(id): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let deck = Deck::find_by_id(id)
		.filter(
			deck::Column::Share.eq(Share::Public).or(deck::Column::Share
				.eq(Share::Private)
				.and(deck::Column::Creator.eq(user.id))),
		)
		.one(&db)
		.await
		.map_err(internal_error)?
		.ok_or_else(|| (StatusCode::NOT_FOUND, "Not found".to_string()))?;

	match deck.share {
		Share::Public => Ok(Json(deck)),
		Share::Private => {
			if user.id == deck.creator {
				Ok(Json(deck))
			} else {
				Err((StatusCode::NOT_FOUND, "Not found".to_string()))
			}
		}
	}
}

async fn update(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<user::Model>,
	Path(uid): Path<Uuid>,
	Json(body): Json<deck::Model>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let deck: deck::ActiveModel = Deck::find_by_id(uid)
		.filter(deck::Column::Creator.eq(user.id))
		.one(&conn)
		.await
		.map_err(internal_error)?
		.ok_or((StatusCode::NOT_FOUND, "Not found.".to_string()))?
		.into();

	Deck::update(deck::ActiveModel {
		uid: deck.uid,
		name: Set(body.name),
		creator: deck.creator,
		kind: Set(body.kind),
		share: Set(body.share),
	})
	.exec(&conn)
	.await
	.map_err(internal_error)?;
	Ok(StatusCode::NO_CONTENT)
}

async fn delete_deck(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<user::Model>,
	Path(uid): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
	Deck::delete(deck::ActiveModel {
		uid: Set(uid),
		..Default::default()
	})
	.filter(flash_card::Column::Creator.eq(user.id))
	.exec(&conn)
	.await
	.map_err(internal_error)?;
	Ok(StatusCode::NO_CONTENT)
}

async fn get_cards(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<user::Model>,
	Path(uid): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let Some(deck) = Deck::find_by_id(uid)
		.filter(
			deck::Column::Share.eq(Share::Public).or(deck::Column::Share
				.eq(Share::Private)
				.and(deck::Column::Creator.eq(user.id))),
		)
		.one(&conn)
		.await
		.map_err(internal_error)?
	else {
		return Err((StatusCode::NOT_FOUND, "Not found".to_string()));
	};

	let cards = deck
		.find_related(FlashCard)
		.filter(
			flash_card::Column::Share
				.eq(Share::Public)
				.or(flash_card::Column::Share
					.eq(Share::Private)
					.and(flash_card::Column::Creator.eq(user.id))),
		)
		.all(&conn)
		.await
		.map_err(internal_error)?;
	Ok((StatusCode::OK, Json(cards)))
}

async fn add_cards(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<user::Model>,
	Path(uid): Path<Uuid>,
	Json(ids): Json<Vec<Uuid>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let Some(deck) = Deck::find_by_id(uid)
		.filter(deck::Column::Creator.eq(user.id))
		.one(&conn)
		.await
		.map_err(internal_error)?
	else {
		return Err((StatusCode::NOT_FOUND, "Not found".to_string()));
	};

	DeckCards::delete_many()
		.filter(deck_cards::Column::Deck.eq(deck.uid))
		.exec(&conn)
		.await
		.map_err(internal_error)?;

	DeckCards::insert_many(ids.into_iter().map(|id| deck_cards::ActiveModel {
		card: Set(id),
		deck: Set(deck.uid),
	}))
	.exec(&conn)
	.await
	.map_err(internal_error)?;
	Ok(StatusCode::NO_CONTENT)
}

#[derive(Deserialize)]
pub struct UpdatePatch<T> {
	#[serde(default)]
	add: Vec<T>,
	#[serde(default)]
	remove: Vec<T>,
}

async fn update_cards(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<user::Model>,
	Path(uid): Path<Uuid>,
	Json(ids): Json<UpdatePatch<Uuid>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let Some(deck) = Deck::find_by_id(uid)
		.filter(deck::Column::Creator.eq(user.id))
		.one(&conn)
		.await
		.map_err(internal_error)?
	else {
		return Err((StatusCode::NOT_FOUND, "Not found".to_string()));
	};

	if !ids.remove.is_empty() {
		DeckCards::delete_many()
			.filter(deck_cards::Column::Card.is_in(ids.remove.clone()))
			.exec(&conn)
			.await
			.map_err(internal_error)?;
	}

	if !ids.add.is_empty() {
		let models = ids.add.into_iter().map(|id| deck_cards::ActiveModel {
			card: Set(id),
			deck: Set(deck.uid),
		});

		DeckCards::insert_many(models)
			.exec(&conn)
			.await
			.map_err(internal_error)?;
	}

	let cards = deck
		.find_related(FlashCard)
		.filter(
			flash_card::Column::Share
				.eq(Share::Public)
				.or(flash_card::Column::Share
					.eq(Share::Private)
					.and(flash_card::Column::Creator.eq(user.id))),
		)
		.all(&conn)
		.await
		.map_err(internal_error)?;
	Ok((StatusCode::OK, Json(cards)))
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", post(create))
		.route("/", get(all))
		.route("/:id", get(get_one))
		.route("/:id", put(update))
		.route("/:id", delete(delete_deck))
		.route("/:id/cards", get(get_cards))
		.route("/:id/cards", put(add_cards))
		.route("/:id/cards", patch(update_cards))
		.route_layer(middleware::from_fn(session::auth))
}
