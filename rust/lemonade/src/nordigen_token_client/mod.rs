use crate::token_manager::{AccessAndRefreshToken, Secret, Token, TokenClient};
use chrono::{Duration, Utc};
use http_api::nordigen::Client;

pub struct NordigenTokenClient {
	client: Client,
}

impl NordigenTokenClient {
	pub fn new(client: Client) -> NordigenTokenClient {
		NordigenTokenClient { client }
	}
}

impl TokenClient for NordigenTokenClient {
	type AcquireError = http_api::nordigen::TokenNewError;
	type RefreshError = http_api::nordigen::TokenRefreshError;

	async fn acquire(&self, secret: &Secret) -> Result<AccessAndRefreshToken, Self::AcquireError> {
		let start = Utc::now();
		let res = self.client.token_new(secret.id(), secret.key()).await?;

		let access = res.access;
		let access_expires_at = start + Duration::seconds(res.access_expires);

		let refresh = res.refresh;
		let refresh_expires_at = start + Duration::seconds(res.refresh_expires);

		let access = Token::new(access, access_expires_at);
		let refresh = Token::new(refresh, refresh_expires_at);

		Ok(AccessAndRefreshToken { access, refresh })
	}

	async fn refresh(&self, refresh: &str) -> Result<Token, Self::RefreshError> {
		let start = Utc::now();
		let res = self.client.token_refresh(refresh).await?;

		let access = res.access;
		let access_expires_at = start + Duration::seconds(res.access_expires);

		let access = Token::new(access, access_expires_at);

		Ok(access)
	}
}
