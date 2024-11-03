use std::convert::Infallible;

pub trait InfallibleResultExt {
	type V;
	fn unwrap_infallible(self) -> Self::V;
}

impl<V> InfallibleResultExt for Result<V, Infallible> {
	type V = V;

	fn unwrap_infallible(self) -> Self::V {
		match self {
			Ok(v) => v,
			Err(e) => match e {},
		}
	}
}

#[cfg(test)]
mod tests {
	use crate::InfallibleResultExt;
	use std::convert::Infallible;

	#[test]
	fn unwrap_infallible() {
		// Arrange
		let res = Ok::<_, Infallible>(5);

		// Act
		let val = res.unwrap_infallible();

		// Assert
		assert_eq!(val, 5);
	}
}
