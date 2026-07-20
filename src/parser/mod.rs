pub mod ast;
mod error;
pub mod lexer;

use lexer::{Lexer, LexerError};
use std::collections::VecDeque;

pub use error::SyntaxError;

use crate::parser::ast::{DougChain, Reference};
use crate::parser::lexer::{KeyWord, ParenThesis, Token};
use crate::runtime::RuntimeError;
use crate::values::Operator;
use crate::values::tape::{Mutator, ROData};
use ast::{Expr, Stmt};

macro_rules! expect_token {
	($self:expr, $pat:pat, $expected:literal) => {
		match $self.consume()? {
			$pat => (),
			other => {
				return Err(SyntaxError::Expected(
					$expected.to_string(),
					other.to_string(),
					$self.row,
					$self.column,
				));
			}
		}
	};

	($self:expr, $pat:pat, $expected:literal, $output:expr) => {
		match $self.consume()? {
			$pat => $output,
			other => {
				return Err(SyntaxError::Expected(
					$expected.to_string(),
					other.to_string(),
					$self.row,
					$self.column,
				));
			}
		}
	};
}

pub struct Parser<'a> {
	lexer: Option<Lexer<'a>>,
	tokens: VecDeque<Token>,
	column: u16,
	row: u16,
}

impl<'a> Mutator<'a> for Parser<'a> {
	type Scope = ROData<'a>;
	type Input = String;
	type Output = Box<[Stmt]>;

	fn run(
		&mut self,
		mem: &'a Self::Scope,
		input: Self::Input,
	) -> Result<Self::Output, crate::runtime::RuntimeError> {
		self.lexer = Some(Lexer::new(input, mem.clone()));

		self.parse()
			.map_err(|e| RuntimeError::SyntaxError(e.to_string()))
	}
}

impl<'a> Parser<'a> {
	#[must_use]
	pub fn new() -> Self {
		Self {
			column: 0,
			row: 0,
			lexer: None,
			tokens: VecDeque::new(),
		}
	}

	/// Takes a [`Token`] from the bottom of the [`Self::tokens`] and removes it, thus shifting the
	/// bottom to the item before it.
	///
	/// If there are no items in [`Self::tokens`], we attempt to [`Self::add_line`].
	///
	/// # Errors
	/// If there are no more tokens, or if we cannot add more lines.
	#[allow(clippy::let_and_return)]
	fn consume(&mut self) -> Result<Token, SyntaxError> {
		self.column += 1;

		while self.tokens.is_empty() {
			self.row += 1;
			self.column = 0;
			self.add_line()?
		}

		let t = self
			.tokens
			.pop_front()
			.ok_or(SyntaxError::NoMoreTokens(self.row, self.column));

		#[cfg(debug_assertions)]
		println!("{t:?}");

		t
	}

	/// Look at the [`Token`] at the bottom of [`Self::tokens`] and don't remove it. Thus this token
	/// will be the one being consumed the next time [`Self::consume`] is called.
	///
	/// # Errors
	/// Because this doesn't mutate [`self`], this function errors when there are no more tokens
	/// left. This is good for separating out things that are dependent on lines.
	#[allow(clippy::let_and_return)]
	fn peek(&self) -> Result<Token, SyntaxError> {
		let t = self
			.tokens
			.front()
			.cloned()
			.ok_or(SyntaxError::NoMoreTokens(self.row, self.column));

		#[cfg(debug_assertions)]
		println!("Peeking at: {t:?}");

		t
	}

	#[allow(unused)]
	fn peek_two(&self) -> Option<Token> {
		self.tokens.get(1).cloned()
	}

	/// Adds more lines to [`Self::tokens`].
	///
	/// This function simply uses [`Self::lexer`] to lex more lines from the file before appending
	/// them to [`Self::tokens`].
	///
	/// # Errors
	/// This fnction only errors out if there is a [`LexerError`], something that should always be
	/// [`LexerError::EOFReached`] or if there is a [`LexerError::InvalidToken`].
	fn add_line(&mut self) -> Result<(), SyntaxError> {
		let tokens = self
			.lexer
			.as_mut()
			.ok_or(SyntaxError::Lexer(
				LexerError::EOFReached,
				self.row,
				self.column,
			))?
			.lex_line()
			.map_err(|e| SyntaxError::Lexer(e, self.row, self.column))?;
		let tokens: &mut VecDeque<_> = &mut tokens.into_vec().into();
		self.tokens.append(tokens);
		Ok(())
	}

