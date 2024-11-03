use crate::domain::lemonade::service::Service;
use crate::inbound::http::app_state::AppState;
use crate::outbound::{InMemoryRepository, Random};
pub use error::StartError;

pub async fn start() -> Result<(), StartError> {
	let random_provider = Random;
	let in_memory_repository = InMemoryRepository::new();
	let lemonade_service = Service::new(random_provider, in_memory_repository);

	let state = AppState::new(lemonade_service);

	let app = handlers::build_router().with_state(state);

	let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
		.await
		.map_err(StartError::Bind)?;

	axum::serve(listener, app)
		.await
		.map_err(StartError::Serve)?;

	Ok(())
}

mod app_state;
mod error;
mod extractors;
mod handlers;
