pub struct Secret {
	pub(super) id: Box<str>,
	pub(super) key: Box<str>,
}

impl Secret {
	pub fn id(&self) -> &str {
		self.id.as_ref()
	}

	pub fn key(&self) -> &str {
		self.key.as_ref()
	}
}
