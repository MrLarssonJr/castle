use std::rc::Rc;

pub type Expr = Rc<Expression>;

#[derive(Debug, PartialEq)]
pub enum Expression {
	Identifier(IdentifierExpression),
	FunctionApplication(FunctionApplicationExpression),
	FunctionDefinition(FunctionDefinitionExpression),
	NumberLiteral(NumberLiteralExpression),
	Negation(NegationExpression),
	Addition(AdditionExpression),
	Subtraction(SubtractionExpression),
	Product(ProductExpression),
	Division(DivisionExpression),
	LetIn(LetInExpression),
	IfElse(IfElseExpression),
	Or(OrExpression),
	And(AndExpression),
	LessThan(LessThanExpression),
	LessThanOrEqualTo(LessThanOrEqualToExpression),
	EqualTo(EqualToExpression),
	NotEqualTo(NotEqualToExpression),
	GreaterThanOrEqualTo(GreaterThanOrEqualToExpression),
	GreaterThan(GreaterThanExpression),
	Not(NotExpression),
}

impl Expression {
	pub fn product(left_hand_side: Expression, tail: Vec<(&str, Expression)>) -> Expression {
		let mut res = left_hand_side;

		for (operator, right_hand_side) in tail {
			res = match operator {
				"*" => Expression::Product(ProductExpression {
					left_hand_side: Rc::new(res),
					right_hand_side: Rc::new(right_hand_side),
				}),
				"/" => Expression::Division(DivisionExpression {
					left_hand_side: Rc::new(res),
					right_hand_side: Rc::new(right_hand_side),
				}),
				_ => panic!("bad product operator: {operator}"),
			};
		}

		res
	}

	pub fn sum(left_hand_side: Expression, tail: Vec<(&str, Expression)>) -> Expression {
		let mut res = left_hand_side;

		for (operator, right_hand_side) in tail {
			res = match operator {
				"+" => Expression::Addition(AdditionExpression {
					left_hand_side: Rc::new(res),
					right_hand_side: Rc::new(right_hand_side),
				}),
				"-" => Expression::Subtraction(SubtractionExpression {
					left_hand_side: Rc::new(res),
					right_hand_side: Rc::new(right_hand_side),
				}),
				_ => panic!("bad summation operator: {operator}"),
			};
		}

		res
	}

	pub fn lazy_boolean_binary(left_hand_side: Expression, tail: Vec<(&str, Expression)>) -> Expression {
		let mut res = left_hand_side;

		for (operator, right_hand_side) in tail {
			res = match operator {
				"||" => Expression::Or(OrExpression {
					left_hand_side: Rc::new(res),
					right_hand_side: Rc::new(right_hand_side),
				}),
				"&&" => Expression::And(AndExpression {
					left_hand_side: Rc::new(res),
					right_hand_side: Rc::new(right_hand_side),
				}),
				_ => panic!("bad summation operator: {operator}"),
			};
		}

		res
	}

	pub fn eager_boolean_binary(left_hand_side: Expression, tail: Vec<(&str, Expression)>) -> Expression {
		let mut res = left_hand_side;

		for (operator, right_hand_side) in tail {
			res = match operator {
				"<" => Expression::LessThan(LessThanExpression {
					left_hand_side: Rc::new(res),
					right_hand_side: Rc::new(right_hand_side),
				}),
				"<=" => Expression::LessThanOrEqualTo(LessThanOrEqualToExpression {
					left_hand_side: Rc::new(res),
					right_hand_side: Rc::new(right_hand_side),
				}),
				"==" => Expression::EqualTo(EqualToExpression {
					left_hand_side: Rc::new(res),
					right_hand_side: Rc::new(right_hand_side),
				}),
				"!=" => Expression::NotEqualTo(NotEqualToExpression {
					left_hand_side: Rc::new(res),
					right_hand_side: Rc::new(right_hand_side),
				}),
				">=" => Expression::GreaterThanOrEqualTo(GreaterThanOrEqualToExpression {
					left_hand_side: Rc::new(res),
					right_hand_side: Rc::new(right_hand_side),
				}),
				">" => Expression::GreaterThan(GreaterThanExpression {
					left_hand_side: Rc::new(res),
					right_hand_side: Rc::new(right_hand_side),
				}),
				_ => panic!("bad summation operator: {operator}"),
			};
		}

		res
	}

	pub fn unary(operators: Vec<&str>, atom: Expression) -> Expression {
		let mut res = atom;

		for operator in operators {
			res = match operator {
				"-" => Expression::Negation(NegationExpression(Rc::new(res))),
				_ => panic!("bad unary operator: {operator}"),
			};
		}

		res
	}

	pub fn eager_boolean_unary(operators: Vec<&str>, atom: Expression) -> Expression {
		let mut res = atom;

		for operator in operators {
			res = match operator {
				"!" => Expression::Not(NotExpression(Rc::new(res))),
				_ => panic!("bad unary operator: {operator}"),
			};
		}

		res
	}
}

#[derive(Debug, PartialEq)]
pub struct OrExpression {
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub struct AndExpression {
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub struct LessThanExpression {
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub struct LessThanOrEqualToExpression {
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub struct EqualToExpression {
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub struct NotEqualToExpression {
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub struct GreaterThanOrEqualToExpression {
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub struct GreaterThanExpression {
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub struct NotExpression(pub Expr);

#[derive(Debug, PartialEq)]
pub struct AdditionExpression {
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub struct SubtractionExpression {
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub struct ProductExpression {
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub struct DivisionExpression {
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub struct NegationExpression(pub Expr);

#[derive(Debug, PartialEq)]
pub struct NumberLiteralExpression(pub isize);

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub struct Identifier(pub String);

#[derive(Debug, PartialEq)]
pub struct IdentifierExpression(pub Identifier);

#[derive(Debug, PartialEq, Clone)]
pub struct FunctionDefinitionExpression {
	pub parameters: Vec<Identifier>,
	pub body: Expr,
}

#[derive(Debug, PartialEq)]
pub struct FunctionApplicationExpression {
	pub function: Expr,
	pub arguments: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
pub struct LetInExpression {
	pub binding: Identifier,
	pub value: Expr,
	pub body: Expr,
}

#[derive(Debug, PartialEq)]
pub struct IfElseExpression {
	pub condition: Expr,
	pub then_branch: Expr,
	pub else_branch: Expr,
}