	/// Top level function to parse the entire file
	///
	/// # Errors
	/// If the `[Self::parse_block]` beneath it fails.
	pub fn parse(&mut self) -> Result<Box<[Stmt]>, SyntaxError> {
		self.parse_block(true)
	}

	/// Parses a block of nodes. This function takes the [`Token`] inputs from the [`Parser::Lexer`]
	/// and converts them into [`Stmt`]s. This function loops until we
	/// ([`SyntaxError::NoMoreTokens`])[run out of tokens] or we hit another [`SyntaxError`]. This
	/// exits cleanly if we have no more tokens to consume, and don't expect any, and exit on a `]`
	/// or nothing depending on whether this is the top level or not.
	fn parse_block(&mut self, is_top: bool) -> Result<Box<[Stmt]>, SyntaxError> {
		let mut nodes = Vec::new();
		self.row += 1;
		self.column = 0;

		while let Ok(token) = self.consume() {
			match token {
				Token::KeyWord(KeyWord::Tts | KeyWord::Ttss) => {
					if let Ok(msg) = self.parse_expr() {
						nodes.push(Stmt::Tts {
							msg: Some(msg),
							use_index: false,
							overlap: matches!(token, Token::KeyWord(KeyWord::Ttss)),
						});
					} else {
						nodes.push(Stmt::Tts {
							msg: None,
							use_index: true,
							overlap: matches!(token, Token::KeyWord(KeyWord::Ttss)),
						});
					}
				}
				Token::KeyWord(KeyWord::Guod) => {
					if let Ok(value) = self.parse_expr() {
						nodes.push(Stmt::Guod {
							value: Some(value),
							use_index: false,
						})
					} else {
						nodes.push(Stmt::Guod {
							value: None,
							use_index: true,
						})
					}
				}
				Token::KeyWord(KeyWord::Call) => {
					if let Ok(index) = self.parse_expr()
						&& let Expr::Variable(name) = index
					{
						nodes.push(Stmt::Call {
							name: Some(name),
							use_index: false,
						})
					} else {
						nodes.push(Stmt::Call {
							name: None,
							use_index: true,
						})
					}
				}

				Token::KeyWord(KeyWord::Rigged) => {
					if let Ok(Token::Variable(v)) = self.consume() {
						let mut args: Vec<Expr> = Vec::new();
						while let Ok(expr) = self.parse_expr() {
							args.push(expr);
						}
						nodes.push(Stmt::Expr(Expr::Rigged { func: v, args }))
					}
				}
				Token::KeyWord(KeyWord::Set) => {
					nodes.push(Stmt::Set {
						value: self.parse_expr()?,
						oper: None,
					});
				}
				Token::Operator(op) => {
					let (value, _) = self.parse_set_expr()?;
					nodes.push(Stmt::Set {
						value,
						oper: Some(op),
					});
				}
				Token::KeyWord(KeyWord::Loop) => {
					expect_token!(self, Token::Paren(ParenThesis::SquareLeft), "[");
					let body = self.parse_block(false)?;
					nodes.push(Stmt::Loop { body });
				}
				Token::KeyWord(KeyWord::Prediction) => {
					let condition = self.parse_expr()?;
					expect_token!(self, Token::Paren(ParenThesis::SquareLeft), "[");

					let mut believers_body: Box<[Stmt]> = Vec::new().into();
					let mut doubters_body: Box<[Stmt]> = Vec::new().into();

<<<<<<< HEAD
					loop {
						match self.consume()? {
							Token::KeyWord(KeyWord::Believers) => {
								expect_token!(self, Token::KeyWord(KeyWord::Wins), "win");
								expect_token!(self, Token::Paren(ParenThesis::SquareLeft), "[");
								believers_body = self.parse_block(false)?;
							}
							Token::KeyWord(KeyWord::Doubters) => {
								expect_token!(self, Token::KeyWord(KeyWord::Wins), "win");
								expect_token!(self, Token::Paren(ParenThesis::SquareLeft), "[");
								doubters_body = self.parse_block(false)?;
							}
							Token::Paren(ParenThesis::SquareRight) => break,
							other => {
								return Err(SyntaxError::Expected(
									"Believers, Doubters, ]".to_string(),
									other.to_string(),
									self.row,
									self.column,
								));
							}
						}
					}
=======
					loop {
						match self.consume()? {
							Token::KeyWord(KeyWord::Believers) => {
								expect_token!(self, Token::KeyWord(KeyWord::Wins), "win");
								expect_token!(self, Token::Paren(ParenThesis::SquareLeft), "[");
								believers_body = self.parse_block(false)?;
							}
							Token::KeyWord(KeyWord::Doubters) => {
								expect_token!(self, Token::KeyWord(KeyWord::Wins), "win");
								expect_token!(self, Token::Paren(ParenThesis::SquareLeft), "[");
								doubters_body = self.parse_block(false)?;
							}
							Token::Paren(ParenThesis::SquareRight) => break,
							other => {
								return Err(SyntaxError::Expected(
									"believers, doubters, ]".to_string(),
									other.to_string(),
									self.row,
									self.column,
								));
							}
						}
					}
>>>>>>> 3c54d28 (minor fixes)

					nodes.push(Stmt::Prediction {
						believe_body: believers_body,
						doubt_body: doubters_body,
						condition,
					});
				}
				Token::KeyWord(KeyWord::FiveMinuteCodingAdventure) => {
					let Token::Variable(name) = self.consume()? else {
						return Err(SyntaxError::Expected(
							"variable name".to_string(),
							"another".to_string(),
							self.row,
							self.column,
						));
					};

					expect_token!(self, Token::Paren(ParenThesis::SquareLeft), "[");
					let body = self.parse_block(false)?;
					nodes.push(Stmt::FiveMinuteCodingAdventure { name, body })
				}

				Token::KeyWord(KeyWord::Bald | KeyWord::DougChain { .. }) | Token::Variable(_) => {
					nodes.push(self.parse_doug_node(&token)?);

					while let Ok(Token::KeyWord(KeyWord::Set) | Token::Operator(_)) = self.peek() {
						let (value, op) = self.parse_set_expr()?;
						nodes.push(Stmt::Set { value, oper: op });
					}
				}
				Token::KeyWord(KeyWord::EndStream) => {
					nodes.push(Stmt::EndStream);
				}
				Token::Paren(ParenThesis::Left) => {
					let chains = self.parse_doug_expr()?;
					expect_token!(self, Token::Paren(ParenThesis::Right), ")");
					nodes.push(Stmt::Doug {
						chains,
						reset: true,
					});
				}
				Token::Paren(ParenThesis::SquareRight) => {
					if is_top {
						return Err(SyntaxError::Unexpected(
							"] in top level".to_string(),
							self.row,
							self.column,
						));
					}
					return Ok(nodes.into());
				}

				other => {
					return Err(SyntaxError::Unexpected(
						other.to_string(),
						self.row,
						self.column,
					));
				}
			}
		}

		if !is_top {
			return Err(SyntaxError::Expected(
				"]".to_string(),
				String::new(),
				self.row,
				self.column,
			));
		}

		Ok(nodes.into())
	}

