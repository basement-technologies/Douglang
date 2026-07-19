use thiserror::Error;

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

#[derive(Error, Debug, Clone, PartialEq, PartialOrd)]
pub enum RuntimeError {
    #[error("Value {0} is out of range of Tape")]
    OutOfRange(i32),
    #[error("Stack overflow :3")]
    TapeOverFlow,

    #[error("{0} is not defined")]
    NotDefined(String),

    #[error("Expression {0} requires both a left and right value")]
    NoRExpression(String),

    #[error("Could not evaluate expression {0} {1} {2}")]
    BadExpression(String, String, String),

    #[error("Allocation error: {0}")]
    AllocError(String),

    #[error("Syntax error: {0}")]
    SyntaxError(String),

    #[error("Value is nil")]
    SegmentationFault,

    #[error("{0}")]
    Unexpected(String),

    #[error("Attempting to call what is not a function")]
    NotAFunction,
}

impl RuntimeError {
    pub fn new(err: impl Into<String>) -> Self {
        Self::Unexpected(err.into())
    }
}

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
