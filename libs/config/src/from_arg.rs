use std::error::Error;

pub trait FromArg: Sized {
	type Error: Error;
	fn parse_arg(argument: &str) -> Result<Self, Self::Error>;
}

macro_rules! impl_from_arg_with_parse {
	($( $t:ty ),*) => {
		$(
			impl FromArg for $t {
				type Error = <$t as ::std::str::FromStr>::Err;

				fn parse_arg(argument: &str) -> Result<Self, Self::Error> {
					argument.parse()
				}
			}
		)*
	};
}

impl_from_arg_with_parse!(u8, u16, u32, u64, u128, usize);
