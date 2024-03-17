use config::{Config, FromConfig};
use std::error::Error;

#[derive(Debug, Config)]
struct Config {
	#[env_file = "BAR_FILE"]
	bar: String,
}

fn main() {
	match Config::parse() {
		Ok(conf) => println!("{}", conf.bar),
		Err(err) => print_error(&err),
	}
}

fn print_error(error: &dyn Error) {
	println!("{}", error);

	if let Some(source) = error.source() {
		println!(" due to ");
		print_error(source);
	}
}
