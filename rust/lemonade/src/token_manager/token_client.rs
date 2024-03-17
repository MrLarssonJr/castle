use super::{AccessAndRefreshToken, Secret, Token};
use std::error::Error;

pub trait TokenClient {
	type AcquireError: 'static + Error + Send + Sync;
	type RefreshError: 'static + Error + Send + Sync;
	async fn acquire(&self, secret: &Secret) -> Result<AccessAndRefreshToken, Self::AcquireError>;
	async fn refresh(&self, refresh: &str) -> Result<Token, Self::RefreshError>;
}
