use http_api::nordigen::types::error::ConversionError;
use http_api::nordigen::types::ErrorResponse;
use http_api::nordigen::Error;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AcquireError {
	#[error("secret part {part} not valid")]
	InvalidSecret {
		part: &'static str,
		source: ConversionError,
	},
	#[error("received error response from API")]
	ErrorResponse(#[from] Error<ErrorResponse>),

	#[error("invalid response: {detail}")]
	InvalidResponse { detail: &'static str },
}
