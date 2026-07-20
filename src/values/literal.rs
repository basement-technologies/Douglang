use std::fmt::Debug;

use crate::parser::SyntaxError;
use crate::values::Value;
use crate::values::tape::CellPtr;

#[derive(Clone)]
pub enum Literal {
	String(String),
	Number(f64),
}

impl Literal {
	pub fn tts(&self) -> Result<String, SyntaxError> {
		match self {
			Literal::String(s) => Ok(s.clone()),
			Literal::Number(n) => Ok(n.to_string()),
		}
	}

	pub fn to_value(&self) -> Value {
		match self {
			Literal::String(s) => Value::String(s.clone()),
			Literal::Number(n) => Value::Number(*n),
		}
	}
}

impl Debug for Literal {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{:?}", self.tts())
	}
}

impl Debug for CellPtr<Literal> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "A literal")
	}
}
