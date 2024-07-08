pub use eval::eval_expression;

mod eval {
	use std::collections::HashMap;

	use thiserror::Error;

	use crate::ast::{
		AdditionExpression, AndExpression, DivisionExpression, EqualToExpression, Expression, FunctionApplicationExpression, FunctionDefinitionExpression, GreaterThanExpression, GreaterThanOrEqualToExpression, Identifier, IdentifierExpression, IfElseExpression, LessThanExpression, LessThanOrEqualToExpression, LetInExpression, NegationExpression, NotEqualToExpression, NotExpression, NumberLiteralExpression, OrExpression, ProductExpression, SubtractionExpression
	};
	use crate::interpreter::value::{NumberValue, Value};

	use super::value::{BooleanValue, FunctionValue};

	pub fn eval_expression(
		bindings: &HashMap<Identifier, Value>,
		ast: &Expression,
	) -> Result<Value, RuntimeException> {
		match ast {
			Expression::NumberLiteral(number_literal) => {
				eval_number_literal(bindings, number_literal)
			}
			Expression::Negation(negation) => eval_negation(bindings, negation),
			Expression::Addition(addition) => eval_addition(bindings, addition),
			Expression::Subtraction(subtraction) => eval_subtraction(bindings, subtraction),
			Expression::Identifier(identifier) => eval_identifier(bindings, identifier),
			Expression::FunctionApplication(function_application) => {
				eval_function_application(bindings, function_application)
			}
			Expression::FunctionDefinition(function_definition) => {
				eval_function_definition(bindings, function_definition)
			}
			Expression::Product(product) => eval_product(bindings, product),
			Expression::Division(division) => eval_division(bindings, division),
			Expression::LetIn(let_in) => eval_let_in(bindings, let_in),
			Expression::IfElse(if_else) => eval_if_else(bindings, if_else),
			Expression::Or(or) => eval_or(bindings, or),
			Expression::And(and) => eval_and(bindings, and),
			Expression::LessThan(less_than) => eval_less_than(bindings, less_than),
			Expression::LessThanOrEqualTo(less_than_or_equal_to) => eval_less_than_or_equal_to(bindings, less_than_or_equal_to),
			Expression::EqualTo(equal_to) => eval_equal_to(bindings, equal_to),
			Expression::NotEqualTo(not_equal_to) => eval_not_equal_to(bindings, not_equal_to),
			Expression::GreaterThanOrEqualTo(greater_than_or_equal_to) => eval_greater_than_or_equal_to(bindings, greater_than_or_equal_to),
			Expression::GreaterThan(greater_than) => eval_greater_than(bindings, greater_than),
			Expression::Not(not) => eval_not(bindings, not),
		}
	}

	pub fn eval_or(
		bindings: &HashMap<Identifier, Value>,
		or: &OrExpression,
	) -> Result<Value, RuntimeException> {
		let OrExpression {
			left_hand_side,
			right_hand_side,
		} = or;

		let lazy_eval = |side| Ok(eval_expression(bindings, side)?.into_boolean()?.0);
		Ok(Value::Boolean(BooleanValue(lazy_eval(left_hand_side)? || lazy_eval(right_hand_side)?)))
	}

	pub fn eval_and(
		bindings: &HashMap<Identifier, Value>,
		and: &AndExpression,
	) -> Result<Value, RuntimeException> {
		let AndExpression {
			left_hand_side,
			right_hand_side,
		} = and;

		let lazy_eval = |side| Ok(eval_expression(bindings, side)?.into_boolean()?.0);
		Ok(Value::Boolean(BooleanValue(lazy_eval(left_hand_side)? && lazy_eval(right_hand_side)?)))
	}

	pub fn eval_less_than(
		bindings: &HashMap<Identifier, Value>,
		less_than: &LessThanExpression,
	) -> Result<Value, RuntimeException> {
		let LessThanExpression {
			left_hand_side,
			right_hand_side,
		} = less_than;

		let NumberValue(lhs_val) = eval_expression(bindings, left_hand_side)?.into_number()?;
		let NumberValue(rhs_val) = eval_expression(bindings, right_hand_side)?.into_number()?;

		Ok(Value::Boolean(BooleanValue(lhs_val < rhs_val)))
	}

