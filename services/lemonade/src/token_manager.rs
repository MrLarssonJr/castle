use std::error::Error;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;

pub trait TokenClient {
	type Error: 'static + Error + Send + Sync;
	async fn new(&self, secret: &Secret) -> Result<AccessAndRefreshToken, Self::Error>;
	async fn refresh(&self, refresh: &str) -> Result<Token, Self::Error>;
}

#[derive(Debug)]
pub struct AccessAndRefreshToken {
	pub access: Token,
	pub refresh: Token,
}

pub struct TokenManager<C> {
	secret: Secret,
	access: RwLock<Option<Token>>,
	refresh: RwLock<Option<Token>>,
	token_client: C,
}

impl<C: TokenClient> TokenManager<C> {
	pub fn new(secret_id: impl Into<Box<str>>, secret_key: impl Into<Box<str>>, token_client: C) -> TokenManager<C> {
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
		let token = { self.access.read()
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
		let token = { self.refresh.read()
			.await
			.as_ref()
			.map(|token| (token.value.clone(), token.expires_at))
		};

		let token = match token {
		    None => return self.refresh_refresh_token().await,
			Some((_, expires_at)) if Utc::now() < expires_at => return self.refresh_refresh_token().await,
			Some((token, _)) => token,
		};

		let access = dbg!(self.token_client.refresh(token.as_ref()).await?);

		let res = access.value.clone();

		(*self.access.write().await) = Some(access);

		Ok(res)
	}

	async fn refresh_refresh_token(&self) -> anyhow::Result<Arc<str>> {
		let AccessAndRefreshToken {
			access,
			refresh
		} = dbg!(self.token_client.new(&self.secret).await?);

		let res = access.value.clone();

		(*self.access.write().await) = Some(access);
		(*self.refresh.write().await) = Some(refresh);

		Ok(res)
	}
}

pub struct Secret {
	id: Box<str>,
	key: Box<str>,
}

impl Secret {
	pub fn id(&self) -> &str {
		self.id.as_ref()
	}

	pub fn key(&self) -> &str {
		self.key.as_ref()
	}
}

#[derive(Debug)]
pub struct Token {
	value: Arc<str>,
	expires_at: DateTime<Utc>,
}

impl Token {
	pub fn new(value: impl AsRef<str>, expires_at: DateTime<Utc>) -> Token {
		Token {
			value: Arc::from(value.as_ref()),
			expires_at,
		}
	}
}
