use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use lemonade_model::{BasicCredential, SessionToken};
use serde::{Deserialize, Serialize};
use snafu::{Report, ResultExt, Whatever};
use sqlx::error::ErrorKind;
use sqlx::PgPool;
use std::str::FromStr;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AppState {
	pub db: PgPool,
}

#[derive(Debug, Serialize)]
struct PageDto<T> {
	items: Vec<T>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct PageQueryOptions {
	limit: i64,
	offset: i64,
}

impl Default for PageQueryOptions {
	fn default() -> Self {
		PageQueryOptions {
			limit: 100,
			offset: 0,
		}
	}
}

enum ApiError {
	NotFound,
	InternalServerError,
	BadRequest,
	Unauthenticated,
	Unauthorized,
}

impl IntoResponse for ApiError {
	fn into_response(self) -> Response {
		match self {
			ApiError::NotFound => StatusCode::NOT_FOUND.into_response(),
			ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
			ApiError::BadRequest => StatusCode::BAD_REQUEST.into_response(),
			ApiError::Unauthenticated => StatusCode::UNAUTHORIZED.into_response(),
			ApiError::Unauthorized => StatusCode::FORBIDDEN.into_response(),
		}
	}
}

impl From<sqlx::Error> for ApiError {
	fn from(value: sqlx::Error) -> Self {
		let res = match &value {
			sqlx::Error::RowNotFound => ApiError::NotFound,
			sqlx::Error::Database(db_err) => match db_err.kind() {
				ErrorKind::Other => ApiError::InternalServerError,
				_ => ApiError::BadRequest,
			},
			_ => ApiError::InternalServerError,
		};

		let report = snafu::Report::from_error(value);
		tracing::warn!("{report}");

		res
	}
}

impl From<Whatever> for ApiError {
	fn from(whatever: Whatever) -> Self {
		let report = snafu::Report::from_error(whatever);
		tracing::warn!("{report}");
		Self::InternalServerError
	}
}

struct Authentication {
	user_id: Uuid,
}

impl Authentication {
	async fn from_basic(db: &PgPool, value: &str) -> Result<Authentication, ApiError> {
		let creds =
			BasicCredential::from_str(value).with_whatever_context::<_, _, Whatever>(|_| {
				format!("could not build authentication from basic creds, got bad creds {value}")
			})?;

		let Some(user_id) = lemonade_db::users::authenticate(db, &creds)
			.await
			.with_whatever_context::<_, _, Whatever>(|_| {
				"could not build authentication from basic creds, failed to authenticate"
			})?
		else {
			return Err(ApiError::Unauthenticated);
		};

		Ok(Authentication { user_id })
	}

	async fn from_session(
		db: &PgPool,
		session_token: &str,
	) -> Result<Option<Authentication>, Whatever> {
		let session_token = SessionToken::from_str(session_token)
			.with_whatever_context(|_| "failed to parse session_token")?;

		let Some(user_id) = lemonade_db::sessions::authenticate(db, session_token)
			.await
			.with_whatever_context(|_| "failed to authenticate session")?
		else {
			return Ok(None);
		};

		Ok(Some(Authentication { user_id }))
	}
}

#[async_trait]
impl FromRequestParts<AppState> for Authentication {
	type Rejection = ApiError;

	async fn from_request_parts(
		parts: &mut Parts,
		state: &AppState,
	) -> Result<Self, Self::Rejection> {
		let auth_header = parts
			.headers
			.get(axum::http::header::AUTHORIZATION)
			.ok_or_else(|| {
				tracing::warn!("missing authorization header");
				ApiError::Unauthenticated
			})?;

		let auth_header = auth_header.to_str().map_err(|err| {
			tracing::warn!("bad authorization header: {}", Report::from_error(err));
			ApiError::BadRequest
		})?;

		if let Some(value) = auth_header.strip_prefix("Basic ") {
			return Ok(Authentication::from_basic(&state.db, value).await?);
		}

		if let Some(value) = auth_header.strip_prefix("Session ") {
			let Some(authentication) = Authentication::from_session(&state.db, value).await? else {
				return Err(ApiError::Unauthenticated);
			};
			return Ok(authentication);
		}

		tracing::warn!("unexpected authorization header - got unexpected type");
		Err(ApiError::BadRequest)
	}
}

pub mod sessions;
pub mod users;
