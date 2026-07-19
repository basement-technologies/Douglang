#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Modulo,

    Equals,
    NotEquals,
    Greater,
    GreaterEquals,
    Less,
    LessEquals,

    BitShiftRight,
    BitShiftLeft,
    BinaryXor,
    BinaryOr,
    BinaryAnd,

    LogicalXor,
    LogicalOr,
    LogicalAnd,
}
