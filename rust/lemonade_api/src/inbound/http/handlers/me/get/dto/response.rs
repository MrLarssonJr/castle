use crate::domain::lemonade::models::ActorId;
use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Response {
	id: Box<str>,
}

impl From<&ActorId> for Response {
	fn from(id: &ActorId) -> Self {
		let id = id.as_ref();
		let id = Box::from(id);

		Response { id }
	}
}
