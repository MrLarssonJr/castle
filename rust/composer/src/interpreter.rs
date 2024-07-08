pub use eval::eval_expression;

mod eval {
	use std::collections::{HashMap, HashSet};

	use itertools::Itertools;
	use thiserror::Error;

	use crate::ast::{
		BinaryOperationExpression, BinaryOperator, Expression, FunctionApplicationExpression, FunctionDefinitionExpression, Identifier, IdentifierExpression, IfElseExpression, LetInExpression, NegationExpression, NotExpression, NumberLiteralExpression
	};
	use crate::interpreter::value::Value;

	use super::value::{FunctionValue, Type};

	pub fn eval_expression(
		bindings: &HashMap<Identifier, Value>,
		ast: &Expression,
	) -> Result<Value, RuntimeException> {
		match ast {
			Expression::NumberLiteral(number_literal) => {
				eval_number_literal(bindings, number_literal)
			}
			Expression::Negation(negation) => eval_negation(bindings, negation),
			Expression::Identifier(identifier) => eval_identifier(bindings, identifier),
			Expression::FunctionApplication(function_application) => {
				eval_function_application(bindings, function_application)
			}
			Expression::FunctionDefinition(function_definition) => {
				eval_function_definition(bindings, function_definition)
			}
			Expression::LetIn(let_in) => eval_let_in(bindings, let_in),
			Expression::IfElse(if_else) => eval_if_else(bindings, if_else),
			Expression::Not(not) => eval_not(bindings, not),
			Expression::BinaryOperation(binary_operation) => eval_binary_operation(bindings, binary_operation),
			
		}
	}

	pub fn eval_binary_operation(
		bindings: &HashMap<Identifier, Value>,
		binary_operation: &BinaryOperationExpression,
	) -> Result<Value, RuntimeException> {
		let BinaryOperationExpression {
			operation,
			left_hand_side,
			right_hand_side,
		} = binary_operation;

		let lhs = || Ok(eval_expression(bindings, &left_hand_side)?);
		let rhs = || Ok(eval_expression(bindings, &right_hand_side)?);

		match operation {
			BinaryOperator::Addition => Value::add(lhs, rhs),
			BinaryOperator::Subtraction => Value::subtract(lhs, rhs),
			BinaryOperator::Multiplication => Value::multiply(lhs, rhs),
			BinaryOperator::Division => Value::divide(lhs, rhs),
			BinaryOperator::Or => Value::or(lhs, rhs),
			BinaryOperator::And => Value::and(lhs, rhs),
			BinaryOperator::LessThan => Value::less_than(lhs, rhs),
			BinaryOperator::LessThanOrEqualTo => Value::less_than_or_equal_to(lhs, rhs),
			BinaryOperator::EqualTo => Value::equal_to(lhs, rhs),
			BinaryOperator::NotEqualTo => Value::not_equal_to(lhs, rhs),
			BinaryOperator::GreaterThanOrEqualTo => Value::greater_than_or_equal_to(lhs, rhs),
			BinaryOperator::GreaterThan => Value::greater_than(lhs, rhs),
		}
	}

	pub fn eval_not(
		bindings: &HashMap<Identifier, Value>,
		not: &NotExpression,
	) -> Result<Value, RuntimeException> {
		let NotExpression(unary) = not;

		eval_expression(bindings, unary)?.not()
	}

	pub fn eval_negation(
		bindings: &HashMap<Identifier, Value>,
		negation: &NegationExpression,
	) -> Result<Value, RuntimeException> {
		let NegationExpression(unary) = negation;

		eval_expression(bindings, unary)?.negate()
	}

