use super::Token;

#[derive(Debug)]
pub struct AccessAndRefreshToken {
	pub access: Token,
	pub refresh: Token,
}
