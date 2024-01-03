use std::error::Error;
use std::ffi::OsString;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArgumentParseError {
	#[error("argument {name} missing")]
	Missing { name: &'static str },
	#[error("argument {name} is not unicode")]
	NotUnicode {
		name: &'static str,
		actual: OsString,
	},
	#[error("argument {name} could not be parsed as {ty}")]
	NotParseable {
		name: &'static str,
		ty: &'static str,
		source: Box<dyn Error>,
	},
}
