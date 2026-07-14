mod error;
mod ffi;
mod value;

pub use error::RuntimeError;
pub use value::Value;

use crate::ast::{BoolOper, Condition, DougChain, Expr, SetOper, Stmt};
use crate::token::ValueLiteral;
use crate::tts::Tts;
use std::sync::Arc;

pub struct Interpreter {
    val_left: Vec<Value>,
    val_right: Vec<Value>,
    val_i: i64,
    tts: Arc<Tts>,
    linked_libs: Vec<String>,
}

impl Interpreter {
    pub fn new(tts: Arc<Tts>, linked_libs: Vec<String>) -> Self {
        Interpreter {
            val_left: Vec::new(),
            val_right: vec![Value::Int(0)],
            val_i: 0,
            tts,
            linked_libs,
        }
    }

    fn get_val(&self, i: i64) -> Value {
        if i < 0 {
            let idx = (-i - 1) as usize;
            self.val_left.get(idx).cloned().unwrap_or(Value::Int(0))
        } else {
            let idx = i as usize;
            self.val_right.get(idx).cloned().unwrap_or(Value::Int(0))
        }
    }

    fn set_val(&mut self, i: i64, value: Value) -> Result<(), RuntimeError> {
        if i < 0 {
            let idx = (-i - 1) as usize;
            let len = self.val_left.len();
            if idx > len {
                return Err(RuntimeError::new(&format!(
                    "You are literally trolling. Do you know how to count? {idx} before {len}?"
                )));
            } else if idx == len {
                self.val_left.push(value);
            } else {
                self.val_left[idx] = value;
            }
        } else {
            let idx = i as usize;
            let len = self.val_right.len();
            if idx > len {
                return Err(RuntimeError::new(&format!(
                    "You are literally trolling. Do you know how to count? {idx} before {len}?"
                )));
            } else if idx == len {
                self.val_right.push(value);
            } else {
                self.val_right[idx] = value;
            }
        }
        Ok(())
    }

    fn get_index(&self, chains: &[DougChain], start_i: i64) -> i64 {
        let mut res_i = start_i;
        for (i, chain) in chains.iter().enumerate() {
            let value: i64 = 1 << (chain.count - 1);
            if i % 2 == 0 {
                res_i += value;
            } else {
                res_i -= value;
            }
        }
        res_i
    }

    fn eval_expr(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit) => Ok(match lit {
                ValueLiteral::Int(v) => Value::Int(*v),
                ValueLiteral::Float(v) => Value::Float(*v),
                ValueLiteral::Str(v) => Value::Str(v.clone()),
            }),
            Expr::DougSequence { chains } => {
                let idx = self.get_index(chains, 0);
                Ok(self.get_val(idx))
            }
            Expr::Rigged { func: func_name, args } => {
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

    fn process(&mut self, nodes: &[Stmt]) -> Result<(), RuntimeError> {
        for node in nodes {
            match node {
                Stmt::Set { value, oper } => {
                    let rhs = self.eval_expr(value)?;
                    match oper {
                        SetOper::Set => {
                            self.set_val(self.val_i, rhs)?;
                        }
                        SetOper::Add => {
                            let left = self.get_val(self.val_i);
                            let result = if left.is_string() || rhs.is_string() {
                                Value::Str(format!("{left}{rhs}"))
                            } else {
                                match (&left, &rhs) {
                                    (Value::Float(_), _) | (_, Value::Float(_)) => {
                                        Value::Float(left.as_f64() + rhs.as_f64())
                                    }
                                    (Value::Int(a), Value::Int(b)) => Value::Int(a + b),
                                    _ => Value::Float(left.as_f64() + rhs.as_f64()),
                                }
                            };
                            self.set_val(self.val_i, result)?;
                        }
                        SetOper::Sub => {
                            let left = self.get_val(self.val_i);
                            let result = match (&left, &rhs) {
                                (Value::Float(_), _) | (_, Value::Float(_)) => {
                                    Value::Float(left.as_f64() - rhs.as_f64())
                                }
                                (Value::Int(a), Value::Int(b)) => Value::Int(a - b),
                                _ => Value::Float(left.as_f64() - rhs.as_f64()),
                            };
                            self.set_val(self.val_i, result)?;
                        }
                        SetOper::Mul => {
                            let left = self.get_val(self.val_i);
                            let result = match (&left, &rhs) {
                                (Value::Float(_), _) | (_, Value::Float(_)) => {
                                    Value::Float(left.as_f64() * rhs.as_f64())
                                }
                                (Value::Int(a), Value::Int(b)) => Value::Int(a * b),
                                _ => Value::Float(left.as_f64() * rhs.as_f64()),
                            };
                            self.set_val(self.val_i, result)?;
                        }
                        SetOper::Div => {
                            let left = self.get_val(self.val_i);
                            let result = Value::Float(left.as_f64() / rhs.as_f64());
                            self.set_val(self.val_i, result)?;
                        }
                        SetOper::Mod => {
                            let left = self.get_val(self.val_i);
                            let result = match (&left, &rhs) {
                                (Value::Int(a), Value::Int(b)) => Value::Int(a % b),
                                _ => Value::Float(left.as_f64() % rhs.as_f64()),
                            };
                            self.set_val(self.val_i, result)?;
                        }
                    }
                }

                Stmt::Tts {
                    msg,
                    use_index,
                    overlap,
                } => {
                    let text = if *use_index {
                        format!("{}", self.get_val(self.val_i))
                    } else {
                        match msg {
                            Some(expr) => format!("{}", self.eval_expr(expr)?),
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
                    let start = if *reset { 0 } else { self.val_i };
                    self.val_i = self.get_index(chains, start);
                }

                Stmt::Loop { body } => loop {
                    match self.process(body) {
                        Ok(()) => {}
                        Err(e) => {
                            if e.message == "__break__" {
                                break;
                            }
                            return Err(e);
                        }
                    }
                },

                Stmt::Goud => {
                    return Err(RuntimeError {
                        message: "__break__".to_string(),
                    });
                }

                Stmt::Prediction {
                    believe_body,
                    doubt_body,
                    condition,
                } => {
                    if self.eval_cond(condition)? {
                        self.process(believe_body)?;
                    } else {
                        self.process(doubt_body)?;
                    }
                }
            }
        }
        Ok(())
    }

    pub fn run(&mut self, nodes: &[Stmt]) -> Result<(), RuntimeError> {
        self.process(nodes)
    }
}
