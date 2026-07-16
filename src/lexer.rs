use crate::token::{ValueLiteral, Token, TokenKind};

#[derive(Debug, Clone)]
pub struct LexErr {
    pub message: String,
}

impl LexErr {
    fn new(line: usize, column: usize, msg: &str) -> Self {
        LexErr {
            message: format!(
                "You are literally trolling. {msg} on line {line}, column {column}"
            ),
        }
    }
}

impl std::fmt::Display for LexErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for LexErr {}

fn resolve_keyword(keyword: &str) -> Option<TokenKind> {
    Some(match keyword {
        "ttss" => TokenKind::Ttss,
        "tts" => TokenKind::Tts,
        "set" => TokenKind::Set,
        "+set" => TokenKind::AddSet,
        "-set" => TokenKind::SubSet,
        "*set" => TokenKind::MulSet,
        "/set" => TokenKind::DivSet,
        "%set" => TokenKind::ModSet,
        "loop" => TokenKind::Loop,
        "guoD" => TokenKind::Guod,
        "Rigged" => TokenKind::Rigged,
        "prediction" => TokenKind::Prediction,
        "Believers" => TokenKind::Believers,
        "Doubters" => TokenKind::Doubters,
        "win" => TokenKind::Win,
        "(" => TokenKind::LParen,
        ")" => TokenKind::RParen,
        "[" => TokenKind::LSquare,
        "]" => TokenKind::RSquare,
        "=" => TokenKind::ComparisonEqual,
        "!=" => TokenKind::ComparisonNotEqual,
        ">" => TokenKind::ComparisonGreater,
        ">=" => TokenKind::ComparisonGreaterEqual,
        "<" => TokenKind::ComparisonLess,
        "<=" => TokenKind::ComparisonLessEqual,
        _ => return None,
    })
}

const KEYWORDS: &[&str] = &[
    "prediction",
    "Believers",
    "Doubters",
    "Rigged",
    "guoD",
    "loop",
    "ttss",
    "+set",
    "-set",
    "*set",
    "/set",
    "%set",
    "tts",
    "set",
    "win",
    "!=",
    ">=",
    "<=",
    "(",
    ")",
    "[",
    "]",
    "=",
    ">",
    "<",
];

pub fn lex(source: &str) -> Result<Vec<Token>, LexErr> {
    let chars: Vec<char> = source.chars().collect();
    let length = chars.len();

    let mut tokens: Vec<Token> = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    let mut column = 1usize;

    let starts_with = |chars: &[char], i: usize, pat: &str| -> bool {
        let pat_chars: Vec<char> = pat.chars().collect();
        if i + pat_chars.len() > chars.len() {
            return false;
        }
        chars[i..i + pat_chars.len()] == pat_chars[..]
    };

    while i < length {
        let c = chars[i];

        if c == '\n' {
            i += 1;
            line += 1;
            column = 1;
            continue;
        }

        if c.is_whitespace() {
            i += 1;
            column += 1;
            continue;
        }

        if starts_with(&chars, i, "D:") {
            let start_line = line;
            let start_col = column;
            i += 2;
            column += 2;
            while i < length && !starts_with(&chars, i, ":D") {
                if chars[i] == '\n' {
                    i += 1;
                    line += 1;
                    column = 1;
                } else {
                    i += 1;
                    column += 1;
                }
            }
            if !starts_with(&chars, i, ":D") {
                return Err(LexErr::new(start_line, start_col, "unclosed comment"));
            }
            i += 2;
            column += 2;
            continue;
        }

        if starts_with(&chars, i, "Bald") {
            tokens.push(Token::new(TokenKind::Bald, line, column));
            i += 4;
            column += 4;
            continue;
        }

        if starts_with(&chars, i, "Doug") {
            let start_col = column;
            let mut count = 0usize;
            while starts_with(&chars, i, "Doug") {
                count += 1;
                i += 4;
                column += 4;
            }
            tokens.push(Token::new(TokenKind::Doug { count }, line, start_col));
            continue;
        }

        let mut matched = false;
        for &keyword in KEYWORDS {
            if starts_with(&chars, i, keyword) {
                let kind = resolve_keyword(keyword).expect("KEYWORDS entry missing from resolve_keyword");
                tokens.push(Token::new(kind, line, column));
                let klen = keyword.chars().count();
                i += klen;
                column += klen;
                matched = true;
                break;
            }
        }
        if matched {
            continue;
        }

        if c == '"' || c == '\'' {
            let quote_char = c;
            let start_line = line;
            let start_col = column;
            i += 1;
            column += 1;
            let mut str_content = String::new();
            let mut terminated = false;
            while i < length {
                if chars[i] == '\\' && i + 1 < length {
                    str_content.push(chars[i]);
                    str_content.push(chars[i + 1]);
                    i += 2;
                    column += 2;
                } else if chars[i] == quote_char {
                    i += 1;
                    column += 1;
                    terminated = true;
                    break;
                } else if chars[i] == '\n' {
                    str_content.push('\n');
                    line += 1;
                    column = 1;
                    i += 1;
                } else {
                    str_content.push(chars[i]);
                    i += 1;
                    column += 1;
                }
            }
            if !terminated {
                return Err(LexErr::new(start_line, start_col, "unclosed string literal"));
            }
            tokens.push(Token::new(
                TokenKind::Literal(ValueLiteral::Str(str_content)),
                start_line,
                start_col,
            ));
            continue;
        }

        if c.is_ascii_digit() || c == '.' {
            let start_i = i;
            let start_col = column;
            let mut has_decimal = c == '.';
            i += 1;
            column += 1;
            while i < length && (chars[i].is_ascii_digit() || chars[i] == '.') {
                if chars[i] == '.' {
                    if has_decimal {
                        return Err(LexErr::new(line, column, "malformed number"));
                    }
                    has_decimal = true;
                }
                i += 1;
                column += 1;
            }
            let text: String = chars[start_i..i].iter().collect();
            let value = if has_decimal {
                ValueLiteral::Float(
                    text.parse::<f64>()
                        .map_err(|_| LexErr::new(line, start_col, "malformed number"))?,
                )
            } else {
                ValueLiteral::Int(
                    text.parse::<i64>()
                        .map_err(|_| LexErr::new(line, start_col, "malformed number"))?,
                )
            };
            tokens.push(Token::new(TokenKind::Literal(value), line, start_col));
            continue;
        }

        return Err(LexErr::new(line, column, "unknown token"));
    }

    Ok(tokens)
}
