mod graphql;

use crate::nordigen_token_client::NordigenTokenClient;
use crate::token_manager::{TokenClient, TokenManager};
use axum::Router;
use mongodb::Client;
use tower_http::trace::{DefaultOnRequest, DefaultOnResponse, TraceLayer};
use tracing::Level;

pub struct Api;

impl Api {
	pub async fn run(
		mongo_client: Client,
		nordigen_client: http_api::nordigen::Client,
		token_manager: TokenManager<NordigenTokenClient>,
	) {
		let root_router = Router::new()
			.nest(
				"/graphql",
				graphql::build_router(mongo_client, nordigen_client, token_manager),
			)
			.layer(
				TraceLayer::new_for_http()
					.on_request(DefaultOnRequest::new().level(Level::INFO))
					.on_response(DefaultOnResponse::new().level(Level::INFO)),
			);
		let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
		axum::serve(listener, root_router).await.unwrap();
	}
}
