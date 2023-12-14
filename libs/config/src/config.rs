use crate::{ParseConfigFromEnvError, RenameConfigError};
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Config {
	values: HashMap<Rc<str>, Rc<str>>,
}

impl Config {
	pub fn parse_from_env() -> Result<Config, ParseConfigFromEnvError> {
		use crate::ParseConfigFromEnvError as E;

		let config = std::env::vars_os()
			.map(|(key, value)| {
				let key = key.into_string().map_err(E::KeyNotUnicode)?;
				let value = value.into_string().map_err(E::ValueNotUnicode)?;
				Ok((key, value))
			})
			.collect::<Result<Config, _>>()?;

		Ok(config)
	}

	pub fn renamed<V: AsRef<str>>(
		self,
		mapping: HashMap<&str, V>,
	) -> Result<Config, RenameConfigError> {
		let mut new_values = HashMap::with_capacity(self.values.len());

		for (old_key, value) in self.values {
			let new_key = if let Some(new_key) = mapping.get(old_key.as_ref()) {
				Rc::from(new_key.as_ref())
			} else {
				old_key
			};

			match new_values.entry(new_key) {
				Entry::Vacant(entry) => entry.insert(value),
				Entry::Occupied(entry) => {
					return Err(RenameConfigError::KeyClash(entry.key().clone()))
				}
			};
		}

		Ok(Config { values: new_values })
	}

	pub fn extend(mut self, other: Config) -> Self {
		self.values.extend(other.values);
		self
	}

	pub fn insert(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Option<Rc<str>> {
		let key = Rc::from(key.as_ref());
		let value = Rc::from(value.as_ref());
		self.values.insert(key, value)
	}

	pub fn get(&self, key: impl AsRef<str>) -> Option<Rc<str>> {
		let key = key.as_ref();
		self.values.get(key).cloned()
	}

	pub fn remove(&mut self, key: impl AsRef<str>) -> Option<Rc<str>> {
		let key = key.as_ref();
		self.values.remove(key)
	}
}

impl<K: AsRef<str>, V: AsRef<str>> FromIterator<(K, V)> for Config {
	fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
		let values = iter
			.into_iter()
			.map(|(key, value)| {
				let key = Rc::from(key.as_ref());
				let value = Rc::from(value.as_ref());
				(key, value)
			})
			.collect();

		Config { values }
	}
}
