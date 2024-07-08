mod connection;
mod institution;
mod user;

use crate::api::graphql::institution::InstitutionQuery;
use crate::api::graphql::user::{UserMutation, UserQuery};
use crate::nordigen_token_client::NordigenTokenClient;
use crate::token_manager::{TokenClient, TokenManager};
use async_graphql::{http::GraphiQLSource, EmptySubscription, MergedObject, Schema};
use async_graphql_axum::GraphQL;
use axum::response::{Html, IntoResponse};
use axum::routing::get;
use axum::Router;
use mongodb::Client;

#[derive(MergedObject)]
struct Query(UserQuery, InstitutionQuery);

#[derive(MergedObject)]
struct Mutation(UserMutation);

async fn graphiql() -> impl IntoResponse {
	Html(GraphiQLSource::build().endpoint("/graphql").finish())
}

pub fn build_router(
	mongo_client: Client,
	nordigen_client: http_api::nordigen::Client,
	token_manager: TokenManager<NordigenTokenClient>,
) -> Router<()> {
	let schema = Schema::build(
		Query(UserQuery, InstitutionQuery),
		Mutation(UserMutation),
		EmptySubscription,
	)
	.data(mongo_client)
	.data(nordigen_client)
	.data(token_manager)
	.finish();
	Router::new().route("/", get(graphiql).post_service(GraphQL::new(schema)))
}
