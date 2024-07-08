use lalrpop::process_root;
use std::error::Error;

fn main() {
	match process_root() {
		Ok(()) => (),
		Err(e) => print_err(e),
	}
}

fn print_err(e: Box<dyn Error>) {
	eprintln!("{e}");

	let mut source = e.source();

	while let Some(err) = source {
		eprintln!("  due to {err}");
		source = err.source();
	}
}
