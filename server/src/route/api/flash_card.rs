use axum::{
	extract::{Path, State},
	http::{header::LOCATION, StatusCode},
	middleware,
	response::IntoResponse,
	routing::{delete, get, post, put},
	Extension, Json, Router,
};
use sea_orm::{ActiveValue::Set, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use uuid::Uuid;

use crate::{app::AppState, internal_error, session, session::CurrentUser};
use entity::{flash_card, prelude::*, sea_orm_active_enums::Share};

async fn create(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<CurrentUser>,
	Json(mut body): Json<flash_card::Model>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	body.uid = Uuid::new_v4();
	body.creator = user.user_id;

	FlashCard::insert(flash_card::ActiveModel {
		uid: Set(body.uid),
		creator: Set(body.creator),
		share: Set(body.share),
		content: Set(body.content.clone()),
	})
	.exec(&conn)
	.await
	.map_err(internal_error)?;

	Ok((
		StatusCode::CREATED,
		[(LOCATION, body.uid.to_string())],
		Json(body),
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
	Path(uuid): Path<Uuid>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let flashcard = FlashCard::find_by_id(uuid)
		.one(&db)
		.await
		.map_err(internal_error)?
		.ok_or_else(|| (StatusCode::NOT_FOUND, "Not found".to_string()))?;

	match flashcard.share {
		Share::Public => Ok(Json(flashcard)),
		Share::Private => {
			if user.user_id == flashcard.creator {
				Ok(Json(flashcard))
			} else {
				Err((StatusCode::NOT_FOUND, "Not found".to_string()))
			}
		}
	}
}

async fn update(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<CurrentUser>,
	Path(uuid): Path<Uuid>,
	Json(body): Json<flash_card::Model>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let flashcard = flash_card::ActiveModel {
		uid: Set(uuid),
		share: Set(body.share),
		content: Set(body.content.clone()),
		..Default::default()
	};

	FlashCard::update(flashcard)
		.filter(flash_card::Column::Creator.eq(user.user_id))
		.exec(&conn)
		.await
		.map_err(internal_error)?;
	Ok(StatusCode::NO_CONTENT)
}

async fn delete_card(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<CurrentUser>,
	Path(uuid): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
	if FlashCard::delete(flash_card::ActiveModel {
		uid: Set(uuid),
		..Default::default()
	})
	.filter(flash_card::Column::Creator.eq(user.user_id))
	.exec(&conn)
	.await
	.map_err(internal_error)?
	.rows_affected
		== 0
	{
		Ok(StatusCode::NOT_FOUND)
	} else {
		Ok(StatusCode::NO_CONTENT)
	}
}

pub fn router() -> Router<AppState> {
	Router::new()
		.route("/", post(create))
		// .route("/", get(all))
		.route("/:id", get(get_one))
		.route("/:id", put(update))
		.route("/:id", delete(delete_card))
		.route_layer(middleware::from_fn(session::auth))
}
