use config::{Config, FromConfig};
use std::error::Error;

#[derive(Debug, Config)]
struct Config {
	#[env = "BAR"]
	bar: u32,
}

fn main() {
	match Config::parse() {
		Ok(conf) => println!("{conf:?}"),
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
