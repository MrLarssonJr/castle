use crate::{ApiError, AppState, Authentication, PageDto, PageQueryOptions};
use axum::extract::{Path, Query, State};
use axum::routing::{delete, post};
use axum::{Json, Router};
use lemonade_model::SessionToken;
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
	let mut conn = app_state.db.conn().await?;

	let session_token = SessionToken::new(Uuid::now_v7())?;
	let session_id = conn.create_session(user_id, &session_token).await?;

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
	let mut conn = app_state.db.conn().await?;
	let sessions = conn.get_sessions(user_id, limit, offset).await?;
	let sessions = sessions.into_iter().map(|id| SessionDto { id }).collect();
	let page = PageDto { items: sessions };
	Ok(Json(page))
}

async fn delete_session(
	State(app_state): State<AppState>,
	Authentication { user_id }: Authentication,
	Path(session_id): Path<Uuid>,
) -> Result<Json<SessionDto>, ApiError> {
	let mut conn = app_state.db.conn().await?;
	let id = conn.delete_session(session_id, user_id).await?;
	Ok(Json(SessionDto { id }))
}
