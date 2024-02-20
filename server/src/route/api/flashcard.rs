use crate::app::AppState;
use crate::dto::flashcard::FlashCard;
use crate::entities::flash_card;
use crate::entities::prelude as entity;
use crate::entities::sea_orm_active_enums::Share;
use crate::session::CurrentUser;
use crate::{internal_error, session};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{
	extract::Path,
	http::{header::LOCATION, StatusCode},
	middleware,
	routing::{delete, get, post, put},
	Extension, Json, Router,
};
use futures::stream::TryStreamExt;
use sea_orm::{ActiveValue, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};

async fn create(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<CurrentUser>,
	Json(mut body): Json<FlashCard>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let flashcard = flash_card::ActiveModel {
		creator: ActiveValue::Set(user.user_id),
		share: ActiveValue::Set(body.share.clone()),
		content: ActiveValue::Set(serde_json::to_string(&body.content).map_err(internal_error)?),
		..Default::default()
	};

	let res = entity::FlashCard::insert(flashcard)
		.exec(&conn)
		.await
		.map_err(internal_error)?;

	body.id = Some(res.last_insert_id);

	Ok((
		StatusCode::CREATED,
		[(LOCATION, res.last_insert_id)],
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
	Path(id): Path<u32>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let flashcard = entity::FlashCard::find_by_id(id)
		.one(&db)
		.await
		.map_err(internal_error)?
		.ok_or_else(|| (StatusCode::NOT_FOUND, "Not found".to_string()))?;

	match flashcard.share {
		Share::Protected | Share::Public => Ok(Json(FlashCard {
			id: Some(flashcard.id),
			share: flashcard.share,
			content: serde_json::from_str(&flashcard.content).map_err(internal_error)?,
		})),
		Share::Private => {
			if user.user_id == flashcard.creator {
				Ok(Json(FlashCard {
					id: Some(flashcard.id),
					share: flashcard.share,
					content: serde_json::from_str(&flashcard.content).map_err(internal_error)?,
				}))
			} else {
				Err((StatusCode::NOT_FOUND, "Not found".to_string()))
			}
		}
	}
}

async fn update(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<CurrentUser>,
	Path(id): Path<u32>,
	Json(body): Json<FlashCard>,
) -> Result<impl IntoResponse, (StatusCode, String)> {
	let flashcard = flash_card::ActiveModel {
		id: ActiveValue::Set(id),
		share: ActiveValue::Set(body.share.clone()),
		content: ActiveValue::Set(serde_json::to_string(&body.content).map_err(internal_error)?),
		..Default::default()
	};

	entity::FlashCard::update(flashcard)
		.filter(flash_card::Column::Creator.eq(user.user_id))
		.exec(&conn)
		.await
		.map_err(internal_error)?;
	Ok(StatusCode::NO_CONTENT)
}

async fn del(
	State(conn): State<DatabaseConnection>,
	Extension(user): Extension<CurrentUser>,
	Path(id): Path<u32>,
) -> Result<StatusCode, (StatusCode, String)> {
	entity::FlashCard::delete(flash_card::ActiveModel {
		id: ActiveValue::Set(id),
		..Default::default()
	})
	.filter(flash_card::Column::Creator.eq(user.user_id))
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
		.route("/:id", delete(del))
		.route_layer(middleware::from_fn(session::auth))
}
