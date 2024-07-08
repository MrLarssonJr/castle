use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
	#[serde(rename = "_id")]
	pub id: Uuid,
	pub name: Arc<str>,
}
