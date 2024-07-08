use std::sync::Arc;

use reqwest::{Response, StatusCode};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Client {
	client: reqwest::Client,
	base_url: Arc<str>,
}

impl Client {
	pub fn new(base_url: impl Into<Arc<str>>) -> Client {
		Client {
			client: reqwest::Client::new(),
			base_url: base_url.into(),
		}
	}

	fn url(&self, path: &str) -> String {
		format!("{}{}", self.base_url, path)
	}

	pub async fn token_new(
		&self,
		secret_id: &str,
		secret_key: &str,
	) -> Result<TokenNewResponse, TokenNewError> {
		#[derive(Serialize)]
		struct RequestBody<'l> {
			secret_id: &'l str,
			secret_key: &'l str,
		}

		let body = RequestBody {
			secret_id,
			secret_key,
		};

		let url = self.url("/api/v2/token/new/");

		let res = self.client.post(url).json(&body).send().await?;
		<Result<TokenNewResponse, TokenNewError> as ParseResponse>::parse_response(res).await
	}

	pub async fn token_refresh(
		&self,
		refresh: &str,
	) -> Result<TokenRefreshResponse, TokenRefreshError> {
		#[derive(Serialize)]
		struct RequestBody<'l> {
			refresh: &'l str,
		}

		let body = RequestBody { refresh };

		let url = self.url("/api/v2/token/refresh/");
		let res = self.client.post(url).json(&body).send().await?;
		<Result<TokenRefreshResponse, TokenRefreshError> as ParseResponse>::parse_response(res)
			.await
	}

	pub async fn institutions(
		&self,
		access_token: impl AsRef<str>,
	) -> Result<Vec<InstitutionsResponse>, InstitutionsError> {
		let url = self.url("/api/v2/institutions/");
		let res = self
			.client
			.get(url)
			.bearer_auth(access_token.as_ref())
			.send()
			.await?;
		<Result<Vec<InstitutionsResponse>, InstitutionsError> as ParseResponse>::parse_response(res)
			.await
	}
}

trait ParseResponse {
	async fn parse_response(res: Response) -> Self;
}

macro_rules! response_error_enum {
	($name:ident $success_type:ty, $($variant:ident StatusCode::$status:ident => $msg:literal),*) => {
		#[derive(Debug, Error)]
		pub enum $name {
		$(
			#[error($msg)]
			$variant {
				detail: Arc<str>
			},
		)*
			#[error("api responded with an unknown status code {0}")]
			UnknownStatus(StatusCode),
			#[error("an http error occurred")]
			Http(#[from] reqwest::Error),
		}

		impl ParseResponse for Result<$success_type, $name> {
			async fn parse_response(res: Response) -> Result<$success_type, $name> {
				match res.status() {
					StatusCode::OK => return Ok(res.json::<$success_type>().await?),
				$(
					StatusCode::$status => Err($name::$variant {
						detail: res.json::<NordigenErrorBody>().await?.detail
					})?,
				)*
					other =>  Err($name::UnknownStatus(other))?
				}
			}
		}
	};
}

#[derive(Deserialize)]
pub struct TokenNewResponse {
	pub access: Arc<str>,
	pub access_expires: i64,
	pub refresh: Arc<str>,
	pub refresh_expires: i64,
}

response_error_enum! {
	TokenNewError TokenNewResponse,
	Unauthenticated StatusCode::UNAUTHORIZED => "could not authenticate",
	IpNotWhitelisted StatusCode::FORBIDDEN => "ip not whitelisted",
	RateLimited StatusCode::TOO_MANY_REQUESTS => "rate limited"
}

#[derive(Deserialize)]
pub struct TokenRefreshResponse {
	pub access: Arc<str>,
	pub access_expires: i64,
}

response_error_enum! {
	TokenRefreshError TokenRefreshResponse,
	Unauthenticated StatusCode::UNAUTHORIZED => "could not authenticate",
	IpNotWhitelisted StatusCode::FORBIDDEN => "ip not whitelisted",
	RateLimited StatusCode::TOO_MANY_REQUESTS => "rate limited"
}

#[derive(Deserialize)]
pub struct InstitutionsResponse {
	pub id: Arc<str>,
	pub name: Arc<str>,
}

response_error_enum! {
	InstitutionsError Vec<InstitutionsResponse>,
	UnkownFields StatusCode::BAD_REQUEST => "unknown fields supplied",
	NotFound StatusCode::NOT_FOUND => "could not find",
	Unauthenticated StatusCode::UNAUTHORIZED => "could not authenticate",
	IpNotWhitelisted StatusCode::FORBIDDEN => "ip not whitelisted",
	RateLimited StatusCode::TOO_MANY_REQUESTS => "rate limited"
}

#[derive(Debug, Error)]
#[error("{kind} {detail}")]
pub struct NordigenError {
	detail: Arc<str>,
	kind: NordigenErrorKind,
}

#[derive(Debug, Error)]
pub enum NordigenErrorKind {
	#[error("could not authenticate")]
	Unauthenticated,
	#[error("ip not whitelisted")]
	IpNotWhitelisted,
	#[error("rate limited")]
	RateLimited,
}

#[derive(Deserialize)]
struct NordigenErrorBody {
	detail: Arc<str>,
}
