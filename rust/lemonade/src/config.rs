use config::Config;

#[derive(Debug, Config)]
pub struct Config {
	#[env = "PG_USER"]
	pub pg_user: String,
	#[env = "PG_PASSWORD"]
	pub pg_password: String,
	#[env = "PG_HOST"]
	pub pg_host: String,
	#[env = "PG_DB"]
	pub pg_db: String,

	#[env = "NORDIGEN_SECRET_ID"]
	pub nordigen_secret_id: String,
	#[env = "NORDIGEN_SECRET_KEY"]
	pub nordigen_secret_key: String,
}

impl Config {
	pub fn pg_connection_url(&self) -> String {
		let Config {
			pg_user,
			pg_password,
			pg_host,
			pg_db,
			..
		} = self;

		format!("postgresql://{pg_user}:{pg_password}@{pg_host}:5432/{pg_db}")
	}
}
