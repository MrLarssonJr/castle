use crate::model::{AccessToken, Expiring, RefreshToken, Secret, SecretId, SecretKey};
use crate::nordigen::NordigenClient;
use crate::store::nordigen_tokens::TokenUpdater;
use axum::Router;
use reqwest::Url;
use snafu::{Report, ResultExt, Whatever};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use std::future::Future;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;

#[tokio::main]
async fn main() {
	tracing_subscriber::FmtSubscriber::builder()
		.with_max_level(tracing::Level::INFO)
		.init();

	let pool = PgPoolOptions::new()
		.connect("postgresql://localhost:5432/lemonade")
		.await
		.unwrap();

	let app = Router::new()
		.nest("/users", routes::users::handlers())
		.nest("/sessions", routes::sessions::handlers())
		.with_state(AppState { db: pool.clone() });

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
		let pool = pool.clone();
		let access_token = access_token.clone();
		let secret_id = &secret.id;
		async move {
			let Err(err): Result<_, Whatever> =
				store::nordigen_tokens::keep_access_token_update(&pool, secret_id, access_token)
					.await
					.with_whatever_context(|_| "task to keep access token updated failed");

			eprintln!("{}", Report::from_error(err));
		}
	});

	tokio::spawn({
		let pool = pool.clone();
		let nordigen_client = nordigen_client.clone();
		let secret: &Secret = secret;

		impl TokenUpdater for NordigenClient {
			async fn update(
				&self,
				secret: &Secret,
				current_tokens: Option<(&Expiring<AccessToken>, &Expiring<RefreshToken>)>,
			) -> Result<(Expiring<AccessToken>, Expiring<RefreshToken>), Whatever> {
				let Some((current_expiring_access, current_expiring_refresh)) = current_tokens
				else {
					return self.new_token(secret).await.with_whatever_context(|_| format!("failed to update tokens for secret id {}, failed to fetch new token pair when none existed", secret.id.0));
				};

				let Some(current_refresh) = current_expiring_refresh.as_ref() else {
					// current refresh token is expired, fetch entirely new token pair
					return self.new_token(secret).await.with_whatever_context(|_| format!("failed to update tokens for secret id {}, failed to fetch new token when current refresh token was expired", secret.id.0));
				};

				let Some(_) = current_expiring_access.as_ref() else {
					// current access token is expired, fetch new one
					let new_access = self.refresh_token(current_refresh).await?;

					return Ok((new_access, current_expiring_refresh.clone()));
				};

				// Both current access and refresh token are still valid, no need to fetch new
				Ok((
					current_expiring_access.clone(),
					current_expiring_refresh.clone(),
				))
			}
		}

		async move {
			let nordigen_client = nordigen_client;
			let Err(err): Result<_, Whatever> =
				store::nordigen_tokens::keep_token_updated(&pool, secret, &nordigen_client)
					.await
					.with_whatever_context(|_| "task to keep tokens updated failed");

			eprintln!("{}", Report::from_error(err));
		}
	});

	axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Clone)]
struct AppState {
	db: PgPool,
}

mod model;
mod nordigen;
mod routes;
pub mod store;
