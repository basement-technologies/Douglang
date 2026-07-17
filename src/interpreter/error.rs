#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeErrorKind {
    InvalidTapeWrite { index: i64, next_valid: i64 },
    DougIndexOverflow { count: usize },
    DivisionByZero,
    ModuloByZero,
    BreakOutsideLoop,
    Ffi(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub kind: RuntimeErrorKind,
    pub message: String,
}

impl RuntimeError {
    pub fn new(msg: &str) -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::Ffi(msg.to_string()),
            message: msg.to_string(),
        }
    }

    pub fn invalid_tape_write(index: i64, next_valid: i64) -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::InvalidTapeWrite { index, next_valid },
            message: format!(
                "invalid tape write at index {index}; next writable index is {next_valid}"
            ),
        }
    }

    pub fn doug_index_overflow(count: usize) -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::DougIndexOverflow { count },
            message: format!("Doug chain of length {count} is too large to index safely"),
        }
    }

    pub fn division_by_zero() -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::DivisionByZero,
            message: "division by zero".to_string(),
        }
    }

    pub fn modulo_by_zero() -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::ModuloByZero,
            message: "modulo by zero".to_string(),
        }
    }

    pub fn break_outside_loop() -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::BreakOutsideLoop,
            message: "guoD can only be used inside loop".to_string(),
        }
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {}