	pub fn eval_number_literal(
		_bindings: &HashMap<Identifier, Value>,
		number_literal: &NumberLiteralExpression,
	) -> Result<Value, RuntimeException> {
		let NumberLiteralExpression(val) = number_literal;

		Ok(Value::Number(*val))
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

		Ok(Value::Function {
			closure,
			parameters,
			body,
		})
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

		let condition = eval_expression(bindings, condition)?.into_boolean()?;

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
		#[error("expected type {}, got {actual}", expected.iter().join(","))]
		UnexpectedType {
			actual: Type,
			expected: HashSet<Type>,
		},
		#[error("expected {expected} arguments, got {actual}")]
		MismatchedArity { actual: usize, expected: usize },
		#[error("expected matching types, got {lhs} and {rhs}")]
		MismatchedOperandTypes { lhs: Type, rhs: Type },
	}
}

pub(super) mod value {
	use std::{collections::HashMap, fmt::Display};

	use crate::ast::{Expr, Identifier};

	use super::eval::RuntimeException;

	#[derive(Debug, PartialEq, Clone)]
	pub enum Value {
		Number(isize),
		Function {
			closure: HashMap<Identifier, Value>,
			parameters: Vec<Identifier>,
			body: Expr,
		},
		Boolean(bool),
	}

	#[derive(Debug, Eq, PartialEq, Clone, Copy, Hash)]
	pub enum Type {
		Number,
		Function,
		Boolean,
	}

