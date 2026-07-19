mod ffi;

use crate::parser::Parser;
use crate::parser::ast::{DougChain, Expr, Reference, Stmt};
use crate::runtime::RuntimeError;
use crate::tts::Tts;
use crate::values::tape::{Mutator, MutatorView, RuntimeTape};
use crate::values::value::Function;
use crate::values::{BuildFxHasher, FxHasher, Value, hash_function};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, PartialEq)]
enum Flow {
    Continue,
    Return(Value),
    Break,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TapeSelection {
    Scoped,
    Main,
}

pub struct Interpreter<'a> {
    full_tape: RuntimeTape,
    scoped_tape: Option<RuntimeTape>,
    tts: Arc<Tts>,
    linked_libs: Vec<String>,

    parser: Parser<'a>,
    hasher: FxHasher,
    adventure_names: HashMap<String, i32, BuildFxHasher>,
    active_tape: TapeSelection,
}

impl<'a> Interpreter<'a> {
    pub fn new(tts: Arc<Tts>, linked_libs: Vec<String>, parser: Parser<'a>) -> Self {
        let hasher = FxHasher::new();

        Interpreter {
            full_tape: RuntimeTape::new(),
            scoped_tape: None,
            tts,
            linked_libs,
            hasher,
            parser,
            adventure_names: HashMap::with_hasher(BuildFxHasher {}),
            active_tape: TapeSelection::Main,
        }
    }

    fn current_tape(&self) -> &RuntimeTape {
        match self.active_tape {
            TapeSelection::Scoped => self.scoped_tape.as_ref().unwrap_or(&self.full_tape),
            TapeSelection::Main => &self.full_tape,
        }
    }

    fn current_tape_mut(&mut self) -> &mut RuntimeTape {
        match self.active_tape {
            TapeSelection::Scoped => self.scoped_tape.as_mut().unwrap_or(&mut self.full_tape),
            TapeSelection::Main => &mut self.full_tape,
        }
    }

    fn main_tape(&self) -> &RuntimeTape {
        &self.full_tape
    }

    fn main_tape_mut(&mut self) -> &mut RuntimeTape {
        &mut self.full_tape
    }

    fn get_doug_notation_index(
        &self,
        chains: &[Reference],
        start_i: i32,
    ) -> Result<(i32, bool), RuntimeError> {
        let mut res_i = start_i;

        let (starting_value, from_negative) = match chains.first() {
            None => return Err(RuntimeError::SegmentationFault),
            Some(Reference::Variable(name)) => (
                *self
                    .adventure_names
                    .get(name)
                    .ok_or(RuntimeError::NotDefined(name.clone()))?,
                false,
            ),
            Some(Reference::Doug(DougChain { count })) => (count.cast_signed() as i32, true),
        };

        res_i += starting_value;

        for (i, chain) in chains.iter().skip(1).enumerate() {
            let Reference::Doug(chain) = chain else {
                return Err(RuntimeError::Unexpected("variable name".to_string()));
            };
            let value = 1 << (chain.count - 1);

            if i % 2 == if from_negative { 0 } else { 1 } {
                res_i += value;
            } else {
                res_i -= value;
            }
        }

        Ok((res_i, !from_negative))
    }

    fn resolve_dougs(
        &self,
        chains: &[Reference],
        start_i: i32,
    ) -> Result<(i32, &RuntimeTape), RuntimeError> {
        let (idx, func_call) = self.get_doug_notation_index(chains.into(), start_i)?;
        if func_call {
            Ok((idx, self.main_tape()))
        } else {
            Ok((idx, self.current_tape()))
        }
    }

    fn resolve_dougs_mut(
        &mut self,
        chains: &[Reference],
        start_i: i32,
    ) -> Result<(i32, &mut RuntimeTape), RuntimeError> {
        let (idx, func_call) = self.get_doug_notation_index(chains.into(), start_i)?;
        if func_call {
            Ok((idx, self.main_tape_mut()))
        } else {
            Ok((idx, self.current_tape_mut()))
        }
    }

    fn run_function(&mut self, idx: i32, guard: &MutatorView) -> Result<Value, RuntimeError> {
        let Value::Fmca(block) = self.full_tape.get(idx, guard)? else {
            return Err(RuntimeError::NotAFunction);
        };

        let old_tape = self.current_tape().clone();
        let new_tape = old_tape.clone_into(idx, 16);
        *self.main_tape_mut() = new_tape;
        let v = match self.process(block.get_nodes(), guard) {
            Ok(Flow::Return(v)) => v,
            Ok(_) => Value::Nil,
            Err(e) => return Err(e),
        };
        *self.main_tape_mut() = old_tape;
        Ok(v)
    }

