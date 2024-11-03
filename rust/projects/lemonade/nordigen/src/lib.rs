use lemonade_model::{AccessToken, ExpiresAt, Expiring, RefreshToken, Secret};
use reqwest::Url;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Whatever};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct NordigenClient {
	base: Url,
	http_client: reqwest::Client,
}

impl NordigenClient {
	pub fn new(base: Url) -> Self {
		NordigenClient {
			base,
			http_client: reqwest::Client::new(),
		}
	}

	pub async fn new_token(
		&self,
		secret: &Secret,
	) -> Result<(Expiring<AccessToken>, Expiring<RefreshToken>), Whatever> {
		let url = self.base.join("token/new/").expect("to be valid URL");

		#[derive(Serialize)]
		struct RequestBody<'l> {
			secret_id: &'l str,
			secret_key: &'l str,
		}

		#[derive(Deserialize)]
		struct ResponseBody {
			access: Arc<str>,
			access_expires: i64,
			refresh: Arc<str>,
			refresh_expires: i64,
		}

		let body = RequestBody {
			secret_id: secret.id.0.as_ref(),
			secret_key: secret.key.0.as_ref(),
		};

		let start = chrono::Utc::now();

		let response = self
			.http_client
			.post(url)
			.json(&body)
			.send()
			.await
			.with_whatever_context(|_| {
				"failed to get new nordigen token, failed to send http request"
			})?;
		let response_body = response
			.json::<ResponseBody>()
			.await
			.with_whatever_context(|_| {
				"failed to get new nordigen token, failed to read response"
			})?;

		let access_token = AccessToken(response_body.access)
			.expires_at(start + chrono::TimeDelta::seconds(response_body.access_expires));
		let refresh_token = RefreshToken(response_body.refresh)
			.expires_at(start + chrono::TimeDelta::seconds(response_body.refresh_expires));

		Ok((access_token, refresh_token))
	}

	pub async fn refresh_token(
		&self,
		refresh_token: &RefreshToken,
	) -> Result<Expiring<AccessToken>, Whatever> {
		let url = self.base.join("token/refresh/").expect("to be valid URL");

		#[derive(Serialize)]
		struct RequestBody<'l> {
			refresh: &'l str,
		}

		#[derive(Deserialize)]
		struct ResponseBody {
			access: Arc<str>,
			access_expires: i64,
		}

		let body = RequestBody {
			refresh: refresh_token.0.as_ref(),
		};

		let start = chrono::Utc::now();

		let response = self
			.http_client
			.post(url)
			.json(&body)
			.send()
			.await
			.with_whatever_context(|_| {
				"failed to refresh nordigen access token, failed to send http request"
			})?;
		let response_body = response
			.json::<ResponseBody>()
			.await
			.with_whatever_context(|_| {
				"failed to refresh nordigen access token, failed to read response"
			})?;

		let access_token = AccessToken(response_body.access)
			.expires_at(start + chrono::TimeDelta::seconds(response_body.access_expires));

		Ok(access_token)
	}
}
