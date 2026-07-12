use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(line: usize, column: usize) -> Self {
        Span { line, column }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ValueLiteral {
    Int(i64),
    Float(f64),
    Str(String),
}

impl fmt::Display for ValueLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ValueLiteral::Int(v) => write!(f, "{v}"),
            ValueLiteral::Float(v) => write!(f, "{v}"),
            ValueLiteral::Str(v) => write!(f, "{v:?}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Doug { count: usize },
    Bald,

    Tts,
    Ttss,
    Set,
    AddSet,
    SubSet,
    MulSet,
    DivSet,
    ModSet,
    Loop,
    Goud,
    Rigged,
    Prediction,
    Believers,
    Doubters,
    Win,

    Literal(ValueLiteral),

    LParen,
    RParen,
    LSquare,
    RSquare,

    ComparisonEqual,
    ComparisonNotEqual,
    ComparisonGreater,
    ComparisonLess,
    ComparisonGreaterEqual,
    ComparisonLessEqual,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, line: usize, column: usize) -> Self {
        Token {
            kind,
            span: Span::new(line, column),
        }
    }
}
