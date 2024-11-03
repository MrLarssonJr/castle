use constrained_str::{constrained_str, Validator};
use std::convert::Infallible;

pub struct NordigenApiAccountIdValidator;

impl Validator for NordigenApiAccountIdValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		// every str is valid account id
		Ok(v)
	}
}

pub struct NordigenApiInstitutionIdValidator;

impl Validator for NordigenApiInstitutionIdValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		// every str is valid institution id
		Ok(v)
	}
}

pub struct NordigenApiRequisitionIdValidator;

impl Validator for NordigenApiRequisitionIdValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		// every str is valid requisition id
		Ok(v)
	}
}

pub struct NordigenApiTransactionIdValidator;

impl Validator for NordigenApiTransactionIdValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		// every str is valid requisition id
		Ok(v)
	}
}

pub struct NordigenApiReferenceValidator;

impl Validator for NordigenApiReferenceValidator {
	type Error = Infallible;

	fn validate(v: &str) -> Result<&str, Self::Error> {
		// every str is valid requisition id
		Ok(v)
	}
}

constrained_str!(pub NordigenApiAccountId, NordigenApiAccountIdValidator);
constrained_str!(pub NordigenApiInstitutionId, NordigenApiInstitutionIdValidator);
constrained_str!(pub NordigenApiRequisitionId, NordigenApiRequisitionIdValidator);
constrained_str!(pub NordigenApiTransactionId, NordigenApiTransactionIdValidator);
constrained_str!(pub NordigenApiReference, NordigenApiReferenceValidator);
