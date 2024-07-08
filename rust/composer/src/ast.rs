use std::rc::Rc;

pub type Expr = Rc<Expression>;

#[derive(Debug, PartialEq)]
pub enum Expression {
	Identifier(IdentifierExpression),
	FunctionApplication(FunctionApplicationExpression),
	FunctionDefinition(FunctionDefinitionExpression),
	NumberLiteral(NumberLiteralExpression),
	Negation(NegationExpression),
	LetIn(LetInExpression),
	IfElse(IfElseExpression),
	Not(NotExpression),
	BinaryOperation(BinaryOperationExpression),
}

impl Expression {
	pub fn binary_operation(head: Expression, tail: Vec<(&str, Expression)>) -> Expression {
		let mut res = head;

		for (operator, operand) in tail {
			let operation = match operator {
				"*" => BinaryOperator::Multiplication,
				"/" => BinaryOperator::Division,
				"+" => BinaryOperator::Addition,
				"-" => BinaryOperator::Subtraction,
				"||" => BinaryOperator::Or,
				"&&" => BinaryOperator::And,
				"<" => BinaryOperator::LessThan,
				"<=" => BinaryOperator::LessThanOrEqualTo,
				"==" => BinaryOperator::EqualTo,
				"!=" => BinaryOperator::NotEqualTo,
				">=" => BinaryOperator::GreaterThanOrEqualTo,
				">" => BinaryOperator::GreaterThan,
				_ => panic!("bad binary operator: {operator}"),
			};

			res = Expression::BinaryOperation(BinaryOperationExpression {
				operation,
				left_hand_side: Rc::new(res),
				right_hand_side: Rc::new(operand),
			});
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

#[derive(Debug, PartialEq)]
pub struct BinaryOperationExpression {
	pub operation: BinaryOperator,
	pub left_hand_side: Expr,
	pub right_hand_side: Expr,
}

#[derive(Debug, PartialEq)]
pub enum BinaryOperator {
	Addition,
	Subtraction,
	Multiplication,
	Division,
	Or,
	And,
	LessThan,
	LessThanOrEqualTo,
	EqualTo,
	NotEqualTo,
	GreaterThanOrEqualTo,
	GreaterThan,
}
