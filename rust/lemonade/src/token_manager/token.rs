use chrono::{DateTime, Utc};
use std::sync::Arc;

#[derive(Debug)]
pub struct Token {
	pub(super) value: Arc<str>,
	pub(super) expires_at: DateTime<Utc>,
}

impl Token {
	pub fn new(value: impl AsRef<str>, expires_at: DateTime<Utc>) -> Token {
		Token {
			value: Arc::from(value.as_ref()),
			expires_at,
		}
	}
}
