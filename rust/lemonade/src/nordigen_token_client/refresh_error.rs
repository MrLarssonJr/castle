use http_api::nordigen::types::error::ConversionError;
use http_api::nordigen::types::ErrorResponse;
use http_api::nordigen::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum RefreshError {
	#[error("refresh token not valid")]
	InvalidRefresh(#[from] ConversionError),

	#[error("received error response from API")]
	ErrorResponse(#[from] Error<ErrorResponse>),

	#[error("invalid response: {detail}")]
	InvalidResponse { detail: &'static str },
}
