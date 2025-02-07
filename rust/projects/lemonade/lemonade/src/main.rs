use axum::Router;
use futures::TryStreamExt;
use lemonade_api::AppState;
use lemonade_db::Database;
use lemonade_model::{AccessToken, Expiring, RefreshToken, Secret, SecretId, SecretKey};
use lemonade_nordigen::NordigenClient;
use reqwest::Url;
use snafu::{Report, ResultExt, Whatever};
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
	tracing_subscriber::FmtSubscriber::builder()
		.with_max_level(tracing::Level::INFO)
		.init();

	let db = Database::new("postgresql://localhost:5432/lemonade")
		.await
		.unwrap();

	let app = Router::new()
		.nest("/users", lemonade_api::users::handlers())
		.nest("/sessions", lemonade_api::sessions::handlers())
		.with_state(AppState { db: db.clone() });

	let listener = TcpListener::bind("0.0.0.0:0").await.unwrap();

	let access_token = Arc::new(RwLock::new(None));

	// intentionally leak secret to ensure static lifetime
	let secret = Box::leak(Box::new(Secret {
		id: SecretId("90a69619-99c1-4037-b519-6cd1ea36bfc3".into()),
		key: SecretKey("613e4a271209474c934aa3d3235c844f1b1621a48b08f6d81ab4376cca098185eec740b3139910d3fd2de6bcdbc53798ff48fc5ac15a945306ec5643c1a7fec1".into()),
	}));

	let nordigen_client =
		NordigenClient::new(Url::parse("https://bankaccountdata.gocardless.com/api/v2/").unwrap());

	tokio::spawn({
		let db = db.clone();
		let access_token = access_token.clone();
		let secret_id = &secret.id;

		async move {
			let res = db
				.observe_access_token(secret_id)
				.try_for_each(|new_access_token| async {
					{
						let mut access_token = access_token.write().expect("lock not poisoned");
						tracing::info!("got new access token {:?}", new_access_token);
						*access_token = new_access_token;
					}

					Ok(())
				})
				.await;

			if let Err(_) = res {
				tracing::error!("observe access token failed");
			}
		}
	});

	tokio::spawn({
		let db = db.clone();
		let nordigen_client = nordigen_client.clone();
		let secret: &Secret = secret;

		struct Updater(NordigenClient);

		impl Updater {
			async fn update(
				&self,
				secret: &Secret,
				current_tokens: Option<&(Expiring<AccessToken>, Expiring<RefreshToken>)>,
			) -> Result<(Expiring<AccessToken>, Expiring<RefreshToken>), Whatever> {
				let Some((current_expiring_access, current_expiring_refresh)) = current_tokens
				else {
					return self.0.new_token(secret).await.with_whatever_context(|_| format!("failed to update tokens for secret id {}, failed to fetch new token pair when none existed", secret.id.0));
				};

				let Some(current_refresh) = current_expiring_refresh.as_ref() else {
					// current refresh token is expired, fetch entirely new token pair
					return self.0.new_token(secret).await.with_whatever_context(|_| format!("failed to update tokens for secret id {}, failed to fetch new token when current refresh token was expired", secret.id.0));
				};

				let Some(_) = current_expiring_access.as_ref() else {
					// current access token is expired, fetch new one
					let new_access = self.0.refresh_token(current_refresh).await?;

					return Ok((new_access, current_expiring_refresh.clone()));
				};

				// Both current access and refresh token are still valid, no need to fetch new
				Ok((
					current_expiring_access.clone(),
					current_expiring_refresh.clone(),
				))
			}
		}

		let updater = Updater(nordigen_client);

		async move {
			loop {
				tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

				let mut tx = match db
					.begin_transaction()
					.await
					.with_whatever_context::<_, _, Whatever>(|_| "")
				{
					Ok(tx) => tx,
					Err(err) => {
						tracing::warn!("{}", Report::from_error(err));
						continue;
					}
				};

				let mut conn = tx.conn();

				if let Err(err) = conn
					.create_token(&secret.id)
					.await
					.with_whatever_context::<_, _, Whatever>(|_| "")
				{
					tracing::warn!("{}", Report::from_error(err));
					continue;
				}

				let pair = match conn
					.get_token_for_update(&secret.id)
					.await
					.with_whatever_context::<_, _, Whatever>(|_| "")
				{
					Ok(Some(pair)) => pair,
					Ok(None) => {
						tracing::info!("other replica is holding lock on token pair");
						continue;
					}
					Err(err) => {
						tracing::warn!("{}", Report::from_error(err));
						continue;
					}
				};

				let new_pair = match updater
					.update(secret, pair.as_ref())
					.await
					.with_whatever_context::<_, _, Whatever>(|_| "")
				{
					Ok(new_pair) => new_pair,
					Err(err) => {
						tracing::warn!("{}", Report::from_error(err));
						continue;
					}
				};

				if pair.as_ref() == Some(&new_pair) {
					tracing::info!("token pair not changed, skipping update");
					continue;
				}

				let (access, refresh) = new_pair;

				if let Err(err) = conn.update_token(&secret.id, access, refresh).await {
					tracing::warn!("{}", Report::from_error(err));
					continue;
				}

				if let Err(err) = tx
					.commit_transaction()
					.await
					.with_whatever_context::<_, _, Whatever>(|_| "")
				{
					tracing::warn!("{}", Report::from_error(err));
					continue;
				}

				tracing::info!("token pair updated");
			}
		}
	});

	axum::serve(listener, app).await.unwrap();
}
