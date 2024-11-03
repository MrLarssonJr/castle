macro_rules! new_str {
    ($vis:vis $name:ident) => {
		#[derive(::std::fmt::Debug, ::std::cmp::PartialEq, ::std::cmp::Eq, ::std::hash::Hash)]
		#[repr(transparent)]
		$vis struct $name(str);

		impl ::std::str::FromStr for &$name {
			type Err = ::std::convert::Infallible;

			fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
				Ok(unsafe { ::std::mem::transmute(s) })
			}
		}

		impl ::std::fmt::Display for $name {
			fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
				::std::write!(f, "{}", &self.0)
			}
		}

		impl ::std::borrow::Borrow<str> for $name {
			fn borrow(&self) -> &str {
				&self.0
			}
		}

		impl ::std::convert::AsRef<str> for $name {
			fn as_ref(&self) -> &str {
				&self.0
			}
		}

		impl ::std::convert::From<&$name> for ::std::boxed::Box<$name> {
			fn from(value: &$name) -> Self {
				let boxed_inner = ::std::boxed::Box::<str>::from(&value.0);
				let raw_pointer = ::std::boxed::Box::into_raw(boxed_inner);
				let raw_pointer = raw_pointer as *mut $name;

				// SAFETY: constructing box from same pointer we got from destructuring box.
				// Still safe despite type case due to one being a transparent version of the
				// other.
				unsafe { ::std::boxed::Box::from_raw(raw_pointer) }
			}
		}

		impl ::std::convert::From<&$name> for ::std::sync::Arc<$name> {
			fn from(value: &$name) -> Self {
				let boxed = ::std::boxed::Box::from(value);
				let arced = ::std::sync::Arc::from(boxed);
				arced
			}
		}

		impl ::std::convert::From<&$name> for ::std::rc::Rc<$name> {
			fn from(value: &$name) -> Self {
				let boxed = ::std::boxed::Box::from(value);
				let rced = ::std::rc::Rc::from(boxed);
				rced
			}
		}

		impl $name {
			pub fn into_str_arc(self: ::std::sync::Arc<$name>) -> ::std::sync::Arc<str> {
				// SAFETY: str has same layout due to #[repr(transparent)]
				unsafe { ::std::sync::Arc::from_raw(::std::sync::Arc::into_raw(self) as *const str) }
			}

			pub fn from_str_arc(value: ::std::sync::Arc<str>) -> ::std::sync::Arc<$name> {
				// SAFETY: str has same layout due to #[repr(transparent)]
				unsafe { ::std::sync::Arc::from_raw(::std::sync::Arc::into_raw(value) as *const $name) }
			}
		}
	};
}

pub(crate) use new_str;
