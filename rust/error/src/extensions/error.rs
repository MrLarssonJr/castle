use std::error::Error;

pub trait ErrorExt: Error {
	fn to_pretty_string(&self) -> String {
		let mut res = self.to_string();

		let mut source = self.source();

		while let Some(e) = source {
			res.push_str(&format!(", caused by\n{e}"));
			source = e.source();
		}

		res
	}
}

impl<E: ?Sized + Error> ErrorExt for E {}

#[cfg(test)]
mod tests {
	use super::*;
	use thiserror::Error;

	#[test]
	fn no_source() {
		// Arrange
		let expected = "foo";

		let err: Box<dyn Error> = "foo".into();

		// Act
		let actual = err.to_pretty_string();

		// Assert
		assert_eq!(expected, actual);
	}

	#[test]
	fn one_source() {
		// Arrange
		let expected = "foo, caused by\nbar";

		#[derive(Debug, Error)]
		#[error("{msg}")]
		struct MyError {
			msg: &'static str,
			source: Box<dyn Error>,
		}

		let source: Box<dyn Error> = "bar".into();
		let err = MyError { msg: "foo", source };

		// Act
		let actual = err.to_pretty_string();

		// Assert
		assert_eq!(expected, actual);
	}
}
