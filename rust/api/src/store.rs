//! The purpose of gather all SQL into one module is to simplify
//! schema updates.
//!
//! Initially there's supposed to one module herewithin for each table in the schema,
//! and each module should contain the statements related to that table. Of course, this
//! will break down when there multi-table statements become required. But problem for then…
//!
//! This simplifies schema updates because if a table schema changes, then one will only have
//! to refactor into the relevant modules herewithin. Any consequent internal API changes should
//! (hopefully) be caught by the compiler.

use crate::model::ExpiresAt;
use snafu::ResultExt;
use std::future::Future;

pub mod sessions {
	use crate::model::SessionToken;
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
		let (id,) =
			sqlx::query_as("delete from sessions where id = $1 and user_id = $2 returning id;")
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
				.with_whatever_context(|_| {
					format!("could not get a session with id {session_id}")
				})?;

		let password_hash =
			PasswordHash::from_encoded(&password_hash).with_whatever_context(|_| {
				format!("got invalid password hash from database ({password_hash})")
			})?;

		if orion::pwhash::hash_password_verify(&password_hash, session_token.password()).is_err() {
			Ok(None)
		} else {
			Ok(Some(user_id))
		}
	}
}

pub mod users {
	use crate::model::{BasicCredential, User};
	use futures::TryStreamExt;
	use orion::pwhash::PasswordHash;
	use snafu::{ResultExt, Whatever};
	use sqlx::PgPool;
	use uuid::Uuid;

	pub async fn create_user(
		db: &PgPool,
		username: &str,
		password_hash: PasswordHash,
	) -> Result<User, Whatever> {
		let id = Uuid::now_v7();

		let (id, username) = sqlx::query_as(
			"insert into users (id, username, password_hash) values ($1, $2, $3) returning id, username",
		)
			.bind(id)
			.bind(username)
			.bind(password_hash.unprotected_as_encoded())
			.fetch_one(db)
			.await.with_whatever_context(|_| format!("could not create user with id {id}"))?;

		Ok(User { id, username })
	}

	pub async fn get_users(db: &PgPool, limit: i64, offset: i64) -> Result<Vec<User>, Whatever> {
		let rows = sqlx::query_as("select id, username from users limit $1 offset $2;")
			.bind(limit)
			.bind(offset)
			.fetch(db)
			.map_ok(|(id, username)| User { id, username })
			.try_collect()
			.await
			.with_whatever_context(|_| {
				format!("could not get users (limit: {limit}, offset: {offset})")
			})?;

		Ok(rows)
	}

	pub async fn get_user(db: &PgPool, user_id: Uuid) -> Result<User, Whatever> {
		let (id, username) = sqlx::query_as("select id, username from users where id = $1;")
			.bind(user_id)
			.fetch_one(db)
			.await
			.with_whatever_context(|_| format!("could not get user with id {user_id}"))?;

		Ok(User { id, username })
	}

	pub async fn delete_user(db: &PgPool, user_id: Uuid) -> Result<User, Whatever> {
		let (id, username) =
			sqlx::query_as("delete from users where id = $1 returning id, username;")
				.bind(user_id)
				.fetch_one(db)
				.await
				.with_whatever_context(|_| format!("could not delete user with id {user_id}"))?;

		Ok(User { id, username })
	}

	pub async fn update_user(
		db: &PgPool,
		user_id: Uuid,
		username: Option<&str>,
	) -> Result<User, Whatever> {
		let (id, username) = sqlx::query_as(
			"UPDATE users SET username = COALESCE($1, username) WHERE id = $2 RETURNING id, username;",
		)
			.bind(username)
			.bind(user_id)
			.fetch_one(db)
			.await
			.with_whatever_context(|_| format!("could not update user with id {user_id}"))?;

		Ok(User { id, username })
	}

