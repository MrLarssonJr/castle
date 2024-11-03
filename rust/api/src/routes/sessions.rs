use crate::model::SessionToken;
use crate::routes::{ApiError, Authentication, PageDto, PageQueryOptions};
use crate::{store, AppState};
use axum::extract::{Path, Query, State};
use axum::routing::{delete, post};
use axum::{Json, Router};
use serde::Serialize;
use uuid::Uuid;

pub fn handlers() -> Router<AppState> {
	Router::new()
		.route("/", post(create_session).get(get_sessions))
		.route("/:id", delete(delete_session))
}

#[derive(Debug, Serialize)]
struct CreateSessionDto {
	id: Uuid,
	token: SessionToken,
}

#[derive(Debug, Serialize)]
struct SessionDto {
	id: Uuid,
}

async fn create_session(
	State(app_state): State<AppState>,
	Authentication { user_id }: Authentication,
) -> Result<Json<CreateSessionDto>, ApiError> {
	let db = app_state.db;

	let session_token = SessionToken::new(Uuid::now_v7())?;
	let session_id = store::sessions::create_session(&db, user_id, &session_token).await?;

	Ok(Json(CreateSessionDto {
		id: session_id,
		token: session_token,
	}))
}

async fn get_sessions(
	State(app_state): State<AppState>,
	Authentication { user_id }: Authentication,
	Query(PageQueryOptions { limit, offset }): Query<PageQueryOptions>,
) -> Result<Json<PageDto<SessionDto>>, ApiError> {
	let db = app_state.db;
	let sessions = store::sessions::get_sessions(&db, user_id, limit, offset).await?;
	let sessions = sessions.into_iter().map(|id| SessionDto { id }).collect();
	let page = PageDto { items: sessions };
	Ok(Json(page))
}

async fn delete_session(
	State(app_state): State<AppState>,
	Authentication { user_id }: Authentication,
	Path(session_id): Path<Uuid>,
) -> Result<Json<SessionDto>, ApiError> {
	let db = app_state.db;
	let id = store::sessions::delete_session(&db, session_id, user_id).await?;
	Ok(Json(SessionDto { id }))
}
