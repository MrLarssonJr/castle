use config::Config;

#[derive(Config)]
pub struct Config {
	#[env_file = "NORDIGEN_SECRET_ID_FILE"]
	pub pg_user: String,
	#[env_file = "PG_PASSWORD_FILE"]
	pub pg_password: String,
	#[env = "PG_HOST"]
	pub pg_host: String,
	#[env = "PG_DB"]
	pub pg_db: String,

	#[env_file = "NORDIGEN_SECRET_ID_FILE"]
	pub nordigen_secret_id: String,
	#[env_file = "NORDIGEN_SECRET_KEY_FILE"]
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
