mod acquire_error;
mod refresh_error;

use crate::token_manager::{AccessAndRefreshToken, Secret, Token, TokenClient};
use acquire_error::AcquireError;
use chrono::{Duration, Utc};
use http_api::nordigen::types::{
	JwtObtainPairRequestSecretId, JwtObtainPairRequestSecretKey, JwtRefreshRequest,
	JwtRefreshRequestRefresh,
};
use http_api::nordigen::Client;
use refresh_error::RefreshError;
use std::str::FromStr;

pub struct NordigenTokenClient<'client> {
	client: &'client Client,
}

impl NordigenTokenClient<'_> {
	pub fn new(client: &Client) -> NordigenTokenClient {
		NordigenTokenClient { client }
	}
}

impl<'client> TokenClient for NordigenTokenClient<'client> {
	type AcquireError = AcquireError;
	type RefreshError = RefreshError;

	async fn acquire(&self, secret: &Secret) -> Result<AccessAndRefreshToken, Self::AcquireError> {
		let secret_id = JwtObtainPairRequestSecretId::from_str(secret.id())
			.map_err(|source| AcquireError::InvalidSecret { part: "id", source })?;

		let secret_key =
			JwtObtainPairRequestSecretKey::from_str(secret.key()).map_err(|source| {
				AcquireError::InvalidSecret {
					part: "key",
					source,
				}
			})?;

		let body = http_api::nordigen::types::JwtObtainPairRequest {
			secret_id,
			secret_key,
		};

		let start = Utc::now();
		let res = self
			.client
			.obtain_new_access_refresh_token_pair(&body)
			.await?
			.into_inner();

		let access = res.access.ok_or(AcquireError::InvalidResponse {
			detail: "missing access token",
		})?;
		let access_expires_at = start + Duration::seconds(res.access_expires);

		let refresh = res.refresh.ok_or(AcquireError::InvalidResponse {
			detail: "missing refresh token",
		})?;
		let refresh_expires_at = start + Duration::seconds(res.refresh_expires);

		let access = Token::new(access, access_expires_at);
		let refresh = Token::new(refresh, refresh_expires_at);

		Ok(AccessAndRefreshToken { access, refresh })
	}

	async fn refresh(&self, refresh: &str) -> Result<Token, Self::RefreshError> {
		let refresh = JwtRefreshRequestRefresh::from_str(refresh)?;

		let body = JwtRefreshRequest { refresh };

		let start = Utc::now();
		let res = self
			.client
			.get_a_new_access_token(&body)
			.await?
			.into_inner();

		let access = res.access.ok_or(RefreshError::InvalidResponse {
			detail: "missing access token",
		})?;
		let access_expires_at = start + Duration::seconds(res.access_expires);

		let access = Token::new(access, access_expires_at);

		Ok(access)
	}
}