	pub async fn authenticate(
		db: &PgPool,
		basic_credential: &BasicCredential,
	) -> Result<Option<Uuid>, Whatever> {
		let (user_id, password_hash): (_, String) =
			sqlx::query_as("SELECT id, password_hash FROM users WHERE username = $1;")
				.bind(basic_credential.username())
				.fetch_one(db)
				.await
				.with_whatever_context(|_| "failed to authenticate user, db error")?;

		let password_hash =
			PasswordHash::from_encoded(&password_hash).with_whatever_context(|_| {
				format!("failed to authenticate user, got invalid password hash from db {password_hash}")
			})?;

		if orion::pwhash::hash_password_verify(&password_hash, basic_credential.password()).is_err()
		{
			Ok(None)
		} else {
			Ok(user_id)
		}
	}
}

pub mod roles {
	use crate::model::Role;
	use snafu::{ResultExt, Whatever};
	use sqlx::PgPool;
	use uuid::Uuid;

	pub async fn has_role(db: &PgPool, user_id: Uuid, role: Role) -> Result<bool, Whatever> {
		let (has_role,) =
			sqlx::query_as("SELECT EXISTS(SELECT * FROM roles WHERE user_id = $1 AND role = $2);")
				.bind(user_id)
				.bind(role)
				.fetch_one(db)
				.await
				.with_whatever_context(|_| {
					format!("failed to check if user with id {user_id} had role {role:?}")
				})?;

		Ok(has_role)
	}
}

pub mod nordigen_tokens {
	use crate::model::{AccessToken, ExpiresAt, Expiring, RefreshToken, Secret, SecretId};
	use chrono::{DateTime, Utc};
	use futures::never::Never;
	use snafu::{whatever, ResultExt, Whatever};
	use sqlx::postgres::PgListener;
	use sqlx::PgPool;
	use std::future::Future;
	use std::sync::{Arc, RwLock};

	async fn get_access_token(
		db: &PgPool,
		secret_id: &SecretId,
	) -> Result<Option<Expiring<AccessToken>>, Whatever> {
		let secret_id = &secret_id.0;

		let access_token = sqlx::query_as(
			"SELECT access, access_expires_at FROM nordigen_tokens WHERE secret_id = $1;",
		)
		.bind(secret_id.as_ref())
		.fetch_optional(db)
		.await
		.with_whatever_context(|_| {
			format!("failed to get access token for secret id {secret_id}, query failed")
		})?
		.map(|(access_token, expires_at): (Option<Box<str>>, Option<DateTime<Utc>>)| {
			let (access_token, expires_at) = match (access_token, expires_at) {
				(Some(access_token), Some(expires_at)) => (access_token, expires_at),
				(None, None) => return Ok(None),
				_ => whatever!("failed to get access token for secret id {secret_id}, expected both fields or none to be null"),
			};

			Ok(Some(AccessToken(access_token.into()).expires_at(expires_at)))
		}).transpose()?.flatten();

		Ok(access_token)
	}

	pub async fn keep_access_token_update(
		db: &PgPool,
		secret_id: &SecretId,
		access_token: Arc<RwLock<Option<Expiring<AccessToken>>>>,
	) -> Result<Never, Whatever> {
		let mut listener = PgListener::connect_with(db).await.with_whatever_context(|_| format!("failed to keep access token for secret id {} updated, failed to create listener", secret_id.0))?;
		listener.listen("nordigen_tokens").await.with_whatever_context(|_| format!("failed to keep access token for secret id {} updated, failed to register to channel", secret_id.0))?;

		loop {
			let new_access_token = get_access_token(db, secret_id).await.with_whatever_context(|_| format!("failed to keep access token for secret id {} updated, failed to fetch access token", secret_id.0))?;

			tracing::info!("new access token: {new_access_token:?}");

			{
				let mut access_token = access_token.write().expect("lock not to be poisoned");
				*access_token = new_access_token;
			}

			let _ = listener.try_recv().await.with_whatever_context(|_| format!("failed to keep access token for secret id {} updated, failed to receive notification", secret_id.0))?;
		}
	}

	pub trait TokenUpdater {
		fn update(
			&self,
			secret: &Secret,
			current_tokens: Option<(&Expiring<AccessToken>, &Expiring<RefreshToken>)>,
		) -> impl Future<Output = Result<(Expiring<AccessToken>, Expiring<RefreshToken>), Whatever>>;
	}

