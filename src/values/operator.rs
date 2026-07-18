#[derive(Clone, Copy, Debug)]
pub enum Operator {
    Plus,
    Minus,
    Multiply,
    Divide,

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
