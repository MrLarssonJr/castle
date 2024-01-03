use crate::argument_parse_error::ArgumentParseError;

pub trait FromConfig: Sized {
	fn parse() -> Result<Self, ArgumentParseError>;
}
