use crate::ast::{DougChain, Stmt};

pub const RUNTIME_ERROR_DIVISION_BY_ZERO: &str = "division by zero";
pub const RUNTIME_ERROR_MODULO_BY_ZERO: &str = "modulo by zero";
pub const RUNTIME_ERROR_BREAK_OUTSIDE_LOOP: &str = "guoD can only be used inside loop";
pub const RUNTIME_ERROR_INTEGER_OVERFLOW_MODULO: &str = "integer overflow during modulo";
pub const TTS_STATE_TEXT_KEY: &str = "TEXT";
pub const TTS_STATE_AMP_KEY: &str = "AMP";
pub const TTS_STATE_SPEAKING_KEY: &str = "SPEAKING";
pub const TTS_STATE_DONE_MARKER: &str = "__DOUGLANG_DONE__";
pub const TTS_SPEAKING_AMP: f64 = 0.5;
pub const TTS_IDLE_AMP: f64 = 0.0;

pub fn doug_index_overflow_message(count: usize) -> String {
    format!("Doug chain of length {count} is too large to index safely")
}

pub fn escape_tts_state_text(text: &str) -> String {
    text.replace('\\', "\\\\")
        .replace('\n', "\\n")
        .replace('|', "\\p")
}

pub fn unescape_tts_state_text(text: &str) -> String {
    let mut unescaped = String::new();
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('p') => unescaped.push('|'),
                Some('n') => unescaped.push('\n'),
                Some('\\') => unescaped.push('\\'),
                Some(other) => {
                    unescaped.push('\\');
                    unescaped.push(other);
                }
                None => unescaped.push('\\'),
            }
        } else {
            unescaped.push(ch);
        }
    }
    unescaped
}

#[derive(Debug, Clone, PartialEq)]
pub struct TtsState {
    pub text: String,
    pub amplitude: f64,
    pub speaking: bool,
}

pub fn format_tts_state(text: &str, amplitude: f64, speaking: bool) -> String {
    format!(
        "{}|{}\n{}|{}\n{}|{}\n",
        TTS_STATE_TEXT_KEY,
        escape_tts_state_text(text),
        TTS_STATE_AMP_KEY,
        amplitude,
        TTS_STATE_SPEAKING_KEY,
        if speaking { 1 } else { 0 }
    )
}

pub fn parse_tts_state(raw: &str) -> Option<TtsState> {
    if raw.trim() == TTS_STATE_DONE_MARKER {
        return None;
    }
    let text_prefix = format!("{}|", TTS_STATE_TEXT_KEY);
    if !raw.contains(&text_prefix) {
        return Some(TtsState {
            text: raw.to_string(),
            amplitude: TTS_IDLE_AMP,
            speaking: false,
        });
    }

    let mut text = String::new();
    let mut amplitude = TTS_IDLE_AMP;
    let mut speaking = false;
    for line in raw.lines() {
        if let Some(rest) = line.strip_prefix(&text_prefix) {
            text = unescape_tts_state_text(rest);
        } else if let Some(rest) = line.strip_prefix(&format!("{}|", TTS_STATE_AMP_KEY)) {
            amplitude = rest.parse::<f64>().unwrap_or(TTS_IDLE_AMP);
        } else if let Some(rest) = line.strip_prefix(&format!("{}|", TTS_STATE_SPEAKING_KEY)) {
            speaking = rest == "1";
        }
    }
    Some(TtsState {
        text,
        amplitude,
        speaking,
    })
}

#[derive(Debug, Clone)]
pub enum Value {
    Int(i64),
    Float(f64),
    Str(String),
    FiveMinuteCodingAdventure { body: Vec<Stmt> },
}