	fn parse_doug_node(&mut self, token: &Token) -> Result<Stmt, SyntaxError> {
		let (mut chains, reset) = match token {
			Token::KeyWord(KeyWord::Bald) => (Vec::new(), true),
			Token::KeyWord(KeyWord::DougChain(count)) => {
				(vec![Reference::Doug(DougChain { count: *count })], false)
			}
			Token::Variable(name) => (vec![Reference::Variable(name.clone())], false),
			_ => {
				return Err(SyntaxError::Expected(
					"Bald, Doug, variable name".to_string(),
					token.to_string(),
					self.row,
					self.column,
				));
			}
		};

		while let Ok(Token::KeyWord(KeyWord::DougChain(count))) = self.peek() {
			self.consume()?;
			chains.push(Reference::Doug(DougChain { count }));
		}

		let chains = chains.into();
		Ok(Stmt::Doug { chains, reset })
	}

	fn parse_set_expr(&mut self) -> Result<(Expr, Option<Operator>), SyntaxError> {
		match self.consume()? {
			Token::KeyWord(KeyWord::Set) => {
				let value = self.parse_expr()?;
				Ok((value, None))
			}
			Token::Operator(op) => match self.consume()? {
				Token::KeyWord(KeyWord::Set) => {
					let value = self.parse_expr()?;
					Ok((value, Some(op)))
				}
				other => Err(SyntaxError::Expected(
					"set".to_string(),
					other.to_string(),
					self.row,
					self.column,
				)),
			},
			other => Err(SyntaxError::Expected(
				"set, operator".to_string(),
				other.to_string(),
				self.row,
				self.column,
			)),
		}
	}

