mod ffi;

#[cfg(test)]
pub use crate::runtime::RuntimeErrorKind;
pub use crate::runtime::{RuntimeError, Value};

use crate::ast::{BoolOper, Condition, Expr, SetOper, Stmt};
use crate::runtime::{self, RuntimeTape};
use crate::token::ValueLiteral;
use crate::tts::Tts;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Flow {
    Continue,
    Break,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TapeSelection {
    Local,
    Main,
}

pub struct Interpreter {
    tape: RuntimeTape,
    main_tape: Option<RuntimeTape>,
    tts: Arc<Tts>,
    linked_libs: Vec<String>,
    adventure_names: HashMap<String, i64>,
    active_tape: TapeSelection,
    adventure_base: Option<i64>,
}

impl Interpreter {
    pub fn new(tts: Arc<Tts>, linked_libs: Vec<String>) -> Self {
        Interpreter {
            tape: RuntimeTape::new(),
            main_tape: None,
            tts,
            linked_libs,
            adventure_names: HashMap::new(),
            active_tape: TapeSelection::Local,
            adventure_base: None,
        }
    }

    fn current_tape(&self) -> &RuntimeTape {
        match self.active_tape {
            TapeSelection::Local => &self.tape,
            TapeSelection::Main => self.main_tape.as_ref().unwrap_or(&self.tape),
        }
    }

    fn current_tape_mut(&mut self) -> &mut RuntimeTape {
        match self.active_tape {
            TapeSelection::Local => &mut self.tape,
            TapeSelection::Main => self.main_tape.as_mut().unwrap_or(&mut self.tape),
        }
    }

    fn main_tape(&self) -> &RuntimeTape {
        self.main_tape.as_ref().unwrap_or(&self.tape)
    }

    fn main_tape_mut(&mut self) -> &mut RuntimeTape {
        self.main_tape.as_mut().unwrap_or(&mut self.tape)
    }

    fn eval_expr(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                ValueLiteral::Int(v) => Value::Int(*v),
                ValueLiteral::Float(v) => Value::Float(*v),
                ValueLiteral::Str(v) => Value::Str(v.clone()),
            }),
            Expr::DougSequence { chains } => {
                let idx = runtime::doug_index(chains, self.adventure_base.unwrap_or(0))?;
                Ok(self.current_tape().get(idx))
            }
            Expr::MainTapeDougSequence { chains } => {
                let idx = runtime::doug_index(chains, 0)?;
                Ok(self.main_tape().get(idx))
            }
            Expr::FmcaCall { name } => {
                let idx = *self
                    .adventure_names
                    .get(name)
                    .ok_or_else(|| RuntimeError::new(&format!("unknown adventure {name}")))?;
                Ok(self.tape.get(idx + 3))
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
                        SetOper::Add => runtime::add(&self.current_tape().current(), &rhs),
                        SetOper::Sub => runtime::sub(&self.current_tape().current(), &rhs),
                        SetOper::Mul => runtime::mul(&self.current_tape().current(), &rhs),
                        SetOper::Div => runtime::div(&self.current_tape().current(), &rhs)?,
                        SetOper::Mod => runtime::modulo(&self.current_tape().current(), &rhs)?,
                    };
                    self.current_tape_mut().set_current(result)?;
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
                        self.current_tape().current().to_string()
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
                    self.active_tape = TapeSelection::Local;
                    if *reset {
                        let idx = runtime::doug_index(chains, self.adventure_base.unwrap_or(0))?;
                        self.tape.set_index(idx);
                    } else {
                        self.tape.move_doug(chains, false)?;
                    }
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

                Stmt::FiveMinuteCodingAdventure { name, body } => {
                    self.adventure_names
                        .insert(name.clone(), self.current_tape().index());
                    self.current_tape_mut().set_current(Value::FiveMinuteCodingAdventure {
                        body: body.clone(),
                    })?;
                }

                Stmt::FmcaCall {
                    name,
                    args,
                    after_call,
                } => {
                    let idx = *self
                        .adventure_names
                        .get(name)
                        .ok_or_else(|| RuntimeError::new(&format!("unknown adventure {name}")))?;
                    let Value::FiveMinuteCodingAdventure { body } = self.tape.get(idx) else {
                        return Err(RuntimeError::new(&format!("{name} is not an adventure")));
                    };
                    let previous_index = self.tape.index();
                    let previous_base = self.adventure_base;
                    self.adventure_base = Some(idx);
                    self.tape.set_index(idx);
                    let arg_flow = self.process(args)?;
                    if arg_flow != Flow::Break {
                        self.tape.set_index(idx);
                        let _ = self.process(&body)?;
                    }
                    self.tape.set_index(idx);
                    let flow = self.process(after_call);
                    self.tape.set_index(previous_index);
                    self.adventure_base = previous_base;
                    if flow? == Flow::Break {
                        return Ok(Flow::Break);
                    }
                }

                Stmt::MainTapeDoug { chains } => {
                    self.active_tape = TapeSelection::Main;
                    self.main_tape_mut().move_doug(chains, false)?;
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
