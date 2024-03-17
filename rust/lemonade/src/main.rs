use ::config::FromConfig;
use ::error::ResultExt;
use chrono::{DateTime, Local};
use sqlx::postgres::PgPoolOptions;
use std::thread::park;

use crate::config::Config;
use crate::nordigen_token_client::NordigenTokenClient;
use crate::token_manager::TokenManager;

mod config;
mod error;
mod nordigen_token_client;
mod token_manager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	println!("hello");
	let config = dbg!(Config::parse().must());

	let pool = PgPoolOptions::new()
		.max_connections(5)
		.connect(&config.pg_connection_url())
		.await?;

	let nordigen_client = http_api::nordigen::Client::new("https://ob.gocardless.com");
	let token_client = NordigenTokenClient::new(&nordigen_client);

	let token_manager = TokenManager::new(
		config.nordigen_secret_id,
		config.nordigen_secret_key,
		token_client,
	);
	let token = token_manager.access_token().await?;
	println!("{token}");

	let now = Local::now();
	let row: (i32, DateTime<Local>) =
		sqlx::query_as("INSERT INTO lemonade.lemonade.log (timestamp) VALUES ( $1 ) RETURNING *;")
			.bind(now)
			.fetch_one(&pool)
			.await?;
	println!("{row:?}");

	loop {
		park()
	}
}
