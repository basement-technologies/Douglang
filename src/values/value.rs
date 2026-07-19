use std::{
    fmt::Display,
    ops::{Add, BitXor, Div, Mul, Rem, Sub},
};

use crate::{
    parser::ast::Stmt,
    runtime::RuntimeError,
    values::{
        Operator,
        tape::{AllocObject, LiteralList, TypeList},
    },
};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum Value {
    String(String),
    Number(f64),
    Boolean(bool),
    Err(RuntimeError),
    FiveMinuteCodingAdventure(FiveMinuteCodingAdventure),
    Nil,
}

impl Value {
    fn into_f64(self) -> f64 {
        self.into()
    }

    fn into_bool(self) -> bool {
        self.into()
    }

    #[must_use]
    pub fn apply_operator(l: Self, op: Operator, r: Self) -> Self {
        match op {
            Operator::Greater => Value::Boolean(l > r),
            Operator::Less => Value::Boolean(l < r),
            Operator::GreaterEquals => Value::Boolean(l >= r),
            Operator::LessEquals => Value::Boolean(l <= r),
            Operator::Equals => Value::Boolean(l == r),
            Operator::NotEquals => Value::Boolean(l != r),
            Operator::LogicalOr => Value::Boolean(l.into() || r.into()),
            Operator::LogicalXor => Value::Boolean(l.into_bool().bitxor(r.into_bool())),
            Operator::LogicalAnd => Value::Boolean(l.into() && r.into()),

            Operator::Plus => l + r,
            Operator::Minus => Value::Number(l - r),
            Operator::Divide => Value::Number(l / r),
            Operator::Multiply => Value::Number(l * r),
            Operator::Modulo => Value::Number(l % r),

            Operator::BinaryOr => {
                Value::Number(f64::from(l.into_f64() as i32 | r.into_f64() as i32))
            }
            Operator::BinaryXor => {
                Value::Number(f64::from(l.into_f64() as i32 ^ r.into_f64() as i32))
            }
            Operator::BinaryAnd => {
                Value::Number(f64::from(l.into_f64() as i32 & r.into_f64() as i32))
            }

            _ => Value::Err(RuntimeError::BadExpression(
                l.to_string(),
                op.to_string(),
                r.to_string(),
            )),
        }
    }
}

impl Add for Value {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => Self::Number(l + r),
            (Self::String(l), Self::Number(r)) => Self::String(l + &r.to_string()),
            (Self::String(l), Self::String(r)) => Self::String(l + &r),
            (l, r) => Self::Err(RuntimeError::BadExpression(
                l.to_string(),
                "+".to_string(),
                r.to_string(),
            )),
        }
    }
}

impl Sub for Value {
    type Output = f64;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => l - r,
            (l, r) => panic!("Invalid expression, {l} - {r}"),
        }
    }
}

impl Mul for Value {
    type Output = f64;

    fn mul(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => l * r,
            (l, r) => panic!("Invalid expression, {l} * {r}"),
        }
    }
}

impl Div for Value {
    type Output = f64;

    fn div(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => l / r,
            (l, r) => panic!("Invalid expression, {l} / {r}"),
        }
    }
}

impl Rem for Value {
    type Output = f64;

    fn rem(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Self::Number(l), Self::Number(r)) => ((l as i32) % (r as i32)) as f64,
            (l, r) => panic!("Invaid expression, {l} % {r}"),
        }
    }
}

impl From<Value> for f64 {
    fn from(value: Value) -> Self {
        match value {
            Value::Number(n) => n,
            Value::Boolean(b) => {
                if b {
                    1f64
                } else {
                    0f64
                }
            }
            Value::Nil => 0f64,
            s => {
                panic!("Cannot use {s} as number")
            }
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> bool {
        match value {
            Value::Boolean(b) => b,
            Value::Number(n) => n != 0f64,
            Value::String(s) => !s.is_empty(),
            Value::Err(_) => false,
            Value::FiveMinuteCodingAdventure(_) => false,
            Value::Nil => false,
        }
    }
}

impl From<crate::values::tape::Value<'_>> for Value {
    fn from(value: crate::values::tape::Value<'_>) -> Self {
        match value {
            super::tape::Value::Nil => Value::Nil,
            super::tape::Value::Array(_a) => Value::Number(0f64),
            super::tape::Value::String(s) => Value::String(s.inner.clone()),
            super::tape::Value::FiveMinuteCodingAdventure(f) => {
                Value::FiveMinuteCodingAdventure(FiveMinuteCodingAdventure::new(f.get_nodes().into_iter().cloned().collect()))
            }
            super::tape::Value::Number(n) => Value::Number(n),
            super::tape::Value::Integer(i) => Value::Number(i as f64),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String(v) => write!(f, "{v}"),
            Self::Number(v) => write!(f, "{v}"),
            Self::Boolean(v) => write!(f, "{v}"),
            Self::Err(v) => write!(f, "{v}"),
            Self::FiveMinuteCodingAdventure(_) => write!(f, "<fiveminutecodingadventure>"),
            Self::Nil => write!(f, "Nil"),
        }
    }
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Plus => write!(f, "+"),
            Self::Minus => write!(f, "-"),
            Self::Greater => write!(f, ">"),
            Self::Less => write!(f, "<"),
            Self::GreaterEquals => write!(f, ">="),
            Self::LessEquals => write!(f, "<="),
            Self::Multiply => write!(f, "*"),
            Self::Divide => write!(f, "/"),
            Self::Modulo => write!(f, "%"),
            Self::Equals => write!(f, "=="),
            Self::NotEquals => write!(f, "!="),
            Self::BinaryOr => write!(f, "oug"),
            Self::BinaryXor => write!(f, "xoug"),
            Self::BinaryAnd => write!(f, "aoug"),
            Self::LogicalOr => write!(f, "ouoD"),
            Self::LogicalXor => write!(f, "xuoD"),
            Self::LogicalAnd => write!(f, "auoD"),
            Self::BitShiftLeft => write!(f, "lump"),
            Self::BitShiftRight => write!(f, "rump"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    pub inner: String,
}
impl AllocObject<LiteralList> for Text {
    const TYPE_ID: LiteralList = LiteralList::String;
}
impl AllocObject<TypeList> for Text {
    const TYPE_ID: TypeList = TypeList::String;
}
impl From<String> for Text {
    fn from(value: String) -> Self {
        Self {
            inner: value.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct FiveMinuteCodingAdventure {
    nodes: Box<[Stmt]>,
}
impl AllocObject<TypeList> for FiveMinuteCodingAdventure {
    const TYPE_ID: TypeList = TypeList::FiveMinuteCodingAdventure;
}
impl FiveMinuteCodingAdventure {
    pub fn get_nodes(&self) -> &[Stmt] {
        &self.nodes
    }

    pub fn new(nodes: Box<[Stmt]>) -> Self {
        Self { nodes }
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Nil {}
impl AllocObject<TypeList> for Nil {
    const TYPE_ID: TypeList = TypeList::Number;
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Array {}
