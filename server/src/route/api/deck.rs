use axum::{
	extract::{Path, State},
	http::{header::LOCATION, StatusCode},
	middleware,
	response::IntoResponse,
	routing::{delete, get, post, put},
	Extension, Json, Router,
};
use sea_orm::{
	ActiveValue, ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, ModelTrait,
	QueryFilter,
};

use crate::{
	app::AppState,
	entities::{deck, deck_cards, flash_card, prelude as entity, sea_orm_active_enums::Share},
	internal_error, session,
	session::CurrentUser,
};

async fn create(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<CurrentUser>,
	Json(mut deck): Json<deck::Model>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let res = entity::Deck::insert(deck::ActiveModel {
		name: Set(deck.name.clone()),
		creator: Set(user.user_id),
		kind: Set(deck.kind.clone()),
		share: Set(deck.share.clone()),
		..Default::default()
	})
	.exec(&conn)
	.await
	.map_err(internal_error)?;
	deck.uid = res.last_insert_id.clone();

	Ok((
		StatusCode::CREATED,
		[(LOCATION, res.last_insert_id)],
		Json(deck),
	))
}

// async fn all(
// 	State(db): State<DatabaseConnection>,
// ) -> Result<impl IntoResponse, (StatusCode, String)> {
// 	Ok(todo!())
// }
//
async fn get_one(
	State(db): State<DatabaseConnection>,
	Extension(user): Extension<CurrentUser>,
	Path(id): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let deck = entity::Deck::find_by_id(id)
		.filter(
			deck::Column::Share.eq(Share::Public).or(deck::Column::Share
				.eq(Share::Private)
				.and(deck::Column::Creator.eq(user.user_id))),
		)
		.one(&db)
		.await
		.map_err(internal_error)?
		.ok_or_else(|| (StatusCode::NOT_FOUND, "Not found".to_string()))?;

	match deck.share {
		Share::Public => Ok(Json(deck)),
		Share::Private => {
			if user.user_id == deck.creator {
				Ok(Json(deck))
			} else {
				Err((StatusCode::NOT_FOUND, "Not found".to_string()))
			}
		}
	}
}

async fn update(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<CurrentUser>,
	Path(uid): Path<String>,
	Json(body): Json<deck::Model>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let deck: deck::ActiveModel = entity::Deck::find_by_id(uid)
		.filter(deck::Column::Creator.eq(user.user_id))
		.one(&conn)
		.await
		.map_err(internal_error)?
		.ok_or((StatusCode::NOT_FOUND, "Not found.".to_string()))?
		.into();

	entity::Deck::update(deck::ActiveModel {
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

async fn get_cards(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<CurrentUser>,
	Path(uid): Path<String>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let Some(deck) = entity::Deck::find_by_id(uid)
		.filter(
			deck::Column::Share.eq(Share::Public).or(deck::Column::Share
				.eq(Share::Private)
				.and(deck::Column::Creator.eq(user.user_id))),
		)
		.one(&conn)
		.await
		.map_err(internal_error)?
	else {
		return Err((StatusCode::NOT_FOUND, "Not found".to_string()));
	};

	let cards = deck
		.find_related(entity::FlashCard)
		.filter(
			flash_card::Column::Share
				.eq(Share::Public)
				.or(flash_card::Column::Share
					.eq(Share::Private)
					.and(flash_card::Column::Creator.eq(user.user_id))),
		)
		.all(&conn)
		.await
		.map_err(internal_error)?;
	Ok((StatusCode::OK, Json(cards)))
}

async fn delete_deck(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<CurrentUser>,
	Path(uid): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
	entity::Deck::delete(deck::ActiveModel {
		uid: ActiveValue::Set(uid),
		..Default::default()
	})
	.filter(flash_card::Column::Creator.eq(user.user_id))
	.exec(&conn)
	.await
	.map_err(internal_error)?;
	Ok(StatusCode::NO_CONTENT)
}

async fn update_cards(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<CurrentUser>,
	Path(uid): Path<String>,
	Json(ids): Json<Vec<String>>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let Some(deck) = entity::Deck::find_by_id(uid)
		.filter(deck::Column::Creator.eq(user.user_id))
		.one(&conn)
		.await
		.map_err(internal_error)?
	else {
		return Err((StatusCode::NOT_FOUND, "Not found".to_string()));
	};

	entity::DeckCards::delete_many()
		.filter(deck_cards::Column::Deck.eq(&deck.uid))
		.exec(&conn)
		.await
		.map_err(internal_error)?;

	entity::DeckCards::insert_many(ids.into_iter().map(|id| deck_cards::ActiveModel {
		card: Set(id),
		deck: Set(deck.uid.clone()),
	}))
	.exec(&conn)
	.await
	.map_err(internal_error)?;
	Ok(StatusCode::NO_CONTENT)
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", post(create))
		// .route("/", get(all))
		.route("/:id", get(get_one))
		.route("/:id", put(update))
		.route("/:id", delete(delete_deck))
		.route("/:id/cards", get(get_cards))
		.route("/:id/cards", put(update_cards))
		.route_layer(middleware::from_fn(session::auth))
}
