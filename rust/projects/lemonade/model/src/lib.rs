pub use basic_credential::*;
pub use expiring::*;
pub use role::*;
pub use secret::*;
pub use session_token::*;
pub use token::*;
pub use user::*;

mod session_token {
	use base64::Engine;
	use orion::pwhash::{Password, PasswordHash};
	use serde::{Deserialize, Serialize};
	use snafu::{OptionExt, ResultExt, Whatever};
	use std::str::FromStr;
	use uuid::Uuid;

	#[derive(Debug, Serialize, Deserialize)]
	#[serde(try_from = "String")]
	#[serde(into = "String")]
	pub struct SessionToken {
		session_id: Uuid,
		password: Password,
	}

	impl Clone for SessionToken {
		fn clone(&self) -> Self {
			let password = Password::from_slice(self.password.unprotected_as_bytes())
				.expect("constructing password from bytes from a password should always be valid");

			SessionToken {
				session_id: self.session_id,
				password,
			}
		}
	}

	impl SessionToken {
		pub fn new(session_id: Uuid) -> Result<SessionToken, Whatever> {
			let password =
				Password::generate(16).with_whatever_context(|_| "failed to generate password")?;

			let session_token = SessionToken {
				session_id,
				password,
			};

			Ok(session_token)
		}

		pub fn hash(&self) -> Result<PasswordHash, Whatever> {
			let hash = orion::pwhash::hash_password(&self.password, 8, 16)
				.with_whatever_context(|_| "failed to hash password")?;
			Ok(hash)
		}

		pub fn session_id(&self) -> Uuid {
			self.session_id
		}

		pub fn password(&self) -> &Password {
			&self.password
		}
	}

	impl From<SessionToken> for String {
		fn from(session_token: SessionToken) -> Self {
			let mut bytes = Vec::new();
			bytes.extend(session_token.session_id.as_bytes());
			bytes.extend(session_token.password.unprotected_as_bytes());
			base64::engine::general_purpose::STANDARD.encode(bytes)
		}
	}

	impl FromStr for SessionToken {
		type Err = Whatever;

		fn from_str(s: &str) -> Result<Self, Self::Err> {
			let bytes = base64::engine::general_purpose::STANDARD
				.decode(s)
				.with_whatever_context(|_| {
					format!("failed to parse session token, string not base64 ({s})")
				})?;
			let (id, password) = bytes
				.split_first_chunk::<16>()
				.with_whatever_context(|| "failed to parse session token, too few bytes")?;

			let id = Uuid::from_bytes_ref(id);
			let password = Password::from_slice(password)
				.with_whatever_context(|_| "failed to parse session token, invalid password")?;

			let session_token = SessionToken {
				session_id: *id,
				password,
			};

			Ok(session_token)
		}
	}

	impl TryFrom<String> for SessionToken {
		type Error = <SessionToken as FromStr>::Err;

		fn try_from(s: String) -> Result<Self, Self::Error> {
			s.parse()
		}
	}
}

mod basic_credential {
	use base64::Engine;
	use orion::pwhash::Password;
	use snafu::{OptionExt, ResultExt, Whatever};
	use std::str::FromStr;

	pub struct BasicCredential {
		username: String,
		password: Password,
	}

	impl BasicCredential {
		pub fn username(&self) -> &str {
			&self.username
		}

		pub fn password(&self) -> &Password {
			&self.password
		}
	}

	impl FromStr for BasicCredential {
		type Err = Whatever;

		fn from_str(s: &str) -> Result<Self, Self::Err> {
			let bytes = base64::engine::general_purpose::STANDARD
				.decode(s)
				.with_whatever_context(|_| {
					format!("failed to parse basic credentials, string not base64 ({s})")
				})?;

			let decoded = std::str::from_utf8(&bytes)
				.with_whatever_context(|_| "failed to decode basic credential, not utf-8")?;

			let (username, password) = decoded
				.split_once(':')
				.with_whatever_context(|| "failed to decode basic credential, no ':'")?;

			let password = Password::from_slice(password.as_bytes())
				.with_whatever_context(|_| "failed to decode basic credential, invalid password")?;

			Ok(BasicCredential {
				username: username.into(),
				password,
			})
		}
	}
}

mod user {
	use uuid::Uuid;

	pub struct User {
		pub id: Uuid,
		pub username: String,
	}
}

mod role {
	pub enum Role {
		Admin,
	}
}

mod expiring {
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct Expiring<T> {
		expires_at: chrono::DateTime<chrono::Utc>,
		inner: T,
	}

	impl<T> Expiring<T> {
		pub fn as_ref(&self) -> Option<&T> {
			let time_until_expiry = self.expires_at - chrono::Utc::now();

			if time_until_expiry < chrono::TimeDelta::zero() {
				return None;
			}

			Some(&self.inner)
		}

		pub fn to_owned(self) -> Option<T> {
			let time_until_expiry = self.expires_at - chrono::Utc::now();

			if time_until_expiry < chrono::TimeDelta::zero() {
				return None;
			}

			Some(self.inner)
		}

		pub fn into_parts(self) -> (T, chrono::DateTime<chrono::Utc>) {
			(self.inner, self.expires_at)
		}
	}

	pub trait ExpiresAt: Sized {
		fn expires_at(self, expires_at: chrono::DateTime<chrono::Utc>) -> Expiring<Self> {
			Expiring {
				inner: self,
				expires_at,
			}
		}
	}

	impl<T: Sized> ExpiresAt for T {}
}

mod token {
	use std::sync::Arc;

	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct RefreshToken(pub Arc<str>);
	#[derive(Debug, Clone, PartialEq, Eq)]
	pub struct AccessToken(pub Arc<str>);
}

mod secret {
	use std::sync::Arc;

	#[derive(Debug, Clone)]
	pub struct SecretId(pub Arc<str>);
	#[derive(Debug, Clone)]
	pub struct SecretKey(pub Arc<str>);

	#[derive(Debug, Clone)]
	pub struct Secret {
		pub id: SecretId,
		pub key: SecretKey,
	}
}
