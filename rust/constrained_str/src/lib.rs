use std::error::Error;

pub trait Validator {
	type Error: Error;

	fn validate(v: &str) -> Result<&str, Self::Error>;
}

#[macro_export]
macro_rules! constrained_str {
	($vis:vis $name:ident, $validator:ident) => {
		#[derive(
			::std::fmt::Debug,
			::std::hash::Hash,
			::std::cmp::PartialEq,
			::std::cmp::Eq,
			::std::clone::Clone,
		)]
		$vis struct $name(::std::sync::Arc<str>);

		impl $name {
			pub fn from_str(
				s: impl AsRef<str>,
			) -> Result<$name, <$validator as $crate::Validator>::Error> {
				let s = s.as_ref();
				let checked_s = $validator::validate(s)?;
				let checked_shared_s = ::std::sync::Arc::from(checked_s);
				Ok($name(checked_shared_s))
			}
		}

		impl ::std::str::FromStr for $name {
			type Err = <$validator as $crate::Validator>::Error;

			fn from_str(s: &str) -> Result<Self, Self::Err> {
				$name::from_str(s)
			}
		}

		impl AsRef<str> for $name {
			fn as_ref(&self) -> &str {
				self.0.as_ref()
			}
		}
	};
}

#[cfg(test)]
mod tests {
	use super::*;
	use error::InfallibleResultExt;
	use std::convert::Infallible;
	use std::hash::{DefaultHasher, Hash, Hasher};

	struct TestIdValidator;

	impl Validator for TestIdValidator {
		type Error = Infallible;

		fn validate(v: &str) -> Result<&str, Self::Error> {
			Ok(v)
		}
	}

	constrained_str!(TestId, TestIdValidator);

	#[test]
	fn create() {
		// Arrange
		let id = "foo";
		let typed_id = TestId::from_str(id).unwrap_infallible();

		let mut id_hasher = DefaultHasher::new();
		let mut typed_id_hasher = DefaultHasher::new();

		// Act
		id.hash(&mut id_hasher);
		let id_hash = id_hasher.finish();

		typed_id.hash(&mut typed_id_hasher);
		let typed_id_hash = typed_id_hasher.finish();

		// Assert
		assert_eq!(id_hash, typed_id_hash);
	}
}
