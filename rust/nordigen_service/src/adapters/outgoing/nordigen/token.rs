use chrono::{DateTime, Utc};
use constrained_str::{constrained_str, Validator};
use std::convert::Infallible;

pub struct NordigenApiAccessTokenValidator;
impl Validator for NordigenApiAccessTokenValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		Ok(v)
	}
}

pub struct NordigenApiRefreshTokenValidator;
impl Validator for NordigenApiRefreshTokenValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		Ok(v)
	}
}

constrained_str!(pub NordigenApiAccessToken, NordigenApiAccessTokenValidator);
constrained_str!(pub NordigenApiRefreshToken, NordigenApiRefreshTokenValidator);

#[derive(Debug, Clone)]
pub struct Expiring<T> {
	pub value: T,
	pub expires_at: DateTime<Utc>,
}

impl<T> Expiring<T> {
	pub fn map<O>(self, f: impl FnOnce(T) -> O) -> Expiring<O> {
		Expiring {
			value: f(self.value),
			expires_at: self.expires_at,
		}
	}
}
