use crate::Connection;
use futures::TryStreamExt;
use lemonade_model::{BasicCredential, User};
use orion::pwhash::PasswordHash;
use snafu::{ResultExt, Whatever};
use std::ops::DerefMut;
use uuid::Uuid;

impl<'c> Connection<'c> {
	pub async fn create_user(
		&mut self,
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
			.fetch_one(self.deref_mut())
			.await.with_whatever_context(|_| format!("could not create user with id {id}"))?;

		Ok(User { id, username })
	}

	pub async fn get_users(&mut self, limit: i64, offset: i64) -> Result<Vec<User>, Whatever> {
		let rows = sqlx::query_as("select id, username from users limit $1 offset $2;")
			.bind(limit)
			.bind(offset)
			.fetch(self.deref_mut())
			.map_ok(|(id, username)| User { id, username })
			.try_collect()
			.await
			.with_whatever_context(|_| {
				format!("could not get users (limit: {limit}, offset: {offset})")
			})?;

		Ok(rows)
	}

	pub async fn get_user(&mut self, user_id: Uuid) -> Result<User, Whatever> {
		let (id, username) = sqlx::query_as("select id, username from users where id = $1;")
			.bind(user_id)
			.fetch_one(self.deref_mut())
			.await
			.with_whatever_context(|_| format!("could not get user with id {user_id}"))?;

		Ok(User { id, username })
	}

	pub async fn delete_user(&mut self, user_id: Uuid) -> Result<User, Whatever> {
		let (id, username) =
			sqlx::query_as("delete from users where id = $1 returning id, username;")
				.bind(user_id)
				.fetch_one(self.deref_mut())
				.await
				.with_whatever_context(|_| format!("could not delete user with id {user_id}"))?;

		Ok(User { id, username })
	}

	pub async fn update_user(
		&mut self,
		user_id: Uuid,
		username: Option<&str>,
	) -> Result<User, Whatever> {
		let (id, username) = sqlx::query_as(
			"UPDATE users SET username = COALESCE($1, username) WHERE id = $2 RETURNING id, username;",
		)
			.bind(username)
			.bind(user_id)
			.fetch_one(self.deref_mut())
			.await
			.with_whatever_context(|_| format!("could not update user with id {user_id}"))?;

		Ok(User { id, username })
	}

	pub async fn authenticate_basic_credential(
		&mut self,
		basic_credential: &BasicCredential,
	) -> Result<Option<Uuid>, Whatever> {
		let (user_id, password_hash): (_, String) =
			sqlx::query_as("SELECT id, password_hash FROM users WHERE username = $1;")
				.bind(basic_credential.username())
				.fetch_one(self.deref_mut())
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