impl Value {
    pub fn as_f64(&self) -> f64 {
        match self {
            Value::Int(v) => *v as f64,
            Value::Float(v) => *v,
            Value::Str(s) => s.parse::<f64>().unwrap_or(0.0),
            Value::FiveMinuteCodingAdventure { .. } => 0.0,
        }
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Value::Str(_))
    }
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Int(v) => write!(f, "{v}"),
            Value::Float(v) => {
                if v.fract() == 0.0 && v.is_finite() {
                    write!(f, "{v:.1}")
                } else {
                    write!(f, "{v}")
                }
            }
            Value::Str(v) => write!(f, "{v}"),
            Value::FiveMinuteCodingAdventure { .. } => write!(f, "<five_minute_coding_adventure>"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum RuntimeErrorKind {
    DougIndexOverflow { count: usize },
    DivisionByZero,
    ModuloByZero,
    IntegerOverflow { operation: String },
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

    pub fn doug_index_overflow(count: usize) -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::DougIndexOverflow { count },
            message: doug_index_overflow_message(count),
        }
    }

    pub fn division_by_zero() -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::DivisionByZero,
            message: RUNTIME_ERROR_DIVISION_BY_ZERO.to_string(),
        }
    }

    pub fn modulo_by_zero() -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::ModuloByZero,
            message: RUNTIME_ERROR_MODULO_BY_ZERO.to_string(),
        }
    }

    pub fn integer_overflow(operation: &str) -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::IntegerOverflow {
                operation: operation.to_string(),
            },
            message: if operation == "modulo" {
                RUNTIME_ERROR_INTEGER_OVERFLOW_MODULO.to_string()
            } else {
                format!("integer overflow during {operation}")
            },
        }
    }

    pub fn break_outside_loop() -> Self {
        RuntimeError {
            kind: RuntimeErrorKind::BreakOutsideLoop,
            message: RUNTIME_ERROR_BREAK_OUTSIDE_LOOP.to_string(),
        }
    }
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for RuntimeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiArgKind {
    Double,
    String,
    UInt,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FfiReturnKind {
    Double,
    Int,
    UInt,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FfiSignature {
    pub return_kind: FfiReturnKind,
    pub arg_kinds: Vec<FfiArgKind>,
}

pub fn ffi_signature(func_name: &str, arg_count: usize) -> FfiSignature {
    let (return_kind, arg_kind) = match func_name {
        "puts" | "fputs" => (FfiReturnKind::Int, FfiArgKind::String),
        "sleep" | "Sleep" => (FfiReturnKind::UInt, FfiArgKind::UInt),
        _ => (FfiReturnKind::Double, FfiArgKind::Double),
    };
    FfiSignature {
        return_kind,
        arg_kinds: vec![arg_kind; arg_count],
    }
}

pub fn validate_doug_chains(chains: &[DougChain]) -> Result<(), RuntimeError> {
    for chain in chains {
        if chain.count == 0 || chain.count >= 64 {
            return Err(RuntimeError::doug_index_overflow(chain.count));
        }
    }
    Ok(())
}

pub fn doug_index(chains: &[DougChain], start_i: i64) -> Result<i64, RuntimeError> {
    validate_doug_chains(chains)?;
    let mut res_i = start_i;
    for (i, chain) in chains.iter().enumerate() {
        let shift = chain.count - 1;
        let value = 1i64 << shift;
        res_i = if i % 2 == 0 {
            res_i.checked_add(value)
        } else {
            res_i.checked_sub(value)
        }
        .ok_or(RuntimeError::doug_index_overflow(chain.count))?;
    }
    Ok(res_i)
}

pub fn add(left: &Value, rhs: &Value) -> Value {
    if left.is_string() || rhs.is_string() {
        Value::Str(format!("{left}{rhs}"))
    } else {
        match (left, rhs) {
            (Value::Float(_), _) | (_, Value::Float(_)) => {
                Value::Float(left.as_f64() + rhs.as_f64())
            }
            (Value::Int(a), Value::Int(b)) => a
                .checked_add(*b)
                .map(Value::Int)
                .unwrap_or_else(|| Value::Float(left.as_f64() + rhs.as_f64())),
            _ => Value::Float(left.as_f64() + rhs.as_f64()),
        }
    }
}

pub fn sub(left: &Value, rhs: &Value) -> Value {
    match (left, rhs) {
        (Value::Float(_), _) | (_, Value::Float(_)) => Value::Float(left.as_f64() - rhs.as_f64()),
        (Value::Int(a), Value::Int(b)) => a
            .checked_sub(*b)
            .map(Value::Int)
            .unwrap_or_else(|| Value::Float(left.as_f64() - rhs.as_f64())),
        _ => Value::Float(left.as_f64() - rhs.as_f64()),
    }
}

pub fn mul(left: &Value, rhs: &Value) -> Value {
    match (left, rhs) {
        (Value::Float(_), _) | (_, Value::Float(_)) => Value::Float(left.as_f64() * rhs.as_f64()),
        (Value::Int(a), Value::Int(b)) => a
            .checked_mul(*b)
            .map(Value::Int)
            .unwrap_or_else(|| Value::Float(left.as_f64() * rhs.as_f64())),
        _ => Value::Float(left.as_f64() * rhs.as_f64()),
    }
}

pub fn div(left: &Value, rhs: &Value) -> Result<Value, RuntimeError> {
    let denom = rhs.as_f64();
    if denom == 0.0 {
        return Err(RuntimeError::division_by_zero());
    }
    Ok(Value::Float(left.as_f64() / denom))
}

pub fn modulo(left: &Value, rhs: &Value) -> Result<Value, RuntimeError> {
    match (left, rhs) {
        (Value::Int(_), Value::Int(0)) => Err(RuntimeError::modulo_by_zero()),
        (Value::Int(a), Value::Int(b)) => a
            .checked_rem(*b)
            .map(Value::Int)
            .ok_or_else(|| RuntimeError::integer_overflow("modulo")),
        _ => {
            let denom = rhs.as_f64();
            if denom == 0.0 {
                Err(RuntimeError::modulo_by_zero())
            } else {
                Ok(Value::Float(left.as_f64() % denom))
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeTape {
    left: Vec<Value>,
    right: Vec<Value>,
    index: i64,
}

impl RuntimeTape {
    pub fn new() -> Self {
        RuntimeTape {
            left: Vec::new(),
            right: vec![Value::Int(0)],
            index: 0,
        }
    }

    pub fn get(&self, i: i64) -> Value {
        if i < 0 {
            let idx = (-i - 1) as usize;
            self.left.get(idx).cloned().unwrap_or(Value::Int(0))
        } else {
            let idx = i as usize;
            self.right.get(idx).cloned().unwrap_or(Value::Int(0))
        }
    }

    pub fn current(&self) -> Value {
        self.get(self.index)
    }

    pub fn index(&self) -> i64 {
        self.index
    }

    pub fn set_index(&mut self, index: i64) {
        self.index = index;
    }

    pub fn set(&mut self, i: i64, value: Value) -> Result<(), RuntimeError> {
        if i < 0 {
            let idx = (-i - 1) as usize;
            while self.left.len() <= idx {
                self.left.push(Value::Int(0));
            }
            self.left[idx] = value;
        } else {
            let idx = i as usize;
            while self.right.len() <= idx {
                self.right.push(Value::Int(0));
            }
            self.right[idx] = value;
        }
        Ok(())
    }

    pub fn set_current(&mut self, value: Value) -> Result<(), RuntimeError> {
        self.set(self.index, value)
    }

    pub fn move_doug(&mut self, chains: &[DougChain], reset: bool) -> Result<(), RuntimeError> {
        let start = if reset { 0 } else { self.index };
        self.index = doug_index(chains, start)?;
        Ok(())
    }
}

impl Default for RuntimeTape {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn doug_index_matches_contract() {
        let chains = vec![
            DougChain { count: 1 },
            DougChain { count: 2 },
            DougChain { count: 3 },
        ];
        assert_eq!(doug_index(&chains, 10).unwrap(), 13);
    }

    #[test]
    fn doug_count_64_is_overflow() {
        let err = doug_index(&[DougChain { count: 64 }], 0).unwrap_err();
        assert_eq!(err.kind, RuntimeErrorKind::DougIndexOverflow { count: 64 });
    }

    #[test]
    fn tape_allows_sparse_writes() {
        let mut tape = RuntimeTape::new();
        assert!(tape.set(4, Value::Int(9)).is_ok());
        assert!(matches!(tape.get(1), Value::Int(0)));
        assert!(matches!(tape.get(4), Value::Int(9)));
    }

    #[test]
    fn checked_integer_arithmetic_falls_back_to_float() {
        assert!(matches!(
            add(&Value::Int(i64::MAX), &Value::Int(1)),
            Value::Float(_)
        ));
        assert!(matches!(
            sub(&Value::Int(i64::MIN), &Value::Int(1)),
            Value::Float(_)
        ));
        assert!(matches!(
            mul(&Value::Int(i64::MAX), &Value::Int(2)),
            Value::Float(_)
        ));
    }

    #[test]
    fn ffi_signature_contract_is_shared() {
        let puts = ffi_signature("puts", 1);
        assert_eq!(puts.return_kind, FfiReturnKind::Int);
        assert_eq!(puts.arg_kinds, vec![FfiArgKind::String]);

        let pow = ffi_signature("pow", 2);
        assert_eq!(pow.return_kind, FfiReturnKind::Double);
        assert_eq!(pow.arg_kinds, vec![FfiArgKind::Double, FfiArgKind::Double]);
    }

    #[test]
    fn tts_state_round_trips_from_shared_contract() {
        let state = format_tts_state("hello|Doug\nline", TTS_SPEAKING_AMP, true);
        assert_eq!(
            parse_tts_state(&state),
            Some(TtsState {
                text: "hello|Doug\nline".to_string(),
                amplitude: TTS_SPEAKING_AMP,
                speaking: true,
            })
        );
        assert_eq!(parse_tts_state(TTS_STATE_DONE_MARKER), None);
    }
}
