use crate::{ApiError, AppState, Authentication, PageDto, PageQueryOptions};
use axum::extract::{Path, Query, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use lemonade_model::{Role, User};
use orion::pwhash::Password;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Whatever};
use uuid::Uuid;

pub fn handlers() -> Router<AppState> {
	Router::new()
		.route("/", post(create_user).get(get_users))
		.route("/me", get(get_me).delete(delete_me).put(update_me))
		.route("/:id", get(get_user).delete(delete_user).patch(update_user))
}

#[derive(Debug, Serialize)]
struct UserDto {
	id: Uuid,
	username: String,
}

#[derive(Debug, Deserialize)]
struct CreateUserDto {
	username: String,
	password: String,
}

#[derive(Debug, Deserialize)]
struct UpdateUserDto {
	username: Option<String>,
}

async fn create_user(
	State(app_state): State<AppState>,
	Authentication { user_id }: Authentication,
	Json(CreateUserDto { username, password }): Json<CreateUserDto>,
) -> Result<Json<UserDto>, ApiError> {
	let mut conn = app_state.db.conn().await?;

	let is_admin = conn.has_role(user_id, Role::Admin).await?;

	if !is_admin {
		return Err(ApiError::Unauthorized);
	}

	let password = Password::from_slice(password.as_bytes())
		.with_whatever_context::<_, _, Whatever>(|_| {
			"unable to create users, due to unable to create password"
		})?;
	let password_hash = orion::pwhash::hash_password(&password, 8, 16)
		.with_whatever_context::<_, _, Whatever>(|_| {
			"unable to create user, due to unable to hash password"
		})?;

	let User { id, username } = conn.create_user(&username, password_hash).await?;

	Ok(Json(UserDto { id, username }))
}

async fn get_users(
	State(app_state): State<AppState>,
	Authentication { user_id }: Authentication,
	Query(PageQueryOptions { limit, offset }): Query<PageQueryOptions>,
) -> Result<Json<PageDto<UserDto>>, ApiError> {
	let mut conn = app_state.db.conn().await?;

	let is_admin = conn.has_role(user_id, Role::Admin).await?;

	if !is_admin {
		return Err(ApiError::Unauthorized);
	}

	let users = conn
		.get_users(limit, offset)
		.await?
		.into_iter()
		.map(|User { id, username }| UserDto { id, username })
		.collect();

	let page = PageDto { items: users };

	Ok(Json(page))
}

async fn get_user(
	State(app_state): State<AppState>,
	Authentication { user_id }: Authentication,
	Path(id): Path<Uuid>,
) -> Result<Json<UserDto>, ApiError> {
	let mut conn = app_state.db.conn().await?;

	let is_admin = conn.has_role(user_id, Role::Admin).await?;
	let is_own_user = user_id == id;

	if !(is_own_user || is_admin) {
		return Err(ApiError::Unauthorized);
	}

	let User { id, username } = conn.get_user(id).await?;

	Ok(Json(UserDto { id, username }))
}

async fn delete_user(
	State(app_state): State<AppState>,
	Authentication { user_id }: Authentication,
	Path(id): Path<Uuid>,
) -> Result<Json<UserDto>, ApiError> {
	let mut conn = app_state.db.conn().await?;

	let is_admin = conn.has_role(user_id, Role::Admin).await?;
	let is_own_user = user_id == id;

	if !(is_own_user || is_admin) {
		return Err(ApiError::Unauthorized);
	}

	let User { id, username } = conn.delete_user(id).await?;

	Ok(Json(UserDto { id, username }))
}

async fn update_user(
	State(app_state): State<AppState>,
	Authentication { user_id }: Authentication,
	Path(id): Path<Uuid>,
	Json(UpdateUserDto { username }): Json<UpdateUserDto>,
) -> Result<Json<UserDto>, ApiError> {
	let mut conn = app_state.db.conn().await?;

	let is_admin = conn.has_role(user_id, Role::Admin).await?;
	let is_own_user = user_id == id;

	if !(is_own_user || is_admin) {
		return Err(ApiError::Unauthorized);
	}

	let User { id, username } = conn.update_user(id, username.as_deref()).await?;

	Ok(Json(UserDto { id, username }))
}

async fn get_me(
	state: State<AppState>,
	Authentication { user_id }: Authentication,
) -> Result<Json<UserDto>, ApiError> {
	get_user(state, Authentication { user_id }, Path(user_id)).await
}

async fn delete_me(
	state: State<AppState>,
	Authentication { user_id }: Authentication,
) -> Result<Json<UserDto>, ApiError> {
	delete_user(state, Authentication { user_id }, Path(user_id)).await
}

async fn update_me(
	state: State<AppState>,
	Authentication { user_id }: Authentication,
	body: Json<UpdateUserDto>,
) -> Result<Json<UserDto>, ApiError> {
	update_user(state, Authentication { user_id }, Path(user_id), body).await
}
