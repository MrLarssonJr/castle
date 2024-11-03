use chrono::{DateTime, Utc};
use futures::never::Never;
use lemonade_model::{AccessToken, ExpiresAt, Expiring, RefreshToken, Secret, SecretId};
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
	let mut listener = PgListener::connect_with(db)
		.await
		.with_whatever_context(|_| {
			format!(
				"failed to keep access token for secret id {} updated, failed to create listener",
				secret_id.0
			)
		})?;
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

	let mut tx =
		db.begin().await.with_whatever_context(|_| {
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
