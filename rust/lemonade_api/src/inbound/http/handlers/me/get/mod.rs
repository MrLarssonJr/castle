use crate::inbound::http::extractors::Authentication;
use axum::Json;

pub async fn handler(authentication: Authentication) -> Json<dto::Response> {
	let actor_id = authentication.actor_id();

	Json(dto::Response::from(actor_id))
}

pub mod dto;
