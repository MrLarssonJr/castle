use crate::domain::lemonade::ports::lemonade_service::errors::{
	AuthenticateWithApiKeyError, AuthenticateWithApiPasswordError, CreateUserError, FindUserError,
};
use crate::domain::lemonade::ports::store;
use crate::domain::lemonade::ports::store::errors::StoreFindApiKeyError;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use std::error::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StartError {
	#[error("could not bind to socket")]
	Bind(#[source] std::io::Error),
	#[error("error occurred while serving")]
	Serve(#[source] std::io::Error),
}

pub struct AppError {
	response: Response,
}

impl IntoResponse for AppError {
	fn into_response(self) -> Response {
		self.response
	}
}

impl<E: Error + IntoResponse> From<E> for AppError {
	fn from(error: E) -> Self {
		AppError {
			response: error.into_response(),
		}
	}
}

impl IntoResponse for store::errors::StoreFindOneUserError {
	fn into_response(self) -> Response {
		match self {
			store::errors::StoreFindOneUserError::NotFound => StatusCode::NOT_FOUND.into_response(),
			store::errors::StoreFindOneUserError::MoreThanOneFound(_) | Self::Unknown(_) => {
				StatusCode::INTERNAL_SERVER_ERROR.into_response()
			}
		}
	}
}

impl IntoResponse for store::errors::StoreCreateUserError {
	fn into_response(self) -> Response {
		match self {
			store::errors::StoreCreateUserError::Unknown(_) => {
				StatusCode::INTERNAL_SERVER_ERROR.into_response()
			}
		}
	}
}

impl IntoResponse for StoreFindApiKeyError {
	fn into_response(self) -> Response {
		match self {
			StoreFindApiKeyError::NotFound => StatusCode::NOT_FOUND.into_response(),
			StoreFindApiKeyError::Unknown(_) | StoreFindApiKeyError::MoreThanOneFound(_) => {
				StatusCode::INTERNAL_SERVER_ERROR.into_response()
			}
		}
	}
}

impl IntoResponse for AuthenticateWithApiKeyError {
	fn into_response(self) -> Response {
		match self {
			AuthenticateWithApiKeyError::Store { source, .. } => source.into_response(),
		}
	}
}

impl IntoResponse for AuthenticateWithApiPasswordError {
	fn into_response(self) -> Response {
		match self {
			AuthenticateWithApiPasswordError::Store { source, .. } => source.into_response(),
			AuthenticateWithApiPasswordError::BadPassword => {
				StatusCode::UNAUTHORIZED.into_response()
			}
		}
	}
}

impl IntoResponse for CreateUserError {
	fn into_response(self) -> Response {
		match self {
			CreateUserError::Store { source, .. } => source.into_response(),
		}
	}
}

impl IntoResponse for FindUserError {
	fn into_response(self) -> Response {
		match self {
			FindUserError::Store { source, .. } => source.into_response(),
		}
	}
}
