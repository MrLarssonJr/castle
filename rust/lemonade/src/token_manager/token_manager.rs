use crate::token_manager::{AccessAndRefreshToken, Secret, Token, TokenClient};
use chrono::Utc;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct TokenManager<C> {
	secret: Secret,
	access: RwLock<Option<Token>>,
	refresh: RwLock<Option<Token>>,
	token_client: C,
}

impl<C: TokenClient> TokenManager<C> {
	pub fn new(
		secret_id: impl Into<Box<str>>,
		secret_key: impl Into<Box<str>>,
		token_client: C,
	) -> TokenManager<C> {
		TokenManager {
			secret: Secret {
				id: secret_id.into(),
				key: secret_key.into(),
			},
			access: RwLock::new(None),
			refresh: RwLock::new(None),
			token_client,
		}
	}

	#[allow(unused)]
	pub async fn access_token(&self) -> anyhow::Result<Arc<str>> {
		let token = {
			self.access
				.read()
				.await
				.as_ref()
				.map(|token| (token.value.clone(), token.expires_at))
		};

		match token {
			None => self.refresh_access_token().await,
			Some((_, expires_at)) if Utc::now() < expires_at => self.refresh_access_token().await,
			Some((token, _)) => Ok(token),
		}
	}

	async fn refresh_access_token(&self) -> anyhow::Result<Arc<str>> {
		let token = {
			self.refresh
				.read()
				.await
				.as_ref()
				.map(|token| (token.value.clone(), token.expires_at))
		};

		let token = match token {
			None => return self.refresh_refresh_token().await,
			Some((_, expires_at)) if Utc::now() < expires_at => {
				return self.refresh_refresh_token().await
			}
			Some((token, _)) => token,
		};

		let access = dbg!(self.token_client.refresh(token.as_ref()).await?);

		let res = access.value.clone();

		(*self.access.write().await) = Some(access);

		Ok(res)
	}

	async fn refresh_refresh_token(&self) -> anyhow::Result<Arc<str>> {
		let AccessAndRefreshToken { access, refresh } =
			dbg!(self.token_client.acquire(&self.secret).await?);

		let res = access.value.clone();

		(*self.access.write().await) = Some(access);
		(*self.refresh.write().await) = Some(refresh);

		Ok(res)
	}
}