	impl Display for Type {
		fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
			match self {
				Type::Number => write!(f, "number"),
				Type::Function => write!(f, "function"),
				Type::Boolean => write!(f, "boolean"),
			}
		}
	}

	impl Value {
		pub const fn get_type(&self) -> Type {
			match self {
				Value::Number(_) => Type::Number,
				Value::Function { .. } => Type::Function,
				Value::Boolean(_) => Type::Boolean,
			}
		}

		pub fn into_number(self) -> Result<isize, RuntimeException> {
			match self {
				Value::Number(val) => Ok(val),
				val => Err(RuntimeException::UnexpectedType { actual: val.get_type(), expected: [Type::Number].into_iter().collect() }),
			}
		}

		pub fn into_boolean(self) -> Result<bool, RuntimeException> {
			match self {
				Value::Boolean(val) => Ok(val),
				val => Err(RuntimeException::UnexpectedType { actual: val.get_type(), expected: [Type::Boolean].into_iter().collect() }),
			}
		}

		pub fn into_function(self) -> Result<FunctionValue, RuntimeException> {
			match self {
				Value::Function { closure, parameters, body } => Ok(FunctionValue { closure, parameters, body }),
				val => Err(RuntimeException::UnexpectedType { actual: val.get_type(), expected: [Type::Function].into_iter().collect() }),
			}
		}

		pub fn not(&self) -> Result<Self, RuntimeException> {
			match self {
				Value::Boolean(val) => Ok(Value::Boolean(!val)),
				operand => Err(RuntimeException::UnexpectedType { actual: operand.get_type(), expected: [Type::Boolean].into_iter().collect() })
			}
		}

		pub fn negate(&self) -> Result<Self, RuntimeException> {
			match self {
				Value::Number(val) => Ok(Value::Number(-val)),
				operand => Err(RuntimeException::UnexpectedType { actual: operand.get_type(), expected: [Type::Number].into_iter().collect() })
			}
		}

		pub fn multiply(lhs: impl Fn() -> Result<Value, RuntimeException>, rhs: impl Fn() -> Result<Value, RuntimeException>) -> Result<Self, RuntimeException> {
			match (lhs()?, rhs()?) {
				(Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs * rhs)),
				(lhs @ Value::Number(_), rhs) => Err(RuntimeException::MismatchedOperandTypes { lhs: lhs.get_type(), rhs: rhs.get_type() }),
				(lhs, _) => Err(RuntimeException::UnexpectedType { actual: lhs.get_type(), expected: [Type::Number].into_iter().collect() }),
			}
		}

		pub fn divide(lhs: impl Fn() -> Result<Value, RuntimeException>, rhs: impl Fn() -> Result<Value, RuntimeException>) -> Result<Self, RuntimeException> {
			match (lhs()?, rhs()?) {
				(Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs / rhs)),
				(lhs @ Value::Number(_), rhs) => Err(RuntimeException::MismatchedOperandTypes { lhs: lhs.get_type(), rhs: rhs.get_type() }),
				(lhs, _) => Err(RuntimeException::UnexpectedType { actual: lhs.get_type(), expected: [Type::Number].into_iter().collect() }),
			}
		}

		pub fn add(lhs: impl Fn() -> Result<Value, RuntimeException>, rhs: impl Fn() -> Result<Value, RuntimeException>) -> Result<Self, RuntimeException> {
			match (lhs()?, rhs()?) {
				(Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs + rhs)),
				(lhs @ Value::Number(_), rhs) => Err(RuntimeException::MismatchedOperandTypes { lhs: lhs.get_type(), rhs: rhs.get_type() }),
				(lhs, _) => Err(RuntimeException::UnexpectedType { actual: lhs.get_type(), expected: [Type::Number].into_iter().collect() }),
			}
		}

		pub fn subtract(lhs: impl Fn() -> Result<Value, RuntimeException>, rhs: impl Fn() -> Result<Value, RuntimeException>) -> Result<Self, RuntimeException> {
			match (lhs()?, rhs()?) {
				(Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Number(lhs - rhs)),
				(lhs @ Value::Number(_), rhs) => Err(RuntimeException::MismatchedOperandTypes { lhs: lhs.get_type(), rhs: rhs.get_type() }),
				(lhs, _) => Err(RuntimeException::UnexpectedType { actual: lhs.get_type(), expected: [Type::Number].into_iter().collect() }),
			}
		}

		pub fn less_than(lhs: impl Fn() -> Result<Value, RuntimeException>, rhs: impl Fn() -> Result<Value, RuntimeException>) -> Result<Self, RuntimeException> {
			match (lhs()?, rhs()?) {
				(Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Boolean(lhs < rhs)),
				(lhs @ Value::Number(_), rhs) => Err(RuntimeException::MismatchedOperandTypes { lhs: lhs.get_type(), rhs: rhs.get_type() }),
				(lhs, _) => Err(RuntimeException::UnexpectedType { actual: lhs.get_type(), expected: [Type::Number].into_iter().collect() }),
			}
		}

		pub fn less_than_or_equal_to(lhs: impl Fn() -> Result<Value, RuntimeException>, rhs: impl Fn() -> Result<Value, RuntimeException>) -> Result<Self, RuntimeException> {
			match (lhs()?, rhs()?) {
				(Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Boolean(lhs <= rhs)),
				(lhs @ Value::Number(_), rhs) => Err(RuntimeException::MismatchedOperandTypes { lhs: lhs.get_type(), rhs: rhs.get_type() }),
				(lhs, _) => Err(RuntimeException::UnexpectedType { actual: lhs.get_type(), expected: [Type::Number].into_iter().collect() }),
			}
		}

		pub fn equal_to(lhs: impl Fn() -> Result<Value, RuntimeException>, rhs: impl Fn() -> Result<Value, RuntimeException>) -> Result<Self, RuntimeException> {
			match (lhs()?, rhs()?) {
				(Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Boolean(lhs == rhs)),
				(lhs @ Value::Number(_), rhs) => Err(RuntimeException::MismatchedOperandTypes { lhs: lhs.get_type(), rhs: rhs.get_type() }),
				(Value::Boolean(lhs), Value::Boolean(rhs)) => Ok(Value::Boolean(lhs == rhs)),
				(lhs @ Value::Boolean(_), rhs) => Err(RuntimeException::MismatchedOperandTypes { lhs: lhs.get_type(), rhs: rhs.get_type() }),
				(lhs, _) => Err(RuntimeException::UnexpectedType { actual: lhs.get_type(), expected: [Type::Number, Type::Boolean].into_iter().collect() }),
			}
		}

		pub fn not_equal_to(lhs: impl Fn() -> Result<Value, RuntimeException>, rhs: impl Fn() -> Result<Value, RuntimeException>) -> Result<Self, RuntimeException> {
			match (lhs()?, rhs()?) {
				(Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Boolean(lhs != rhs)),
				(lhs @ Value::Number(_), rhs) => Err(RuntimeException::MismatchedOperandTypes { lhs: lhs.get_type(), rhs: rhs.get_type() }),
				(Value::Boolean(lhs), Value::Boolean(rhs)) => Ok(Value::Boolean(lhs != rhs)),
				(lhs @ Value::Boolean(_), rhs) => Err(RuntimeException::MismatchedOperandTypes { lhs: lhs.get_type(), rhs: rhs.get_type() }),
				(lhs, _) => Err(RuntimeException::UnexpectedType { actual: lhs.get_type(), expected: [Type::Number, Type::Boolean].into_iter().collect() }),
			}
		}

		pub fn greater_than_or_equal_to(lhs: impl Fn() -> Result<Value, RuntimeException>, rhs: impl Fn() -> Result<Value, RuntimeException>) -> Result<Self, RuntimeException> {
			match (lhs()?, rhs()?) {
				(Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Boolean(lhs >= rhs)),
				(lhs @ Value::Number(_), rhs) => Err(RuntimeException::MismatchedOperandTypes { lhs: lhs.get_type(), rhs: rhs.get_type() }),
				(lhs, _) => Err(RuntimeException::UnexpectedType { actual: lhs.get_type(), expected: [Type::Number].into_iter().collect() }),
			}
		}
		
		pub fn greater_than(lhs: impl Fn() -> Result<Value, RuntimeException>, rhs: impl Fn() -> Result<Value, RuntimeException>) -> Result<Self, RuntimeException> {
			match (lhs()?, rhs()?) {
				(Value::Number(lhs), Value::Number(rhs)) => Ok(Value::Boolean(lhs > rhs)),
				(lhs @ Value::Number(_), rhs) => Err(RuntimeException::MismatchedOperandTypes { lhs: lhs.get_type(), rhs: rhs.get_type() }),
				(lhs, _) => Err(RuntimeException::UnexpectedType { actual: lhs.get_type(), expected: [Type::Number].into_iter().collect() }),
			}
		}


		pub fn or(lhs: impl Fn() -> Result<Value, RuntimeException>, rhs: impl Fn() -> Result<Value, RuntimeException>) -> Result<Self, RuntimeException> {
			match lhs()? {
				Value::Boolean(true) => return Ok(Value::Boolean(true)),
				Value::Boolean(false) => {},
				lhs => return Err(RuntimeException::UnexpectedType { actual: lhs.get_type(), expected: [Type::Boolean].into_iter().collect() })
			}

			match rhs()? {
				Value::Boolean(true) => return Ok(Value::Boolean(true)),
				Value::Boolean(false) => {},
				rhs => return Err(RuntimeException::UnexpectedType { actual: rhs.get_type(), expected: [Type::Boolean].into_iter().collect() })
			}

			Ok(Value::Boolean(false))
		}

		pub fn and(lhs: impl Fn() -> Result<Value, RuntimeException>, rhs: impl Fn() -> Result<Value, RuntimeException>) -> Result<Self, RuntimeException> {
			match lhs()? {
				Value::Boolean(false) => return Ok(Value::Boolean(false)),
				Value::Boolean(true) => {},
				lhs => return Err(RuntimeException::UnexpectedType { actual: lhs.get_type(), expected: [Type::Boolean].into_iter().collect() })
			}

			match rhs()? {
				Value::Boolean(false) => return Ok(Value::Boolean(false)),
				Value::Boolean(true) => {},
				rhs => return Err(RuntimeException::UnexpectedType { actual: rhs.get_type(), expected: [Type::Boolean].into_iter().collect() })
			}

			Ok(Value::Boolean(true))
		}
	}

	#[derive(Debug, PartialEq, Clone)]
	pub struct FunctionValue {
		pub closure: HashMap<Identifier, Value>,
		pub parameters: Vec<Identifier>,
		pub body: Expr,
	}
}
