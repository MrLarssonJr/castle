use crate::adapters::outgoing::nordigen::NordigenApiTransactionId;
use chrono::NaiveDate;
use iban::Iban;
use iso_currency::Currency;
use rust_decimal::Decimal;

pub struct NordigenApiTransaction {
	pub id: NordigenApiTransactionId,
	pub reciprocal_account: Iban,
	pub amount: Decimal,
	pub currency: Currency,
	pub booking_date: NaiveDate,
}
