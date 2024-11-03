use crate::domain::lemonade::models::{ActorId, ApiKey};
use crate::domain::lemonade::ports::lemonade_service::errors::AuthenticateWithApiKeyError;
use crate::domain::lemonade::ports::lemonade_service::LemonadeService;
use crate::domain::lemonade::ports::store::errors::StoreFindApiKeyError;
use crate::inbound::http::app_state::AppState;
use crate::inbound::http::error::AppError;
use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::header::ToStrError;
use axum::http::request::Parts;
use axum::http::{HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use base64::Engine;
use error::{ErrorExt, InfallibleResultExt};
use itertools::Itertools;
use std::future::Future;
use std::num::ParseIntError;
use std::process::Output;
use std::str::Utf8Error;
use std::sync::Arc;
use thiserror::Error;

struct Credentials {
	username: Box<str>,
	password: Box<str>,
}

trait PartsExt: Sync {
	fn extract_header(&self, header: &'static str) -> Result<&str, ExtractHeaderError>;

	fn extract_authorization_header(
		&self,
		expected_scheme: &'static str,
	) -> Result<&str, ExtractAuthorizationHeaderError> {
		let header_value = self.extract_header("Authorization")?;

		let Some((actual_scheme, value)) = header_value.split_once(' ') else {
			return Err(ExtractAuthorizationHeaderError::NoScheme {
				expected: expected_scheme,
				header_value: header_value.into(),
			});
		};

		if expected_scheme != actual_scheme {
			return Err(ExtractAuthorizationHeaderError::UnexpectedScheme {
				expected: expected_scheme,
				actual: actual_scheme.into(),
			});
		}

		Ok(value)
	}

	fn extract_bearer_token(&self) -> Result<&str, ExtractBearerTokenError> {
		let token = self.extract_authorization_header("Bearer")?;
		Ok(token)
	}

	fn extract_basic_credentials(&self) -> Result<Credentials, ExtractBasicCredentialsError> {
		let encoded_credentials = self.extract_authorization_header("Basic")?;
		let decoded_credentials =
			base64::engine::general_purpose::STANDARD.decode(encoded_credentials)?;
		let decoded_credentials = std::str::from_utf8(&decoded_credentials)?;

		let Some((username, password)) = decoded_credentials.split_once(":") else {
			return Err(ExtractBasicCredentialsError::Format(
				decoded_credentials.into(),
			));
		};

		let username = username.into();
		let password = password.into();

		Ok(Credentials { username, password })
	}

	fn authenticate_by_api_key<LS: LemonadeService + Sync>(
		&self,
		lemonade_service: &LS,
	) -> impl Future<Output = Result<Arc<ActorId>, AuthenticateByApiKeyError>> + Send {
		async {
			let api_key = self.extract_bearer_token()?;
			let api_key = api_key.parse::<&ApiKey>().unwrap_infallible();
			let actor_id = lemonade_service.authenticate_with_api_key(api_key).await?;

			Ok(actor_id)
		}
	}

	fn authenticate_by_username_password<LS: LemonadeService + Sync>(
		&self,
		lemonade_service: &LS,
	) -> impl Future<Output = Result<Arc<ActorId>, AuthenticateByCredentialsError>> {
		async { todo!() }
	}
}

impl PartsExt for Parts {
	fn extract_header(&self, header: &'static str) -> Result<&str, ExtractHeaderError> {
		let header_value = self
			.headers
			.get_all("Authorization")
			.into_iter()
			.at_most_one()
			.map_err(|values| ExtractHeaderError::TooMany(header, values.cloned().collect()))?
			.ok_or(ExtractHeaderError::None(header))?;

		let header_value = header_value
			.to_str()
			.map_err(|err| ExtractHeaderError::NotUtf8(header, err))?;

		Ok(header_value)
	}
}

#[derive(Debug, Error)]
enum ExtractHeaderError {
	#[error("expected exactly one Authorization header, got more: {0:?}")]
	TooMany(&'static str, Vec<HeaderValue>),
	#[error("expected exactly one Authorization header, got none")]
	None(&'static str),
	#[error("the Authorization header did not contain valid UTF-8")]
	NotUtf8(&'static str, #[source] ToStrError),
}

#[derive(Debug, Error)]
enum ExtractAuthorizationHeaderError {
	#[error("failed to extract authorization header value due to bad invalid header")]
	InvalidHeader(#[from] ExtractHeaderError),
	#[error("failed to extract authorization header value due to missing scheme, expected {expected}, actual header value {header_value}")]
	NoScheme {
		expected: &'static str,
		header_value: Box<str>,
	},
	#[error("failed to extract authorization header value due unexpected scheme, expected {expected}, actual {actual}")]
	UnexpectedScheme {
		expected: &'static str,
		actual: Box<str>,
	},
}

#[derive(Debug, Error)]
#[error("failed to extract authorization header token")]
struct ExtractBearerTokenError(#[from] ExtractAuthorizationHeaderError);

#[derive(Debug, Error)]
enum ExtractBasicCredentialsError {
	#[error("failed to extract basic credentials due to bad Authorization header")]
	AuthorizationHeader(#[from] ExtractAuthorizationHeaderError),
	#[error("failed to extract basic credentials due to Authorization header value containing invalid base64 data")]
	Base64(#[from] base64::DecodeError),
	#[error("failed to extract basic credentials due to Authorization header value after base64 decoding containing invalid UTF-8 data")]
	NotUtf8(#[from] Utf8Error),
	#[error("failed to extract basic credentials due  Authorization header value after decoding containing invalid data, expected a ':' to split username and password, was {0}")]
	Format(Box<str>),
}

#[derive(Debug, Error)]
enum AuthenticateByApiKeyError {
	#[error("failed to authenticate by api key due to bad header")]
	Header(#[from] ExtractBearerTokenError),
	#[error("failed to authenticate by api key due to invalid key")]
	Authentication(#[from] AuthenticateWithApiKeyError),
}

fn foo(v: AuthenticateByApiKeyError) -> impl Send {
	v
}

#[derive(Debug, Error)]
enum AuthenticateByCredentialsError {}

pub struct Authentication {
	actor_id: Arc<ActorId>,
}

impl Authentication {
	pub fn actor_id(&self) -> &ActorId {
		&self.actor_id
	}
}

#[async_trait]
impl<LS: LemonadeService + Sync + Send> FromRequestParts<AppState<LS>> for Authentication {
	type Rejection = AuthenticationRejection;

	async fn from_request_parts(
		parts: &mut Parts,
		state: &AppState<LS>,
	) -> Result<Self, Self::Rejection> {
		let lemonade_service = state.lemonade_service();
		let auth_by_api_key_error = match parts.authenticate_by_api_key(lemonade_service).await {
			Ok(actor_id) => return Ok(Authentication { actor_id }),
			Err(err) => err,
		};

		let auth_by_api_key_error = match parts
			.authenticate_by_username_password(lemonade_service)
			.await
		{
			Ok(actor_id) => return Ok(Authentication { actor_id }),
			Err(err) => err,
		};

		tracing::warn!(
			"failed to authenticate request:\n{}\n{}",
			auth_by_api_key_error,
			auth_by_api_key_error
		);

		Err(AuthenticationRejection)
	}
}

pub struct AuthenticationRejection;

impl IntoResponse for AuthenticationRejection {
	fn into_response(self) -> Response {
		StatusCode::UNAUTHORIZED.into_response()
	}
}
