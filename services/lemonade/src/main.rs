use ::config::FromConfig;
use chrono::{DateTime, Duration, Local, Utc};
use error::ResultExt;
use reqwest::Client;
use sqlx::postgres::PgPoolOptions;

use crate::config::Config;
use crate::nordigen::NordigenError;
use crate::token_manager::{AccessAndRefreshToken, Secret, Token, TokenClient, TokenManager};

mod config;
mod nordigen;
mod token_manager;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	println!("hello");
	let config = dbg!(Config::parse().must());

	let pool = PgPoolOptions::new()
		.max_connections(5)
		.connect(&config.pg_connection_url())
		.await?;

	let client = Client::new();

	let token_manager = TokenManager::new(
		config.nordigen_secret_id,
		config.nordigen_secret_key,
		NordigenTokenClient { client: &client },
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

	Ok(())
}

struct NordigenTokenClient<'client> {
	client: &'client Client,
}

impl<'client> TokenClient for NordigenTokenClient<'client> {
	type Error = NordigenError;

	async fn new(&self, secret: &Secret) -> Result<AccessAndRefreshToken, Self::Error> {
		let args = nordigen::token::NewArgs {
			secret_id: secret.id(),
			secret_key: secret.key(),
		};

		let start = Utc::now();
		let res = nordigen::token::new(self.client, args).await?;

		let access = Token::new(res.access, start + Duration::seconds(res.access_expires));

		let refresh = Token::new(res.refresh, start + Duration::seconds(res.refresh_expires));

		Ok(AccessAndRefreshToken { access, refresh })
	}

	async fn refresh(&self, refresh: &str) -> Result<Token, Self::Error> {
		let args = nordigen::token::RefreshArgs { refresh };

		let start = Utc::now();
		let res = nordigen::token::refresh(self.client, args).await?;

		let access = Token::new(res.access, start + Duration::seconds(res.access_expires));

		Ok(access)
	}
}
