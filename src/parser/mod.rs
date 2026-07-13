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
                            let chains = self.parse_expr()?;
                            nodes.push(Stmt::Tts {
                                msg: Some(Expr::DougSequence { chains }),
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
                    match self.next().map(|t| (&t.kind).clone()) {
                        Some(TokenKind::Literal(value)) => {
                            nodes.push(Stmt::Set {
                                value: Expr::Literal(value),
                                oper,
                            });
                        }
                        Some(TokenKind::LParen) => {
                            let chains = self.parse_expr()?;
                            nodes.push(Stmt::Set {
                                value: Expr::DougSequence { chains },
                                oper,
                            });
                        }
                        _ => {}
                    }
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
                        None => {
                            return Err(ParseErr::new(
                                self.i as usize,
                                0,
                                "expected [, got end of input",
                            ));
                        }
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
                        None => {
                            return Err(ParseErr::new(
                                self.i as usize,
                                0,
                                "expected [, got end of input",
                            ));
                        }
                    }

                    let token_kind1 = match self.next() {
                        Some(t)
                            if matches!(
                                t.kind,
                                TokenKind::Doubters | TokenKind::Believers
                            ) =>
                        {
                            t.kind.clone()
                        }
                        Some(t) => {
                            return Err(ParseErr::new(
                                t.line,
                                t.column,
                                "expected BelieversToken or DoubtersToken at start of prediction block",
                            ));
                        }
                        None => {
                            return Err(ParseErr::new(
                                self.i as usize,
                                0,
                                "expected BelieversToken or DoubtersToken at start of prediction block",
                            ));
                        }
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
                        None => {
                            return Err(ParseErr::new(
                                self.i as usize,
                                0,
                                "expected win, got end of input",
                            ));
                        }
                    }

                    let body1 = self.process(false)?;

                    let doubt1 = matches!(token_kind1, TokenKind::Doubters);
                    let mut doubt_body: Vec<Stmt> =
                        if doubt1 { body1.clone() } else { Vec::new() };
                    let mut believe_body: Vec<Stmt> =
                        if !doubt1 { body1 } else { Vec::new() };

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
                            None => {
                                return Err(ParseErr::new(
                                    self.i as usize,
                                    0,
                                    "expected win, got end of input",
                                ));
                            }
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
                        None => {
                            return Err(ParseErr::new(
                                self.i as usize,
                                0,
                                "expected ] to close prediction block",
                            ));
                        }
                    }

                    nodes.push(Stmt::Prediction {
                        believe_body,
                        doubt_body,
                        condition,
                    });
                }

                TokenKind::Goud => {
                    nodes.push(Stmt::Goud);
                }

                TokenKind::Rigged => {
                    let func_name = match self.next() {
                        Some(t) => match &t.kind {
                            TokenKind::Literal(ValueLiteral::Str(s)) => s.clone(),
                            other => {
                                return Err(ParseErr::new(
                                    t.line,
                                    t.column,
                                    &format!(
                                        "expected function after Rigged, got {}",
                                        token_name(other)
                                    ),
                                ));
                            }
                        },
                        None => {
                            return Err(ParseErr::new(
                                self.i as usize,
                                0,
                                "expected function after Rigged, got end of input",
                            ));
                        }
                    };

                    let mut args: Vec<Expr> = Vec::new();
                    loop {
                        match self.check_next().map(|t| &t.kind) {
                            Some(TokenKind::LParen) => {
                                self.next();
                                let chains = self.parse_expr()?;
                                args.push(Expr::DougSequence { chains });
                            }
                            Some(TokenKind::Literal(value)) => {
                                let value = value.clone();
                                self.next();
                                args.push(Expr::Literal(value));
                            }
                            Some(TokenKind::Doug { count }) => {
                                let count = *count;
                                self.next();
                                args.push(Expr::DougSequence {
                                    chains: vec![DougChain { count }],
                                });
                            }
                            _ => break,
                        }
                    }
                    nodes.push(Stmt::Rigged { func: func_name, args });
                }

                TokenKind::RSquare => {
                    if is_top {
                        return Err(ParseErr::new(token.line, token.column, "unexpected ]"));
                    }
                    return Ok(nodes);
                }

                _ => {}
            }
        }

        if !is_top {
            return Err(ParseErr::new(self.i as usize, 0, "expected ] to close block"));
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

        while matches!(self.check_next().map(|t| &t.kind), Some(TokenKind::Doug { .. })) {
            if let Some(TokenKind::Doug { count }) = self.next().map(|t| &t.kind) {
                chains.push(DougChain { count: *count });
            }
        }

        Stmt::Doug { chains, reset }
    }

    fn parse_expr(&mut self) -> Result<Vec<DougChain>, ParseErr> {
        let mut dougs: Vec<DougChain> = Vec::new();
        while let Some(token) = self.next() {
            match &token.kind {
                TokenKind::Doug { count } => dougs.push(DougChain { count: *count }),
                TokenKind::RParen => return Ok(dougs),
                other => {
                    return Err(ParseErr::new(
                        token.line,
                        token.column,
                        &format!("expected DougToken or ), got {}", token_name(other)),
                    ));
                }
            }
        }
        Err(ParseErr::new(
            self.i as usize,
            0,
            "expected DougToken or ), got end of input",
        ))
    }

    fn parse_cond(&mut self) -> Result<Condition, ParseErr> {
        let left = match self.next() {
            Some(t) => match &t.kind {
                TokenKind::LParen => {
                    let chains = self.parse_expr()?;
                    Expr::DougSequence { chains }
                }
                TokenKind::Literal(value) => Expr::Literal(value.clone()),
                other => {
                    return Err(ParseErr::new(
                        t.line,
                        t.column,
                        &format!("expected expression, got {}", token_name(other)),
                    ));
                }
            },
            None => {
                return Err(ParseErr::new(
                    self.i as usize,
                    0,
                    "expected expression, got end of input",
                ));
            }
        };

        let oper = match self.next() {
            Some(t) => match comp_oper(&t.kind) {
                Some(oper) => oper,
                None => {
                    return Err(ParseErr::new(
                        t.line,
                        t.column,
                        &format!(
                            "expected comparison operator, got {}",
                            token_name(&t.kind)
                        ),
                    ));
                }
            },
            None => {
                return Err(ParseErr::new(
                    self.i as usize,
                    0,
                    "expected comparison operator, got end of input",
                ));
            }
        };

        let right = match self.next() {
            Some(t) => match &t.kind {
                TokenKind::LParen => {
                    let chains = self.parse_expr()?;
                    Expr::DougSequence { chains }
                }
                TokenKind::Literal(value) => Expr::Literal(value.clone()),
                other => {
                    return Err(ParseErr::new(
                        t.line,
                        t.column,
                        &format!("expected expression, got {}", token_name(other)),
                    ));
                }
            },
            None => {
                return Err(ParseErr::new(
                    self.i as usize,
                    0,
                    "expected expression, got end of input",
                ));
            }
        };

        Ok(Condition { left, oper, right })
    }
}

pub fn parse(tokens: &[Token]) -> Result<Vec<Stmt>, ParseErr> {
    Parser::new(tokens).parse()
}
