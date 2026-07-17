mod ffi;

#[cfg(test)]
pub use crate::runtime::RuntimeErrorKind;
pub use crate::runtime::{RuntimeError, Value};

use crate::ast::{BoolOper, Condition, Expr, SetOper, Stmt};
use crate::runtime::{self, RuntimeTape};
use crate::token::ValueLiteral;
use crate::tts::Tts;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Continue,
    Break,
}

pub struct Interpreter {
    tape: RuntimeTape,
    tts: Arc<Tts>,
    linked_libs: Vec<String>,
}

impl Interpreter {
    pub fn new(tts: Arc<Tts>, linked_libs: Vec<String>) -> Self {
        Interpreter {
            tape: RuntimeTape::new(),
            tts,
            linked_libs,
        }
    }

    fn eval_expr(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                ValueLiteral::Int(v) => Value::Int(*v),
                ValueLiteral::Float(v) => Value::Float(*v),
                ValueLiteral::Str(v) => Value::Str(v.clone()),
            }),
            Expr::DougSequence { chains } => {
                let idx = runtime::doug_index(chains, 0)?;
                Ok(self.tape.get(idx))
            }
            Expr::Rigged {
                func: func_name,
                args,
            } => {
                let arg_val: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_expr(a))
                    .collect::<Result<_, _>>()?;
                ffi::ffi(func_name, &arg_val, &self.linked_libs)
            }
        }
    }

    fn eval_cond(&self, condition: &Condition) -> Result<bool, RuntimeError> {
        let left = self.eval_expr(&condition.left)?;
        let right = self.eval_expr(&condition.right)?;

        let result = match condition.oper {
            BoolOper::Equal => left.as_f64() == right.as_f64(),
            BoolOper::NotEqual => left.as_f64() != right.as_f64(),
            BoolOper::Less => left.as_f64() < right.as_f64(),
            BoolOper::LessEqual => left.as_f64() <= right.as_f64(),
            BoolOper::Greater => left.as_f64() > right.as_f64(),
            BoolOper::GreaterEqual => left.as_f64() >= right.as_f64(),
        };
        Ok(result)
    }

    fn process(&mut self, nodes: &[Stmt]) -> Result<Flow, RuntimeError> {
        for node in nodes {
            match node {
                Stmt::Set { value, oper } => {
                    let rhs = self.eval_expr(value)?;
                    let result = match oper {
                        SetOper::Set => rhs,
                        SetOper::Add => runtime::add(&self.tape.current(), &rhs),
                        SetOper::Sub => runtime::sub(&self.tape.current(), &rhs),
                        SetOper::Mul => runtime::mul(&self.tape.current(), &rhs),
                        SetOper::Div => runtime::div(&self.tape.current(), &rhs)?,
                        SetOper::Mod => runtime::modulo(&self.tape.current(), &rhs)?,
                    };
                    self.tape.set_current(result)?;
                }

                Stmt::Expr(expr) => {
                    self.eval_expr(expr)?;
                }

                Stmt::Tts {
                    msg,
                    use_index,
                    overlap,
                } => {
                    let text = if *use_index {
                        self.tape.current().to_string()
                    } else {
                        match msg {
                            Some(expr) => self.eval_expr(expr)?.to_string(),
                            None => String::new(),
                        }
                    };
                    if *overlap {
                        self.tts.speak_overlap(&text);
                    } else {
                        self.tts.wait();
                        self.tts.speak(&text);
                    }
                }

                Stmt::Doug { chains, reset } => {
                    self.tape.move_doug(chains, *reset)?;
                }

                Stmt::Loop { body } => loop {
                    if self.process(body)? == Flow::Break {
                        break;
                    }
                },

                Stmt::Guod => return Ok(Flow::Break),

                Stmt::Prediction {
                    believe_body,
                    doubt_body,
                    condition,
                } => {
                    let flow = if self.eval_cond(condition)? {
                        self.process(believe_body)?
                    } else {
                        self.process(doubt_body)?
                    };
                    if flow == Flow::Break {
                        return Ok(Flow::Break);
                    }
                }
            }
        }
        Ok(Flow::Continue)
    }

    pub fn run(&mut self, nodes: &[Stmt]) -> Result<(), RuntimeError> {
        match self.process(nodes)? {
            Flow::Continue => Ok(()),
            Flow::Break => Err(RuntimeError::break_outside_loop()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::DougChain;

    fn interpreter() -> Interpreter {
        Interpreter::new(Arc::new(Tts::new()), Vec::new())
    }

    #[test]
    fn top_level_break_is_user_error() {
        let mut interp = interpreter();
        let err = interp.run(&[Stmt::Guod]).unwrap_err();
        assert_eq!(err.kind, RuntimeErrorKind::BreakOutsideLoop);
    }

    #[test]
    fn division_by_zero_is_error() {
        let mut interp = interpreter();
        let program = vec![Stmt::Set {
            value: Expr::Literal(ValueLiteral::Int(0)),
            oper: SetOper::Div,
        }];
        let err = interp.run(&program).unwrap_err();
        assert_eq!(err.kind, RuntimeErrorKind::DivisionByZero);
    }

    #[test]
    fn oversized_doug_chain_is_error() {
        let mut interp = interpreter();
        let program = vec![Stmt::Doug {
            chains: vec![DougChain { count: 128 }],
            reset: false,
        }];
        let err = interp.run(&program).unwrap_err();
        assert_eq!(err.kind, RuntimeErrorKind::DougIndexOverflow { count: 128 });
    }
}
