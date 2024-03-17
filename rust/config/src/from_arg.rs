use std::error::Error;
use std::ffi::OsString;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
use std::num::{
	NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
	NonZeroU32, NonZeroU64, NonZeroU8,
};
use std::path::PathBuf;

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
impl_from_arg_with_parse!(NonZeroU8, NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU128);
impl_from_arg_with_parse!(i8, i16, i32, i64, i128, isize);
impl_from_arg_with_parse!(NonZeroI8, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI128);
impl_from_arg_with_parse!(f32, f64);
impl_from_arg_with_parse!(bool, char);
impl_from_arg_with_parse!(String, OsString, PathBuf);
impl_from_arg_with_parse!(
	Ipv4Addr,
	Ipv6Addr,
	IpAddr,
	SocketAddrV4,
	SocketAddrV6,
	SocketAddr
);
