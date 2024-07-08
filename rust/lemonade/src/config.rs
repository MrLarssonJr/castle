use config::Config;
use mongodb::options::{ClientOptions, Credential, ServerAddress};

#[derive(Debug, Config)]
pub struct Config {
	#[env = "DB_USER"]
	pub db_user: String,
	#[env = "DB_PASSWORD"]
	pub db_password: String,
	#[env = "DB_HOST"]
	pub db_host: String,
	#[env = "DB_PORT"]
	pub db_port: u16,

	#[env = "NORDIGEN_SECRET_ID"]
	pub nordigen_secret_id: String,
	#[env = "NORDIGEN_SECRET_KEY"]
	pub nordigen_secret_key: String,
}

impl Config {
	pub fn db_connection_options(&self) -> ClientOptions {
		let Config {
			db_user,
			db_password,
			db_host,
			db_port,
			..
		} = self;

		let cred = Credential::builder()
			.username(db_user.to_string())
			.password(db_password.to_string())
			.build();

		let server_addresses = vec![ServerAddress::Tcp {
			host: db_host.to_string(),
			port: Some(*db_port),
		}];

		ClientOptions::builder()
			.app_name("lemonade".to_string())
			.hosts(server_addresses)
			.credential(cred)
			.build()
	}
}