	pub fn eval_less_than_or_equal_to(
		bindings: &HashMap<Identifier, Value>,
		less_than_or_equal_to: &LessThanOrEqualToExpression,
	) -> Result<Value, RuntimeException> {
		let LessThanOrEqualToExpression {
			left_hand_side,
			right_hand_side,
		} = less_than_or_equal_to;

		let NumberValue(lhs_val) = eval_expression(bindings, left_hand_side)?.into_number()?;
		let NumberValue(rhs_val) = eval_expression(bindings, right_hand_side)?.into_number()?;

		Ok(Value::Boolean(BooleanValue(lhs_val <= rhs_val)))
	}

	pub fn eval_equal_to(
		bindings: &HashMap<Identifier, Value>,
		equal_to: &EqualToExpression,
	) -> Result<Value, RuntimeException> {
		let EqualToExpression {
			left_hand_side,
			right_hand_side,
		} = equal_to;

		let lhs = eval_expression(bindings, left_hand_side)?;
		let rhs = eval_expression(bindings, right_hand_side)?;

		match (lhs, rhs) {
			(Value::Number(NumberValue(lhs)), rhs) => Ok(Value::Boolean(BooleanValue(lhs == rhs.into_number()?.0))),
			(Value::Boolean(BooleanValue(lhs)), rhs) => Ok(Value::Boolean(BooleanValue(lhs == rhs.into_boolean()?.0))),
			(Value::Function(_), _) => return Err(RuntimeException::UnexpectedType { actual: "function", expected: "number or boolean" }),
		}
	}

	pub fn eval_not_equal_to(
		bindings: &HashMap<Identifier, Value>,
		not_equal_to: &NotEqualToExpression,
	) -> Result<Value, RuntimeException> {
		let NotEqualToExpression {
			left_hand_side,
			right_hand_side,
		} = not_equal_to;

		let lhs = eval_expression(bindings, left_hand_side)?;
		let rhs = eval_expression(bindings, right_hand_side)?;

		match (lhs, rhs) {
			(Value::Number(NumberValue(lhs)), rhs) => Ok(Value::Boolean(BooleanValue(lhs == rhs.into_number()?.0))),
			(Value::Boolean(BooleanValue(lhs)), rhs) => Ok(Value::Boolean(BooleanValue(lhs == rhs.into_boolean()?.0))),
			(Value::Function(_), _) => return Err(RuntimeException::UnexpectedType { actual: "function", expected: "number or boolean" }),
		}
	}

	pub fn eval_greater_than_or_equal_to(
		bindings: &HashMap<Identifier, Value>,
		greater_than_or_equal_to: &GreaterThanOrEqualToExpression,
	) -> Result<Value, RuntimeException> {
		let GreaterThanOrEqualToExpression {
			left_hand_side,
			right_hand_side,
		} = greater_than_or_equal_to;

		let NumberValue(lhs_val) = eval_expression(bindings, left_hand_side)?.into_number()?;
		let NumberValue(rhs_val) = eval_expression(bindings, right_hand_side)?.into_number()?;

		Ok(Value::Boolean(BooleanValue(lhs_val >= rhs_val)))
	}

	pub fn eval_greater_than(
		bindings: &HashMap<Identifier, Value>,
		greater_than: &GreaterThanExpression,
	) -> Result<Value, RuntimeException> {
		let GreaterThanExpression {
			left_hand_side,
			right_hand_side,
		} = greater_than;

		let NumberValue(lhs_val) = eval_expression(bindings, left_hand_side)?.into_number()?;
		let NumberValue(rhs_val) = eval_expression(bindings, right_hand_side)?.into_number()?;

		Ok(Value::Boolean(BooleanValue(lhs_val > rhs_val)))
	}

