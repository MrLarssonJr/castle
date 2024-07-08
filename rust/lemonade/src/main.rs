use crate::api::Api;
use crate::config::Config;
use crate::model::User;
use crate::nordigen_token_client::NordigenTokenClient;
use crate::token_manager::TokenManager;
use ::config::FromConfig;
use ::error::ResultExt;
use mongodb::Client;
use std::time::Duration;
use tracing::info;
use uuid::{NoContext, Timestamp, Uuid};

mod api;
mod config;
mod error;
mod model;
mod nordigen_token_client;
mod token_manager;

fn main() -> anyhow::Result<()> {
	tracing_subscriber::fmt().compact().init();
	info!("initialized tracing subscriber");

	let config = Config::parse().must();
	info!("parsed config");

	tokio::runtime::Builder::new_multi_thread()
		.enable_all()
		.build()
		.unwrap()
		.block_on(async_entrypoint(config))?;

	Ok(())
}

async fn async_entrypoint(config: Config) -> anyhow::Result<()> {
	let mongo_client = Client::with_options(config.db_connection_options())?;
	info!("initialised mongo client");

	let nordigen_client = http_api::nordigen::Client::new("https://bankaccountdata.gocardless.com");
	let token_manager = TokenManager::new(
		config.nordigen_secret_id,
		config.nordigen_secret_key,
		NordigenTokenClient::new(nordigen_client.clone()),
	);

	Api::run(mongo_client, nordigen_client, token_manager).await;

	Ok(())
}