	pub async fn keep_token_updated(
		db: &PgPool,
		secret: &Secret,
		updater: &impl TokenUpdater,
	) -> Result<Never, Whatever> {
		loop {
			tracing::info!("ensuring token pair is up to date");
			update_token(db, secret, updater)
				.await
				.with_whatever_context(|_| {
					format!(
						"failed to keep token for secret id {} updated, update failed",
						secret.id.0
					)
				})?;

			tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
		}
	}

	async fn update_token(
		db: &PgPool,
		secret: &Secret,
		updater: &impl TokenUpdater,
	) -> Result<(), Whatever> {
		let secret_id = &secret.id.0;

		let mut tx = db.begin().await.with_whatever_context(|_| {
			format!("failed to update token for secret with id {secret_id}, failed to start transaction")
		})?;

		sqlx::query("INSERT INTO nordigen_tokens VALUES ($1, NULL, NULL, NULL, NULL) ON CONFLICT DO NOTHING")
			.bind(secret_id.as_ref())
			.execute(&mut *tx)
			.await
			.with_whatever_context(|_| format!("failed to update token for secret with id {secret_id}, failed to ensure row exists for secret"))?;

		let token = 'get_token: {
			type ValueField = Option<Box<str>>;
			type TsField = Option<DateTime<Utc>>;
			type Row = Option<(ValueField, TsField, ValueField, TsField)>;

			let row: Row =
				sqlx::query_as("SELECT access, access_expires_at, refresh, refresh_expires_at FROM nordigen_tokens WHERE secret_id = $1 FOR UPDATE SKIP LOCKED;")
					.bind(secret_id.as_ref())
					.fetch_optional(&mut *tx)
					.await.
					with_whatever_context(|_| format!("failed to update token for secret with id {secret_id}, failed to get row"))?;

			let Some((access, access_expiry, refresh, refresh_expiry)) = row else {
				// Other replica holds lock on row and is currently performing update
				tracing::info!(
					"skipping token pair update, other replicas is currently performing update"
				);
				return Ok(());
			};

			let (access, access_expiry, refresh, refresh_expiry) = match (access, access_expiry, refresh, refresh_expiry) {
				(Some(access), Some(access_expiry), Some(refresh), Some(refresh_expiry)) => (access, access_expiry, refresh, refresh_expiry),
				(None, None, None, None) => break 'get_token None,
				_ => whatever!("failed to update token for secret with id {secret_id}, expected either all or none of the fields to be null"),
			};

			let access_token = AccessToken(access.into()).expires_at(access_expiry);
			let refresh_token = RefreshToken(refresh.into()).expires_at(refresh_expiry);

			Some((access_token, refresh_token))
		};

		let (access, refresh) = updater
			.update(secret, token.as_ref().map(|(a, r)| (a, r)))
			.await
			.with_whatever_context(|_| {
				format!("failed to update token for secret with id {secret_id}, failed to get updated token")
			})?;

		if token.as_ref().map(|(a, r)| (a, r)) == Some((&access, &refresh)) {
			// token pair is unchanged, no need to updated db
			tracing::info!("skipping token pair update, token pair unchanged");
			return Ok(());
		}

		let (access, access_expires_at) = access.into_parts();
		let (refresh, refresh_expires_at) = refresh.into_parts();

		sqlx::query("UPDATE nordigen_tokens SET access = $1, access_expires_at = $2, refresh = $3, refresh_expires_at = $4 WHERE secret_id = $5")
			.bind(access.0.as_ref())
			.bind(access_expires_at)
			.bind(refresh.0.as_ref())
			.bind(refresh_expires_at)
			.bind(secret_id.as_ref())
			.execute(&mut *tx)
			.await.with_whatever_context(|_| format!("failed to update token for secret with id {secret_id}, failed to update row with new values"))?;

		tx.commit()
			.await
			.with_whatever_context(|_| "failed to update token, failed to commit transaction")?;

		Ok(())
	}
}
