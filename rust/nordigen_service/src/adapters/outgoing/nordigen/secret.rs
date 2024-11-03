use constrained_str::{constrained_str, Validator};
use std::convert::Infallible;

pub struct NordigenApiSecretIdValidator;
impl Validator for NordigenApiSecretIdValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		Ok(v)
	}
}

pub struct NordigenApiSecretKeyValidator;
impl Validator for NordigenApiSecretKeyValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		Ok(v)
	}
}

constrained_str!(pub NordigenApiSecretId, NordigenApiSecretIdValidator);
constrained_str!(pub NordigenApiSecretKey, NordigenApiSecretKeyValidator);

pub struct Secret {
	pub id: NordigenApiSecretId,
	pub key: NordigenApiSecretKey,
}
