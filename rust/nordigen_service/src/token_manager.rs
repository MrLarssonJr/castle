use crate::adapters::outgoing::nordigen::{Expiring, NordigenApi, NordigenApiAccessToken, Secret};
use chrono::{TimeDelta, Utc};
use error::extensions::error::ErrorExt;
use std::cmp::min;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use sync_utils::Watch;
use thiserror::Error;
use tokio::sync::Notify;
use tokio::time::sleep;
use tokio_util::sync::{CancellationToken, DropGuard};

pub struct TokenManager {
	access_token: Watch<Expiring<NordigenApiAccessToken>>,
	cancellation_token: DropGuard,
}

impl TokenManager {
	pub fn new<N: NordigenApi + Debug + Send + 'static + Sync>(
		secret: Secret,
		nordigen_api: N,
	) -> TokenManager {
		let access_token = Watch::new();
		let refresh_notify = Arc::new(Notify::new());

		let cancellation_token = CancellationToken::new();
		let join_handle = {
			let access_token = access_token.clone();
			let refresh_notify = refresh_notify.clone();
			let cancellation_token = cancellation_token.child_token();

			let supervised_token_refresher_future =
				supervisor(secret, nordigen_api, access_token, refresh_notify);

			let token_refresher_future = async move {
				cancellation_token
					.run_until_cancelled(supervised_token_refresher_future)
					.await
			};

			tokio::spawn(token_refresher_future)
		};

		TokenManager {
			access_token,
			refresh_notify,
			cancellation_token: cancellation_token.drop_guard(),
		}
	}

	pub async fn access_token(&self) -> NordigenApiAccessToken {
		self.access_token.latest(&self.refresh_notify).await
	}
}

async fn supervisor<N: NordigenApi + Debug>(
	secret: Secret,
	nordigen_api: N,
	access_token_watch: Watch<NordigenApiAccessToken>,
	refresh_notify: Arc<Notify>,
) {
	loop {
		match token_refresher(&secret, &nordigen_api, &access_token_watch, &refresh_notify).await {
			Ok(res) => break res,
			Err(err) => {
				eprintln!(
					"token refresher failed, restarting in 500ms\nerror cause: {}",
					err.to_pretty_string()
				);
				sleep(Duration::from_millis(500)).await;
			}
		}
	}
}

async fn token_refresher<N: NordigenApi>(
	secret: &Secret,
	nordigen_api: &N,
	access_token_watch: &Watch<NordigenApiAccessToken>,
	refresh_notify: &Notify,
) -> Result<(), TokenRefresherError<N>> {
	let (mut access_token, mut refresh_token) = nordigen_api
		.obtain_token_pair(&secret)
		.await
		.map_err(TokenRefresherError::Obtain)?;
	access_token_watch.update(access_token.value.clone());

	loop {
		let time_until_closest_expiry =
			Utc::now() - min(access_token.expires_at, refresh_token.expires_at);
		let next_refresh_in = (time_until_closest_expiry * 4 / 5)
			.clamp(TimeDelta::milliseconds(100), TimeDelta::hours(1));

		tokio::select! {
			_ = sleep(next_refresh_in.to_std().expect("clamped above 100ms")) => {},
			_ = refresh_notify.notified() => {},
		}

		let now = Utc::now();

		if (refresh_token.expires_at - TimeDelta::milliseconds(200)) < now {
			let (new_access_token, new_refresh_token) = nordigen_api
				.obtain_token_pair(&secret)
				.await
				.map_err(TokenRefresherError::Obtain)?;

			refresh_token = new_refresh_token;

			access_token = new_access_token;
			access_token_watch.update(access_token.value.clone());
		}

		if (access_token.expires_at - TimeDelta::milliseconds(200)) < now {
			let new_access_token = nordigen_api
				.refresh_access_token(&refresh_token.value)
				.await
				.map_err(TokenRefresherError::Refresh)?;

			access_token = new_access_token;
			access_token_watch.update(access_token.value.clone());
		}
	}
}

#[derive(Debug, Error)]
enum TokenRefresherError<N: NordigenApi> {
	#[error(
		"an error occurred while obtaining fresh access-refresh-token pair in token-refresher-task"
	)]
	Obtain(#[source] N::ObtainTokenPairError),
	#[error("an error occurred while fresh access token in token-refresher-task")]
	Refresh(#[source] N::RefreshAccessTokenError),
}