	pub fn eval_not(
		bindings: &HashMap<Identifier, Value>,
		not: &NotExpression,
	) -> Result<Value, RuntimeException> {
		let NotExpression(unary) = not;

		let BooleanValue(val) = eval_expression(bindings, unary)?.into_boolean()?;

		Ok(Value::Boolean(BooleanValue(!val)))
	}

	pub fn eval_product(
		bindings: &HashMap<Identifier, Value>,
		product: &ProductExpression,
	) -> Result<Value, RuntimeException> {
		let ProductExpression {
			left_hand_side,
			right_hand_side,
		} = product;

		let NumberValue(lhs_val) = eval_expression(bindings, left_hand_side)?.into_number()?;
		let NumberValue(rhs_val) = eval_expression(bindings, right_hand_side)?.into_number()?;

		Ok(Value::Number(NumberValue(lhs_val * rhs_val)))
	}

	pub fn eval_division(
		bindings: &HashMap<Identifier, Value>,
		division: &DivisionExpression,
	) -> Result<Value, RuntimeException> {
		let DivisionExpression {
			left_hand_side,
			right_hand_side,
		} = division;

		let NumberValue(lhs_val) = eval_expression(bindings, left_hand_side)?.into_number()?;
		let NumberValue(rhs_val) = eval_expression(bindings, right_hand_side)?.into_number()?;

		Ok(Value::Number(NumberValue(lhs_val / rhs_val)))
	}

	pub fn eval_addition(
		bindings: &HashMap<Identifier, Value>,
		addition: &AdditionExpression,
	) -> Result<Value, RuntimeException> {
		let AdditionExpression {
			left_hand_side,
			right_hand_side,
		} = addition;

		let NumberValue(lhs_val) = eval_expression(bindings, left_hand_side)?.into_number()?;
		let NumberValue(rhs_val) = eval_expression(bindings, right_hand_side)?.into_number()?;

		Ok(Value::Number(NumberValue(lhs_val + rhs_val)))
	}

	pub fn eval_subtraction(
		bindings: &HashMap<Identifier, Value>,
		subtraction: &SubtractionExpression,
	) -> Result<Value, RuntimeException> {
		let SubtractionExpression {
			left_hand_side,
			right_hand_side,
		} = subtraction;

		let NumberValue(lhs_val) = eval_expression(bindings, left_hand_side)?.into_number()?;
		let NumberValue(rhs_val) = eval_expression(bindings, right_hand_side)?.into_number()?;

		Ok(Value::Number(NumberValue(lhs_val - rhs_val)))
	}

	pub fn eval_negation(
		bindings: &HashMap<Identifier, Value>,
		negation: &NegationExpression,
	) -> Result<Value, RuntimeException> {
		let NegationExpression(unary) = negation;

		let NumberValue(unary_val) = eval_expression(bindings, unary)?.into_number()?;

		Ok(Value::Number(NumberValue(-unary_val)))
	}

	pub fn eval_number_literal(
		_bindings: &HashMap<Identifier, Value>,
		number_literal: &NumberLiteralExpression,
	) -> Result<Value, RuntimeException> {
		let NumberLiteralExpression(val) = number_literal;

		Ok(Value::Number(NumberValue(*val)))
	}

	pub fn eval_identifier(
		bindings: &HashMap<Identifier, Value>,
		identifier: &IdentifierExpression,
	) -> Result<Value, RuntimeException> {
		let IdentifierExpression(identifier) = identifier;

		let Some(val) = bindings.get(identifier) else {
			return Err(RuntimeException::UnboundIdentifier(identifier.clone()));
		};

		Ok(val.clone())
	}

	pub fn eval_function_definition(
		bindings: &HashMap<Identifier, Value>,
		function_definition: &FunctionDefinitionExpression,
	) -> Result<Value, RuntimeException> {
		let closure = bindings.clone();

		let FunctionDefinitionExpression { parameters, body } = function_definition.clone();

		Ok(Value::Function(FunctionValue {
			closure,
			parameters,
			body,
		}))
	}

