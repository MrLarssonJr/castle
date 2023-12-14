use reqwest::{Client, Response, StatusCode};
use serde::Serialize;
use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NordigenError {
	#[error("an http error occurred while performing operation")]
	HTTP(#[from] reqwest::Error),
	#[error("invalid credentials: {0}")]
	Unauthenticated(Box<str>),
	#[error("not authorized to create new token: {0}")]
	Unauthorized(Box<str>),
	#[error("rate limit reached: {0}")]
	RateLimited(Box<str>),
	#[error("received an unknown response: {0}")]
	UnknownResponse(Box<str>),
}

pub mod token {
	use reqwest::{Client};
	use serde::{Deserialize, Serialize};

	use super::NordigenError;

	#[derive(Serialize)]
	pub struct NewArgs<'a> {
		pub secret_id: &'a str,
		pub secret_key: &'a str,
	}


	#[derive(Deserialize)]
	pub struct NewReturn {
		pub access: Box<str>,
		pub access_expires: i64,
		pub refresh: Box<str>,
		pub refresh_expires: i64,
	}

	pub async fn new<'args>(client: &Client, args: NewArgs<'args>) -> Result<NewReturn, NordigenError> {
		super::get(client, "https://ob.gocardless.com/api/v2/token/new/", args).await
	}

	#[derive(Serialize)]
	pub struct RefreshArgs<'a> {
		pub refresh: &'a str,
	}


	#[derive(Deserialize)]
	pub struct RefreshReturn {
		pub access: Box<str>,
		pub access_expires: i64,
	}

	pub async fn refresh<'args>(client: &Client, args: RefreshArgs<'args>) -> Result<RefreshReturn, NordigenError> {
		super::get(client, "https://ob.gocardless.com/api/v2/token/refresh/", args).await
	}
}

async fn get<Args, Return>(client: &Client, url: &str, args: Args) -> Result<Return, NordigenError> where Args: Serialize, Return: DeserializeOwned {
	use NordigenError as E;

	let res = client.post(url)
		.json(&args)
		.send()
		.await?;

	match res.status() {
		StatusCode::OK => Ok(res.json::<Return>().await?),

		StatusCode::UNAUTHORIZED => Err(E::Unauthenticated(get_detail(res).await?)),

		StatusCode::FORBIDDEN => Err(E::Unauthorized(get_detail(res).await?)),

		StatusCode::TOO_MANY_REQUESTS => Err(E::RateLimited(get_detail(res).await?)),

		_ => {
			let status = res.status();
			if let Ok(body) = res.text().await {
				Err(E::UnknownResponse(Box::from(format!("status: {status}, body: {body}"))))
			} else {
				Err(E::UnknownResponse(Box::from(format!("status: {status}, unable to get body"))))
			}
		}
	}
}

async fn get_detail(res: Response) -> Result<Box<str>, reqwest::Error> {
	let json = res.json::<serde_json::Value>().await?;

	let detail = json.get("detail");
	let detail = detail.and_then(serde_json::Value::as_str);
	let detail: Box<str> = detail.map(Box::from).unwrap_or_else(|| Box::from("no detail"));

	Ok(detail)
}
