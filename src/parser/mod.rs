mod error;
mod helpers;

pub use error::ParseErr;

use crate::ast::{Condition, DougChain, Expr, Stmt};
use crate::token::{Token, TokenKind, ValueLiteral};
use helpers::{comp_oper, set_oper, token_name};

pub struct Parser<'a> {
    tokens: &'a [Token],
    i: isize,
    length: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Parser {
            tokens,
            i: -1,
            length: tokens.len(),
        }
    }

    fn next(&mut self) -> Option<&'a Token> {
        self.i += 1;
        if (self.i as usize) < self.length {
            Some(&self.tokens[self.i as usize])
        } else {
            None
        }
    }

    fn check_next(&self) -> Option<&'a Token> {
        let next = self.i + 1;
        if next >= 0 && (next as usize) < self.length {
            Some(&self.tokens[next as usize])
        } else {
            None
        }
    }

    fn eof(&self) -> (usize, usize) {
        if self.length == 0 {
            (1, 1)
        } else {
            let tok = &self.tokens[self.length - 1];
            (tok.line, tok.column)
        }
    }

    fn eof_err(&self, msg: &str) -> ParseErr {
        let (line, column) = self.eof();
        ParseErr::new(line, column, msg)
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseErr> {
        self.process(true)
    }

    fn process(&mut self, is_top: bool) -> Result<Vec<Stmt>, ParseErr> {
        let mut nodes: Vec<Stmt> = Vec::new();

        while let Some(token) = self.next() {
            match &token.kind {
                TokenKind::Tts | TokenKind::Ttss => {
                    let overlap = matches!(token.kind, TokenKind::Ttss);
                    match self.check_next().map(|t| &t.kind) {
                        Some(TokenKind::Literal(value)) => {
                            let value = value.clone();
                            self.next();
                            nodes.push(Stmt::Tts {
                                msg: Some(Expr::Literal(value)),
                                use_index: false,
                                overlap,
                            });
                        }
                        Some(TokenKind::LParen) => {
                            self.next();
                            let expr = self.parse_par_expr()?;
                            nodes.push(Stmt::Tts {
                                msg: Some(expr),
                                use_index: false,
                                overlap,
                            });
                        }
                        _ => {
                            nodes.push(Stmt::Tts {
                                msg: None,
                                use_index: true,
                                overlap,
                            });
                        }
                    }
                }

                TokenKind::Set
                | TokenKind::AddSet
                | TokenKind::SubSet
                | TokenKind::MulSet
                | TokenKind::DivSet
                | TokenKind::ModSet => {
                    let oper = set_oper(&token.kind).expect("matched set family");
                    match self.next().map(|t| t.kind.clone()) {
                        Some(TokenKind::Literal(value)) => {
                            nodes.push(Stmt::Set {
                                value: Expr::Literal(value),
                                oper,
                            });
                        }
                        Some(TokenKind::LParen) => {
                            let expr = self.parse_par_expr()?;
                            nodes.push(Stmt::Set { value: expr, oper });
                        }
                        Some(other) => {
                            let tok = &self.tokens[self.i as usize];
                            return Err(ParseErr::new(
                                tok.line,
                                tok.column,
                                &format!(
                                    "expected literal or expression after set, got {}",
                                    token_name(&other)
                                ),
                            ));
                        }
                        None => {
                            return Err(self.eof_err("expected literal or expression after set"));
                        }
                    }
                }

                TokenKind::Rigged => {
                    let expr = self.parse_rigged(token.line, token.column)?;
                    nodes.push(Stmt::Expr(expr));
                }

                TokenKind::Bald | TokenKind::Doug { .. } => {
                    let node = self.parse_node(&token.kind);
                    nodes.push(node);
                }

                TokenKind::Loop => {
                    let token2 = self.next();
                    match token2 {
                        Some(t) if matches!(t.kind, TokenKind::LSquare) => {}
                        Some(t) => {
                            return Err(ParseErr::new(
                                t.line,
                                t.column,
                                &format!("expected [, got {}", token_name(&t.kind)),
                            ));
                        }
                        None => return Err(self.eof_err("expected [, got end of input")),
                    }
                    let body = self.process(false)?;
                    nodes.push(Stmt::Loop { body });
                }

                TokenKind::Prediction => {
                    let condition = self.parse_cond()?;

                    let lbrack = self.next();
                    match lbrack {
                        Some(t) if matches!(t.kind, TokenKind::LSquare) => {}
                        Some(t) => {
                            return Err(ParseErr::new(
                                t.line,
                                t.column,
                                &format!("expected [, got {}", token_name(&t.kind)),
                            ));
                        }
                        None => return Err(self.eof_err("expected [, got end of input")),
                    }

                    let token_kind1 = match self.next() {
                        Some(t) if matches!(t.kind, TokenKind::Doubters | TokenKind::Believers) => {
                            t.kind.clone()
                        }
                        Some(t) => {
                            return Err(ParseErr::new(
                                t.line,
                                t.column,
                                "expected BelieversToken or DoubtersToken at start of prediction block",
                            ));
                        }
                        None => return Err(self.eof_err(
                            "expected BelieversToken or DoubtersToken at start of prediction block",
                        )),
                    };

                    match self.next() {
                        Some(t) if matches!(t.kind, TokenKind::Win) => {}
                        Some(t) => {
                            return Err(ParseErr::new(
                                t.line,
                                t.column,
                                &format!("expected win, got {}", token_name(&t.kind)),
                            ));
                        }
                        None => return Err(self.eof_err("expected win, got end of input")),
                    }

                    match self.next() {
                        Some(t) if matches!(t.kind, TokenKind::LSquare) => {}
                        Some(t) => {
                            return Err(ParseErr::new(
                                t.line,
                                t.column,
                                &format!("expected [, got {}", token_name(&t.kind)),
                            ));
                        }
                        None => return Err(self.eof_err("expected [, got end of input")),
                    }

                    let body1 = self.process(false)?;

                    let doubt1 = matches!(token_kind1, TokenKind::Doubters);
                    let mut doubt_body: Vec<Stmt> = if doubt1 { body1.clone() } else { Vec::new() };
                    let mut believe_body: Vec<Stmt> = if !doubt1 { body1 } else { Vec::new() };

                    let is_next_token = matches!(
                        self.check_next().map(|t| &t.kind),
                        Some(TokenKind::Doubters) | Some(TokenKind::Believers)
                    );
                    if is_next_token {
                        let token_kind2 = self.next().unwrap().kind.clone();

                        if std::mem::discriminant(&token_kind2)
                            == std::mem::discriminant(&token_kind1)
                        {
                            let tok = &self.tokens[self.i as usize];
                            return Err(ParseErr::new(
                                tok.line,
                                tok.column,
                                "second branch must be opposite of first in prediction",
                            ));
                        }

                        match self.next() {
                            Some(t) if matches!(t.kind, TokenKind::Win) => {}
                            Some(t) => {
                                return Err(ParseErr::new(
                                    t.line,
                                    t.column,
                                    &format!("expected win, got {}", token_name(&t.kind)),
                                ));
                            }
                            None => return Err(self.eof_err("expected win, got end of input")),
                        }

                        match self.next() {
                            Some(t) if matches!(t.kind, TokenKind::LSquare) => {}
                            Some(t) => {
                                return Err(ParseErr::new(
                                    t.line,
                                    t.column,
                                    &format!("expected [, got {}", token_name(&t.kind)),
                                ));
                            }
                            None => return Err(self.eof_err("expected [, got end of input")),
                        }

                        let body2 = self.process(false)?;

                        if matches!(token_kind2, TokenKind::Believers) {
                            believe_body = body2;
                        } else {
                            doubt_body = body2;
                        }
                    }

                    match self.next() {
                        Some(t) if matches!(t.kind, TokenKind::RSquare) => {}
                        Some(t) => {
                            return Err(ParseErr::new(
                                t.line,
                                t.column,
                                "expected ] to close prediction block",
                            ));
                        }
                        None => return Err(self.eof_err("expected ] to close prediction block")),
                    }

                    nodes.push(Stmt::Prediction {
                        believe_body,
                        doubt_body,
                        condition,
                    });
                }

                TokenKind::Guod => {
                    nodes.push(Stmt::Guod);
                }

                TokenKind::RSquare => {
                    if is_top {
                        return Err(ParseErr::new(token.line, token.column, "unexpected ]"));
                    }
                    return Ok(nodes);
                }

                other => {
                    return Err(ParseErr::new(
                        token.line,
                        token.column,
                        &format!("unexpected {}", token_name(other)),
                    ));
                }
            }
        }

        if !is_top {
            return Err(self.eof_err("expected ] to close block"));
        }

        Ok(nodes)
    }

    fn parse_node(&mut self, kind: &TokenKind) -> Stmt {
        let reset = matches!(kind, TokenKind::Bald);
        let mut chains: Vec<DougChain> = if let TokenKind::Doug { count } = kind {
            vec![DougChain { count: *count }]
        } else {
            Vec::new()
        };

        while matches!(
            self.check_next().map(|t| &t.kind),
            Some(TokenKind::Doug { .. })
        ) {
            if let Some(TokenKind::Doug { count }) = self.next().map(|t| &t.kind) {
                chains.push(DougChain { count: *count });
            }
        }

        Stmt::Doug { chains, reset }
    }

    fn parse_doug_seq(&mut self, first_count: usize) -> Expr {
        let mut chains = vec![DougChain { count: first_count }];
        while matches!(
            self.check_next().map(|t| &t.kind),
            Some(TokenKind::Doug { .. })
        ) {
            if let Some(TokenKind::Doug { count }) = self.next().map(|t| &t.kind) {
                chains.push(DougChain { count: *count });
            }
        }
        Expr::DougSequence { chains }
    }

    fn parse_rigged(&mut self, line: usize, column: usize) -> Result<Expr, ParseErr> {
        let func_name = match self.next() {
            Some(t) => match &t.kind {
                TokenKind::Literal(ValueLiteral::Str(s)) => s.clone(),
                other => {
                    return Err(ParseErr::new(
                        t.line,
                        t.column,
                        &format!("expected function after Rigged, got {}", token_name(other)),
                    ));
                }
            },
            None => {
                return Err(ParseErr::new(
                    line,
                    column,
                    "expected function after Rigged, got end of input",
                ));
            }
        };

        let mut args: Vec<Expr> = Vec::new();
        loop {
            match self.check_next().map(|t| &t.kind) {
                Some(TokenKind::LParen) => {
                    self.next();
                    args.push(self.parse_par_expr()?);
                }
                Some(TokenKind::Literal(value)) => {
                    args.push(Expr::Literal(value.clone()));
                    self.next();
                }
                Some(TokenKind::Doug { count }) => {
                    let count = *count;
                    self.next();
                    args.push(self.parse_doug_seq(count));
                }
                Some(TokenKind::RParen) => {
                    return Ok(Expr::Rigged {
                        func: func_name,
                        args,
                    });
                }
                Some(TokenKind::RSquare)
                | Some(TokenKind::Tts)
                | Some(TokenKind::Ttss)
                | Some(TokenKind::Set)
                | Some(TokenKind::AddSet)
                | Some(TokenKind::SubSet)
                | Some(TokenKind::MulSet)
                | Some(TokenKind::DivSet)
                | Some(TokenKind::ModSet)
                | Some(TokenKind::Loop)
                | Some(TokenKind::Guod)
                | Some(TokenKind::Prediction)
                | Some(TokenKind::Believers)
                | Some(TokenKind::Doubters)
                | Some(TokenKind::Win)
                | Some(TokenKind::Bald)
                | None => {
                    return Ok(Expr::Rigged {
                        func: func_name,
                        args,
                    });
                }
                Some(other) => {
                    let tok = self.check_next().unwrap();
                    return Err(ParseErr::new(
                        tok.line,
                        tok.column,
                        &format!("expected Rigged argument, got {}", token_name(other)),
                    ));
                }
            }
        }
    }

    fn parse_par_expr(&mut self) -> Result<Expr, ParseErr> {
        let expr = self.parse_expr()?;
        match self.next() {
            Some(t) if matches!(t.kind, TokenKind::RParen) => Ok(expr),
            Some(t) => Err(ParseErr::new(
                t.line,
                t.column,
                &format!("expected ), got {}", token_name(&t.kind)),
            )),
            None => Err(self.eof_err("expected ), got end of input")),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, ParseErr> {
        match self.next() {
            Some(t) => match &t.kind {
                TokenKind::Rigged => self.parse_rigged(t.line, t.column),
                TokenKind::Doug { count } => Ok(self.parse_doug_seq(*count)),
                TokenKind::Literal(value) => Ok(Expr::Literal(value.clone())),
                other => Err(ParseErr::new(
                    t.line,
                    t.column,
                    &format!("expected expression, got {}", token_name(other)),
                )),
            },
            None => Err(self.eof_err("expected expression, got end of input")),
        }
    }

    fn parse_cond(&mut self) -> Result<Condition, ParseErr> {
        let left = match self.next() {
            Some(t) => match &t.kind {
                TokenKind::LParen => self.parse_par_expr()?,
                TokenKind::Literal(value) => Expr::Literal(value.clone()),
                other => {
                    return Err(ParseErr::new(
                        t.line,
                        t.column,
                        &format!("expected expression, got {}", token_name(other)),
                    ));
                }
            },
            None => return Err(self.eof_err("expected expression, got end of input")),
        };

        let oper = match self.next() {
            Some(t) => match comp_oper(&t.kind) {
                Some(oper) => oper,
                None => {
                    return Err(ParseErr::new(
                        t.line,
                        t.column,
                        &format!("expected comparison operator, got {}", token_name(&t.kind)),
                    ));
                }
            },
            None => return Err(self.eof_err("expected comparison operator, got end of input")),
        };

        let right = match self.next() {
            Some(t) => match &t.kind {
                TokenKind::LParen => self.parse_par_expr()?,
                TokenKind::Literal(value) => Expr::Literal(value.clone()),
                other => {
                    return Err(ParseErr::new(
                        t.line,
                        t.column,
                        &format!("expected expression, got {}", token_name(other)),
                    ));
                }
            },
            None => return Err(self.eof_err("expected expression, got end of input")),
        };

        Ok(Condition { left, oper, right })
    }
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Stmt>, ParseErr> {
    Parser::new(tokens).parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer;

    fn parse_src(source: &str) -> Result<Vec<Stmt>, ParseErr> {
        parse(&lexer::lex(source).unwrap())
    }

    #[test]
    fn invalid_set_is_rejected() {
        let err = parse_src("set ]").unwrap_err();
        assert!(
            err.message
                .contains("expected literal or expression after set")
        );
    }

    #[test]
    fn unexpected_top_level_token_is_rejected() {
        let err = parse_src("(").unwrap_err();
        assert!(err.message.contains("unexpected"));
    }

    #[test]
    fn top_level_rigged_is_statement() {
        let ast = parse_src("Rigged \"pow\" 2 3").unwrap();
        assert!(matches!(ast[0], Stmt::Expr(Expr::Rigged { .. })));
    }

    #[test]
    fn rigged_accepts_doug_sequences_as_args() {
        let ast = parse_src("set (Rigged \"id\" Doug DougDoug)").unwrap();
        match &ast[0] {
            Stmt::Set {
                value: Expr::Rigged { args, .. },
                ..
            } => {
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Expr::DougSequence { .. }));
            }
            other => panic!("unexpected AST: {other:?}"),
        }
    }
}
