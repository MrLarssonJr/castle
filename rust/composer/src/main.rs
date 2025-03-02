use std::collections::HashMap;
use std::error::Error;
use std::io::{stdin, Read};

use interpreter::eval_expression;
use lalrpop_util::lalrpop_mod;


mod ast;
mod interpreter;

//mod parse;

lalrpop_mod!(pub grammar);


fn main() -> Result<(), Box<dyn Error>> {
	let mut input = String::new();
	stdin().lock().read_to_string(&mut input)?;

	let ast: ast::Expression = match grammar::ExpressionParser::new().parse(&input) {
		Ok(ast) => ast,
		Err(err) => {
			print_err(err);
			return Ok(());
		},
	};
	let res = eval_expression(&HashMap::new(), &ast);
	println!("{res:?}");

	Ok(())
}

fn print_err(err: impl Error) {
	let mut e = Some(&err as &dyn Error);

	while let Some(err) = e.take() {
		println!("{err}");
		e = err.source();
	}
}
