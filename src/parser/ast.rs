use crate::values::{Operator, tape::TaggedCellPtr};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DougChain {
    pub count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal(TaggedCellPtr),
    Variable(String),
    DougSequence {
        chains: Vec<DougChain>,
    },
    MainTapeDougSequence {
        chains: Vec<DougChain>,
    },
    Rigged {
        func: String,
        args: Vec<Expr>,
    },
    FmcaCall {
        name: Option<String>,
    },
    Condition {
        left: Box<Expr>,
        operator: Option<Operator>,
        right: Option<Box<Expr>>,
    },
}

pub enum Reference {
    Doug(DougChain),
    Variable(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Doug {
        chains: Box<[Reference]>,
        reset: bool,
    },
    Tts {
        msg: Option<Expr>,
        use_index: bool,
        overlap: bool,
    },
    Call {
        name: Option<String>,
        use_index: bool,
    },
    Guod {
        value: Option<Expr>,
        use_index: bool,
    },
    Set {
        value: Expr,
        oper: Option<Operator>,
    },
    Expr(Expr),
    Loop {
        body: Box<[Stmt]>,
    },
    Prediction {
        believe_body: Box<[Stmt]>,
        doubt_body: Box<[Stmt]>,
        condition: Expr,
    },
    FiveMinuteCodingAdventure {
        name: String,
        body: Box<[Stmt]>,
    },
}