	pub fn eval_function_application(
		bindings: &HashMap<Identifier, Value>,
		function_application: &FunctionApplicationExpression,
	) -> Result<Value, RuntimeException> {
		let FunctionApplicationExpression {
			function,
			arguments,
		} = function_application;

		let FunctionValue {
			closure,
			parameters,
			body,
		} = eval_expression(bindings, function)?.into_function()?;

		if parameters.len() != arguments.len() {
			return Err(RuntimeException::MismatchedArity {
				actual: arguments.len(),
				expected: parameters.len(),
			});
		}

		let mut new_bindings = closure;
		for (parameter, argument) in parameters.into_iter().zip(arguments) {
			let argument = eval_expression(bindings, argument)?;
			new_bindings.insert(parameter, argument);
		}

		eval_expression(&new_bindings, &body)
	}

	pub fn eval_let_in(bindings: &HashMap<Identifier, Value>, let_in: &LetInExpression) -> Result<Value, RuntimeException> {
		let LetInExpression { binding, value, body } = let_in;

		let value = eval_expression(bindings, value)?;
		let mut new_bindings = bindings.clone();
		new_bindings.insert(binding.clone(), value);

		eval_expression(&new_bindings, body)
	}

	pub fn eval_if_else(bindings: &HashMap<Identifier, Value>, if_else: &IfElseExpression) -> Result<Value, RuntimeException> {
		let IfElseExpression { condition, then_branch, else_branch } = if_else;

		let BooleanValue(condition) = eval_expression(bindings, condition)?.into_boolean()?;

		if condition {
			eval_expression(bindings, then_branch)
		} else {
			eval_expression(bindings, else_branch)
		}
	}

	#[derive(Debug, Error)]
	pub enum RuntimeException {
		#[error("{} not bound", 0.0)]
		UnboundIdentifier(Identifier),
		#[error("expected type {expected}, got {actual}")]
		UnexpectedType {
			actual: &'static str,
			expected: &'static str,
		},
		#[error("expected {expected} arguments, got {actual}")]
		MismatchedArity { actual: usize, expected: usize },
	}
}

pub(super) mod value {
	use std::collections::HashMap;

	use crate::ast::{Expr, Identifier};

	use super::eval::RuntimeException;

	#[derive(Debug, PartialEq, Clone)]
	pub enum Value {
		Number(NumberValue),
		Function(FunctionValue),
		Boolean(BooleanValue),
	}

	impl Value {
		pub fn into_number(self) -> Result<NumberValue, RuntimeException> {
			match self {
				Value::Number(value) => Ok(value),
				Value::Function(_) => Err(RuntimeException::UnexpectedType {
					actual: "function",
					expected: "number",
				}),
				Value::Boolean(_) => Err(RuntimeException::UnexpectedType {
					actual: "boolean",
					expected: "number",
				}),
			}
		}

		pub fn into_function(self) -> Result<FunctionValue, RuntimeException> {
			match self {
				Value::Number(_) => Err(RuntimeException::UnexpectedType {
					actual: "number",
					expected: "function",
				}),
				Value::Function(value) => Ok(value),
				Value::Boolean(_) => Err(RuntimeException::UnexpectedType {
					actual: "boolean",
					expected: "number",
				}),
			}
		}

		pub fn into_boolean(self) -> Result<BooleanValue, RuntimeException> {
			match self {
				Value::Number(_) => Err(RuntimeException::UnexpectedType {
					actual: "number",
					expected: "function",
				}),
				Value::Function(_) => Err(RuntimeException::UnexpectedType {
					actual: "function",
					expected: "number",
				}),
				Value::Boolean(value) => Ok(value),
			}
		}
	}

	#[derive(Debug, PartialEq, Clone)]
	pub struct NumberValue(pub isize);

	#[derive(Debug, PartialEq, Clone)]
	pub struct FunctionValue {
		pub closure: HashMap<Identifier, Value>,
		pub parameters: Vec<Identifier>,
		pub body: Expr,
	}

	#[derive(Debug, PartialEq, Clone)]
	pub struct BooleanValue(pub bool);
}