    fn eval_expr(&mut self, expr: &Expr, mem: &MutatorView) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Literal(lit) => Ok(lit.get(mem).get_value().into()),
            Expr::Variable(var) => {
                let idx = *self
                    .adventure_names
                    .get(var)
                    .ok_or_else(|| RuntimeError::new(&format!("unknown adventure {var}")))?;
                self.main_tape().get(idx, mem)
            }
            Expr::DougSequence { chains } => {
                let (idx, tape) = self.resolve_dougs(chains, 0)?;
                tape.get(idx, mem)
            }
            Expr::MainTapeDougSequence { chains } => {
                let (idx, tape) = self.resolve_dougs(chains, 0)?;
                tape.get(idx, mem)
            }
            Expr::FmcaCall { name } => {
                let idx = if let Some(name) = name {
                    self.adventure_names
                        .get(name)
                        .ok_or(RuntimeError::NotDefined(name.clone()))?
                } else {
                    &self.main_tape().cursor
                };
                self.run_function(*idx, mem)
            }
            Expr::Rigged {
                func: func_name,
                args,
            } => {
                let arg_val: Vec<Value> = args
                    .iter()
                    .map(|a| self.eval_expr(a, mem))
                    .collect::<Result<_, _>>()?;
                ffi::ffi(func_name, &arg_val, &self.linked_libs)
            }
            Expr::Condition {
                left,
                operator,
                right,
            } => {
                if let Some(operator) = operator
                    && let Some(rhs) = right
                {
                    Ok(Value::apply_operator(
                        self.eval_expr(left, mem)?,
                        *operator,
                        self.eval_expr(rhs, mem)?,
                    ))
                } else {
                    self.eval_expr(left, mem)
                }
            }
        }
    }

    fn process(&mut self, nodes: &[Stmt], guard: &MutatorView) -> Result<Flow, RuntimeError> {
        for node in nodes {
            match node {
                Stmt::Set { value, oper } => match oper {
                    None => {
                        let value = self.eval_expr(value, guard)?;
                        let tape = self.current_tape_mut();
                        tape.set_value(guard, value)?
                    }
                    Some(op) => {
                        let l = self.current_tape().get_current(guard)?;
                        let v = Value::apply_operator(l, *op, self.eval_expr(value, guard)?);
                        self.current_tape_mut().set_value(guard, v)?;
                    }
                },

                Stmt::Expr(expr) => {
                    self.eval_expr(expr, guard)?;
                }

                Stmt::Tts {
                    msg,
                    use_index,
                    overlap,
                } => {
                    let text = if *use_index {
                        self.current_tape().get_current(guard)?.to_string()
                    } else {
                        match msg {
                            Some(expr) => self.eval_expr(expr, guard)?.to_string(),
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
                    let (idx, tape) = self.resolve_dougs_mut(chains, if *reset { 0 } else { 1 })?;
                    tape.set_cursor(idx);
                }

                Stmt::Loop { body } => loop {
                    if self.process(body, guard)? == Flow::Break {
                        break;
                    }
                },

                Stmt::Guod { value, use_index } => {
                    let v = if *use_index {
                        self.current_tape().get_current(guard)
                    } else {
                        self.eval_expr(value.as_ref().unwrap(), guard)
                    }
                    .unwrap_or(Value::Nil);
                    return Ok(Flow::Return(v));
                }
                Stmt::Break => {
                    return Ok(Flow::Break);
                }

                Stmt::Prediction {
                    believe_body,
                    doubt_body,
                    condition,
                } => {
                    let flow = if self.eval_expr(condition, guard)?.into() {
                        self.process(believe_body, guard)?
                    } else {
                        self.process(doubt_body, guard)?
                    };
                    if flow == Flow::Break {
                        return Ok(Flow::Break);
                    }
                }

                Stmt::FiveMinuteCodingAdventure { name, body } => {
                    let function = Function::new(body.clone());
                    let index = hash_function(&function, &mut self.hasher);

                    self.adventure_names.insert(name.clone(), index);
                    self.current_tape_mut()
                        .set_value(guard, Value::Fmca(function));
                }

                Stmt::Call { name, use_index } => {
                    assert!(name.is_none() && !*use_index);
                    let idx = if *use_index {
                        &self.main_tape().cursor
                    } else {
                        self.adventure_names
                            .get(name.as_ref().unwrap())
                            .ok_or(RuntimeError::NotDefined(name.as_ref().unwrap().clone()))?
                    };

                    self.run_function(*idx, guard);
                }
            }
        }
        Ok(Flow::Continue)
    }
}

impl<'a> Mutator<'a> for Interpreter<'a> {
    type Input = String;
    type Output = ();
    type Scope = MutatorView<'a>;

    fn run(
        &mut self,
        mem: &'a Self::Scope,
        input: Self::Input,
    ) -> Result<Self::Output, RuntimeError> {
        match self.parser.run(mem.get_data(), input) {
            Ok(nodes) => {
                self.process(&nodes, mem)?;
            }
            Err(e) => {
                eprintln!("Error: {e}");
            }
        }

        Ok(())
    }
}
