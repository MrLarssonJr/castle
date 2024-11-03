use lemonade_model::SessionToken;
use orion::pwhash::PasswordHash;
use snafu::{ResultExt, Whatever};
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create_session(
	db: &PgPool,
	user_id: Uuid,
	session_token: &SessionToken,
) -> Result<Uuid, Whatever> {
	let session_id = session_token.session_id();
	let token_hash = session_token.hash()?;

	let (id,) = sqlx::query_as(
		"insert into sessions (id, user_id, token_hash) values ($1, $2, $3) returning id;",
	)
	.bind(session_id)
	.bind(user_id)
	.bind(token_hash.unprotected_as_encoded())
	.fetch_one(db)
	.await
	.with_whatever_context(|_| {
		format!("could not create session with id {session_id} for user id {user_id}")
	})?;

	Ok(id)
}

pub async fn get_sessions(
	db: &PgPool,
	user_id: Uuid,
	limit: i64,
	offset: i64,
) -> Result<Vec<Uuid>, Whatever> {
	let rows: Vec<_> =
		sqlx::query_as("select id from sessions where user_id = $1 limit $2 offset $3;")
			.bind(user_id)
			.bind(limit)
			.bind(offset)
			.fetch_all(db)
			.await
			.with_whatever_context(|_| {
				format!("could not get sessions for user id {user_id} (limit: {limit}, offset: {offset})")
			})?;

	Ok(rows.into_iter().map(|(id,)| id).collect())
}

pub async fn delete_session(
	db: &PgPool,
	session_id: Uuid,
	user_id: Uuid,
) -> Result<Uuid, Whatever> {
	let (id,) = sqlx::query_as("delete from sessions where id = $1 and user_id = $2 returning id;")
		.bind(session_id)
		.bind(user_id)
		.fetch_one(db)
		.await
		.with_whatever_context(|_| {
			format!("could not delete session with id {session_id} for user id {user_id}")
		})?;

	Ok(id)
}

pub async fn authenticate(
	db: &PgPool,
	session_token: SessionToken,
) -> Result<Option<Uuid>, Whatever> {
	let session_id = session_token.session_id();

	let (user_id, password_hash): (_, String) =
		sqlx::query_as("SELECT user_id, token_hash FROM sessions WHERE id = $1;")
			.bind(session_id)
			.fetch_one(db)
			.await
			.with_whatever_context(|_| format!("could not get a session with id {session_id}"))?;

	let password_hash = PasswordHash::from_encoded(&password_hash).with_whatever_context(|_| {
		format!("got invalid password hash from database ({password_hash})")
	})?;

	if orion::pwhash::hash_password_verify(&password_hash, session_token.password()).is_err() {
		Ok(None)
	} else {
		Ok(Some(user_id))
	}
}
