use std::error::Error;

pub trait FromConfig: Sized {
	type Error: Error;

	fn parse() -> Result<Self, Self::Error>;
}
