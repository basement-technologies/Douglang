use crate::token::ValueLiteral;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoolOper {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SetOper {
    Set,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DougChain {
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(ValueLiteral),
    DougSequence { chains: Vec<DougChain> },
    Rigged { func: String, args: Vec<Expr> },
}

#[derive(Debug, Clone, PartialEq)]
pub struct Condition {
    pub left: Expr,
    pub oper: BoolOper,
    pub right: Expr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Doug {
        chains: Vec<DougChain>,
        reset: bool,
    },
    Tts {
        msg: Option<Expr>,
        use_index: bool,
        overlap: bool,
    },
    Set {
        value: Expr,
        oper: SetOper,
    },
    Expr(Expr),
    Loop {
        body: Vec<Stmt>,
    },
    Guod,
    Prediction {
        believe_body: Vec<Stmt>,
        doubt_body: Vec<Stmt>,
        condition: Condition,
    },
}
