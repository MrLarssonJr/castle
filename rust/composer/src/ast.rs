use std::rc::Rc;

pub type Expr = Rc<Expression>;

#[derive(Debug, PartialEq)]
pub enum Expression {
	Identifier(IdentifierExpression),
	FunctionApplication(FunctionApplicationExpression),
	FunctionDefinition(FunctionDefinitionExpression),
	NumberLiteral(NumberLiteralExpression),
	BooleanLiteral(BooleanLiteralExpression),
	LetIn(LetInExpression),
	IfElse(IfElseExpression),
	UnaryOperation(UnaryOperationExpression),
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

	pub fn unary(mut operators: Vec<&str>, operand: Expression) -> Expression {
		let mut res = operand;
		
		operators.reverse();
		for operator in operators {
			let operation = match operator {
				"-" => UnaryOperator::Negate,
				"!" => UnaryOperator::Not,
				_ => panic!("bad unary operator: {operator}"),
			};

			res = Expression::UnaryOperation(UnaryOperationExpression { operation, operand: Rc::new(res) });
		}

		res
	}
}

#[derive(Debug, PartialEq)]
pub struct NumberLiteralExpression(pub isize);

#[derive(Debug, PartialEq)]
pub struct BooleanLiteralExpression(pub bool);

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

#[derive(Debug, PartialEq)]
pub struct UnaryOperationExpression {
	pub operation: UnaryOperator,
	pub operand: Expr,
}

#[derive(Debug, PartialEq)]
pub enum UnaryOperator {
	Negate,
	Not,
}
