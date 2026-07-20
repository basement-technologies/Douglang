use crate::parser::lexer::LexerError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SyntaxError {
	#[error("Expected `{0}` got `{1}` at r:{2},c:{3}")]
	Expected(String, String, u16, u16),

	#[error("Unexpected `{0}` at r:{1},c:{2}")]
	Unexpected(String, u16, u16),

	#[error("Lexing error `{0}` at r:{1},c:{2}")]
	Lexer(LexerError, u16, u16),

	#[error("Cannot print literal")]
	NotPrintable,

	#[error("No more tokens left at r:{0},c:{1}")]
	NoMoreTokens(u16, u16),

	#[error("Breaking from expression tree")]
	BreakFromExprTree,
}
