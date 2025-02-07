use crate::{Connection, Database};
use async_stream::try_stream;
use chrono::{DateTime, Utc};
use futures::Stream;
use lemonade_model::{AccessToken, ExpiresAt, Expiring, RefreshToken, SecretId};
use snafu::{whatever, ResultExt, Whatever};
use sqlx::postgres::PgListener;
use std::ops::DerefMut;

impl Database {
	pub fn observe_access_token<'l>(
		&'l self,
		secret_id: &'l SecretId,
	) -> impl Stream<Item = Result<Option<Expiring<AccessToken>>, ()>> + 'l + Send {
		try_stream! {
			let mut listener = PgListener::connect_with(&self.connection_pool)
			.await
			.with_whatever_context::<_, _, Whatever>(|_| {
				format!(
					"failed to keep access token for secret id {} updated, failed to create listener",
					secret_id.0
				)
			}).map_err(|_| ())?;

			listener.listen("nordigen_tokens")
				.await
				.with_whatever_context::<_, _, Whatever>(|_| format!("failed to keep access token for secret id {} updated, failed to register to channel", secret_id.0))
				.map_err(|_| ())?;

			loop {
				let new_access_token = self.conn()
					.await
					.with_whatever_context::<_, _, Whatever>(|_| "")
					.map_err(|_| ())?
					.get_access_token(secret_id)
					.await
					.with_whatever_context::<_, _, Whatever>(|_| format!("failed to keep access token for secret id {} updated, failed to fetch access token", secret_id.0))
					.map_err(|_| ())?;

				yield new_access_token;

				let _ = listener.try_recv()
					.await
					.with_whatever_context::<_, _, Whatever>(|_| format!("failed to keep access token for secret id {} updated, failed to receive notification", secret_id.0))
					.map_err(|_| ())?;
			}
		}
	}
}

impl<'c> Connection<'c> {
	async fn get_access_token(
		&mut self,
		secret_id: &SecretId,
	) -> Result<Option<Expiring<AccessToken>>, Whatever> {
		let secret_id = &secret_id.0;

		let access_token = sqlx::query_as(
			"SELECT access, access_expires_at FROM nordigen_tokens WHERE secret_id = $1;",
		)
			.bind(secret_id.as_ref())
			.fetch_optional(self.deref_mut())
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

	pub async fn create_token(&mut self, secret_id: &SecretId) -> Result<(), Whatever> {
		let secret_id = &secret_id.0;

		sqlx::query("INSERT INTO nordigen_tokens VALUES ($1, NULL, NULL, NULL, NULL) ON CONFLICT DO NOTHING")
			.bind(secret_id.as_ref())
			.execute(self.deref_mut())
			.await
			.with_whatever_context(|_| format!("failed to update token for secret with id {secret_id}, failed to ensure row exists for secret"))?;

		Ok(())
	}

	pub async fn get_token_for_update(
		&mut self,
		secret_id: &SecretId,
	) -> Result<Option<Option<(Expiring<AccessToken>, Expiring<RefreshToken>)>>, Whatever> {
		let secret_id = &secret_id.0;

		let row =
			sqlx::query_as("SELECT access, access_expires_at, refresh, refresh_expires_at FROM nordigen_tokens WHERE secret_id = $1 FOR UPDATE SKIP LOCKED;")
				.bind(secret_id.as_ref())
				.fetch_optional(self.deref_mut())
				.await
				.with_whatever_context(|_| format!("failed to update token for secret with id {secret_id}, failed to get row"))?;

		let Some((access, access_expiry, refresh, refresh_expiry)) = row else {
			// Other replica holds lock on row and is currently performing update
			tracing::info!(
				"skipping token pair update, other replicas is currently performing update"
			);
			return Ok(None);
		};

		let (access, access_expiry, refresh, refresh_expiry): (Box<str>, DateTime<Utc>, Box<str>, DateTime<Utc>) = match (access, access_expiry, refresh, refresh_expiry) {
			(Some(access), Some(access_expiry), Some(refresh), Some(refresh_expiry)) => (access, access_expiry, refresh, refresh_expiry),
			(None, None, None, None) => return Ok(Some(None)),
			_ => whatever!("failed to update token for secret with id {secret_id}, expected either all or none of the fields to be null"),
		};

		let access_token = AccessToken(access.into()).expires_at(access_expiry);
		let refresh_token = RefreshToken(refresh.into()).expires_at(refresh_expiry);

		Ok(Some(Some((access_token, refresh_token))))
	}

	pub async fn update_token(
		&mut self,
		secret_id: &SecretId,
		access: Expiring<AccessToken>,
		refresh: Expiring<RefreshToken>,
	) -> Result<(), Whatever> {
		let secret_id = &secret_id.0;
		let (access, access_expires_at) = access.into_parts();
		let (refresh, refresh_expires_at) = refresh.into_parts();

		sqlx::query("UPDATE nordigen_tokens SET access = $1, access_expires_at = $2, refresh = $3, refresh_expires_at = $4 WHERE secret_id = $5")
			.bind(access.0.as_ref())
			.bind(access_expires_at)
			.bind(refresh.0.as_ref())
			.bind(refresh_expires_at)
			.bind(secret_id.as_ref())
			.execute(self.deref_mut())
			.await.with_whatever_context(|_| format!("failed to update token for secret with id {secret_id}, failed to update row with new values"))?;

		Ok(())
	}
}
