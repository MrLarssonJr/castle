use crate::extensions::error::ErrorExt;
use std::error::Error;
use std::process::exit;

pub trait ResultExt {
	type V;
	fn must(self) -> Self::V;
}

impl<V, E: Error> ResultExt for Result<V, E> {
	type V = V;
	fn must(self) -> Self::V {
		match self {
			Ok(v) => v,
			Err(err) => {
				eprintln!("{}", err.to_pretty_string());
				exit(1);
			}
		}
	}
}