	fn parse_doug_expr(&mut self) -> Result<Box<[Reference]>, SyntaxError> {
		let mut dougs = Vec::new();
		while let Ok(token) = self.peek() {
			match token {
				Token::KeyWord(KeyWord::DougChain(count)) => {
					self.consume()?;
					dougs.push(Reference::Doug(DougChain { count }));
				}
				Token::Paren(ParenThesis::Right | ParenThesis::AngleRight) => {
					return Ok(dougs.into());
				}
				Token::Variable(v) => {
					self.consume()?;
					dougs.push(Reference::Variable(v));
				}
				_ => {
					return Err(SyntaxError::Expected(
						"Doug, Closing Brace".to_string(),
						token.to_string(),
						self.row,
						self.column,
					));
				}
			}
		}

		Ok(dougs.into())
	}

	fn tokens_to_expr(&mut self) -> Result<Box<Expr>, SyntaxError> {
		let expr = match self.peek() {
			Ok(Token::Paren(ParenThesis::Left)) => {
				self.consume()?;
				let expr = self.parse_expr()?;
				expect_token!(self, Token::Paren(ParenThesis::Right), ")");

				Box::new(expr)
			}
			Ok(Token::Literal(lit)) => {
				self.consume()?;
				Box::new(Expr::Literal(lit))
			}
			Ok(Token::KeyWord(KeyWord::DougChain(_)) | Token::KeyWord(KeyWord::Bald)) => {
				Box::new(Expr::DougSequence {
					chains: self.parse_doug_expr()?,
				})
			}

			Ok(Token::Variable(_))
				if let Some(Token::KeyWord(KeyWord::DougChain(_))) = self.peek_two() =>
			{
				Box::new(Expr::DougSequence {
					chains: self.parse_doug_expr()?,
				})
			}
			Ok(Token::Variable(v)) => {
				self.consume()?;
				Box::new(Expr::Variable(v))
			}

			Ok(Token::KeyWord(KeyWord::Call)) if let Some(Token::Variable(s)) = self.peek_two() => {
				self.consume()?;
				self.consume()?;
				Box::new(Expr::FiveMinuteCodingAdventureCall { name: Some(s) })
			}

			Ok(Token::KeyWord(KeyWord::Rigged))
				if let Some(Token::Variable(v)) = self.peek_two() =>
			{
				self.consume()?;
				self.consume()?;
				let mut args: Vec<Expr> = Vec::new();
				while let Ok(expr) = self.parse_expr() {
					args.push(expr);
				}
				Box::new(Expr::Rigged { func: v, args })
			}

			Ok(Token::Paren(ParenThesis::Right)) => {
				return Err(SyntaxError::BreakFromExprTree);
			}

			Ok(other) => {
				return Err(SyntaxError::Expected(
					"expression".to_string(),
					other.to_string(),
					self.row,
					self.column,
				));
			}
			Err(_) => return Err(SyntaxError::BreakFromExprTree),
		};
		Ok(expr)
	}

	fn parse_expr(&mut self) -> Result<Expr, SyntaxError> {
		let left = self.tokens_to_expr()?;

		let op = match self.peek() {
			Ok(Token::Operator(op)) => {
				if let Some(Token::KeyWord(KeyWord::Set)) = self.peek_two() {
					None
				} else {
					self.consume()?;
					Some(op)
				}
			}
			_ => None,
		};

		let Some(op) = op else {
			return Ok(*left);
		};

		let right = self.tokens_to_expr().ok();

		Ok(Expr::Condition {
			left,
			operator: Some(op),
			right,
		})
	}
}

impl<'a> Default for Parser<'a> {
	fn default() -> Self {
		Self::new()
	}
}
