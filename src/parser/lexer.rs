use crate::values::Operator;

use std::{
    fmt::{Debug, Display},
    fs::File,
    io::{BufRead, BufReader},
};
use thiserror::Error;

use crate::values::tape::{ROData, TaggedCellPtr};

#[derive(Clone, Copy, Debug)]
pub enum KeyWord {
    Tts,
    Ttss,
    Set,
    DougChain(usize),
    Bald,
    Loop,
    Prediction,
    Believers,
    Doubters,
    Wins,
    Break,
    Call,
    FiveMinuteCodingAdventure,
    Guod,
    Rigged,
}

#[derive(Clone, Copy, Debug)]
pub enum ParenThesis {
    Left,
    Right,
    SquareLeft,
    SquareRight,
    AngleLeft,
    AngleRight,
}

#[derive(Clone, Debug)]
pub enum Token {
    KeyWord(KeyWord),
    Operator(Operator),
    Paren(ParenThesis),
    Literal(TaggedCellPtr),
    Variable(String),
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::KeyWord(k) => k.fmt(f),
            Self::Paren(p) => p.fmt(f),
            Self::Literal(l) => Debug::fmt(l, f),
            Self::Variable(s) => Display::fmt(s, f),
            Self::Operator(o) => Debug::fmt(o, f),
        }
    }
}

fn match_token(s: &str) -> Option<Token> {
    match s {
        // Keywords
        "tts" => Some(Token::KeyWord(KeyWord::Tts)),
        "ttss" => Some(Token::KeyWord(KeyWord::Ttss)),
        "set" => Some(Token::KeyWord(KeyWord::Set)),
        "Doug" => Some(Token::KeyWord(KeyWord::DougChain(1))),
        "Bald" => Some(Token::KeyWord(KeyWord::Bald)),
        "Rigged" => Some(Token::KeyWord(KeyWord::Rigged)),
        "loop" => Some(Token::KeyWord(KeyWord::Loop)),
        "prediction" => Some(Token::KeyWord(KeyWord::Prediction)),
        "Believers" => Some(Token::KeyWord(KeyWord::Believers)),
        "Doubters" => Some(Token::KeyWord(KeyWord::Doubters)),
        "win" => Some(Token::KeyWord(KeyWord::Wins)),
        "break" => Some(Token::KeyWord(KeyWord::Break)),
        "five_minute_coding_adventure" => Some(Token::KeyWord(KeyWord::FiveMinuteCodingAdventure)),
        "call" => Some(Token::KeyWord(KeyWord::Call)),
        "guoD" => Some(Token::KeyWord(KeyWord::Guod)),

        // Parenthesis
        "(" => Some(Token::Paren(ParenThesis::Left)),
        ")" => Some(Token::Paren(ParenThesis::Right)),
        "[" => Some(Token::Paren(ParenThesis::SquareLeft)),
        "]" => Some(Token::Paren(ParenThesis::SquareRight)),

        // Operators
        "+" => Some(Token::Operator(Operator::Plus)),
        "-" => Some(Token::Operator(Operator::Minus)),
        "*" => Some(Token::Operator(Operator::Multiply)),
        "/" => Some(Token::Operator(Operator::Divide)),
        "%" => Some(Token::Operator(Operator::Modulo)),

        "lump" => Some(Token::Operator(Operator::BitShiftRight)),
        "rump" => Some(Token::Operator(Operator::BitShiftLeft)),
        "xoug" => Some(Token::Operator(Operator::BinaryXor)),
        "oug" => Some(Token::Operator(Operator::BinaryOr)),
        "aoug" => Some(Token::Operator(Operator::BinaryAnd)),

        "xuoD" => Some(Token::Operator(Operator::LogicalAnd)),
        "ouoD" => Some(Token::Operator(Operator::LogicalOr)),
        "auoD" => Some(Token::Operator(Operator::LogicalXor)),

        ">" => Some(Token::Operator(Operator::Greater)),
        "<" => Some(Token::Operator(Operator::Less)),
        "<=" => Some(Token::Operator(Operator::LessEquals)),
        ">=" => Some(Token::Operator(Operator::GreaterEquals)),
        "!=" => Some(Token::Operator(Operator::NotEquals)),
        "=" => Some(Token::Operator(Operator::Equals)),
        _ => None,
    }
}

#[must_use]
fn merge_words(words: &[&str]) -> Box<[String]> {
    let mut split: Vec<String> = Vec::new();
    for word in words {
        let mut current = String::new();
        let chars: Vec<char> = word.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            match chars[i] {
                '(' | ')' | '[' | ']' | '{' | '}' => {
                    if !current.is_empty() {
                        split.push(current.clone());
                        current.clear();
                    }
                    split.push(chars[i].to_string());
                    i += 1;
                }
                's' if chars.get(i + 1) == Some(&'s') && chars[i - 2..=i] == ['t', 't', 's'] => {
                    if !current.is_empty() {
                        split.push(current.clone());
                        current.clear();
                    }
                    split.push(chars[i].to_string());
                    i += 1;
                }
                _ => {
                    current.push(chars[i]);
                    i += 1;
                }
            }
        }
        if !current.is_empty() {
            split.push(current);
        }
    }

    let mut merged: Vec<String> = Vec::new();
    let mut i = 0;

    while i < split.len() {
        let word = &split[i];
        if let Some(start) = word.find('"') {
            if word[start + 1..].contains('"') {
                merged.push(word.clone());
                i += 1;
            } else {
                let mut group = word.clone();
                i += 1;
                while i < split.len() {
                    group.push(' ');
                    group.push_str(&split[i]);
                    if split[i].contains('"') {
                        i += 1;
                        break;
                    }
                    i += 1;
                }
                merged.push(group);
            }
        } else {
            merged.push(word.clone());
            i += 1;
        }
    }

    merged.into()
}

/// Creates a list of potential [`Token`]s from a [`word`] input. If there is an error parsing
/// the next chunk as a [`Token`] then what is pushed is an [`Err`].
#[must_use]
fn from_word<'a>(word: &str, mutator: &'a ROData<'a>) -> Box<[Result<Token, LexerError>]> {
    let mut res = Vec::new();
    let mut i = 0;
    while i < word.len() {
        let mut matched = false;
        for j in ((i + 1)..=word.len()).rev() {
            let view = &word[i..j];
            if view == "Doug" {
                let c = (i..word.len())
                    .step_by(4)
                    .take_while(|k| word.get(*k..*k + 4) == Some("Doug"))
                    .count();
                res.push(Ok(Token::KeyWord(KeyWord::DougChain(c))));
                i += c * 4;
                matched = true;
                break;
            }
            if let Some(token) = match_token(view) {
                res.push(Ok(token.clone()));
                i = j;
                matched = true;
                break;
            }
        }
        if !matched {
            break;
        }
    }

    if i == 0 {
        if let Some(node) = ROData::alloc(mutator, word.to_string()) {
            res.push(Ok(Token::Literal(node)));
        } else {
            res.push(Ok(Token::Variable(word.to_string())));
        }
    } else if i + 1 < word.len() {
        res.push(Err(LexerError::InvalidToken(word[i..].to_string())));
    }

    res.into()
}

#[derive(Error, Debug, Clone)]
pub enum LexerError {
    #[error("This token is invalid: {0}")]
    InvalidToken(String),

    #[error("No closing quotation mark")]
    StringNotClosed,

    #[error("No closing `[`")]
    NoClosingBracket,

    #[error("Invalid Number")]
    InvalidNumber,

    #[error("No more to read")]
    EOFReached,

    #[error("Binary file provided")]
    BinaryFile,
}

pub struct Lexer<'guard> {
    /// The buffer that the lexer reads from
    reader: BufReader<File>,
    /// The mutator that provides access to modify the `Tape` - Required to allocate literals into
    /// it.
    data: ROData<'guard>,
    comment: bool,
}

impl<'a> Lexer<'a> {
    /// Creates a new [`Lexer`]
    ///
    /// # Panics
    /// If there is not a valid file path inputted.
    #[must_use]
    pub fn new(path: impl Into<String>, data: ROData<'a>) -> Lexer<'a> {
        let file = File::open(path.into()).expect("There should have been a valid path inputted");
        let reader = BufReader::new(file);

        Self {
            reader,
            data,
            comment: false,
        }
    }

    fn strip_comments(&mut self, input: &str) -> String {
        let mut output = String::new();
        let mut index = 0;

        while index < input.len() {
            if self.comment {
                if let Some(end) = input[index..].find(":D") {
                    index += end + 2;
                    self.comment = false;
                } else {
                    break;
                }
            } else if input[index..].starts_with("D:") {
                self.comment = true;
                index += 2;
            } else {
                let char = input[index..].chars().next().unwrap();
                output.push(char);
                index += char.len_utf8();
            }
        }

        output
    }

    /// Lex a block
    ///
    /// The same thing as `lex_line` except it loops until it reaches `]`.
    ///
    /// # Errors
    /// If there are no more bytes to be read, or if there is no closing brace.
    pub fn lex_block(&mut self) -> Result<Box<[Token]>, LexerError> {
        let mut block = Vec::new();
        self.reader
            .read_until(b']', &mut block)
            .map_err(|_| LexerError::NoClosingBracket)
            .and_then(|a| {
                if a == 0 {
                    Err(LexerError::EOFReached)
                } else {
                    Ok(a)
                }
            })?;
        let words = String::from_utf8(block).map_err(|_| LexerError::BinaryFile)?;
        let words = self.strip_comments(&words);
        let data = &mut self.data;
        merge_words(&words.split_whitespace().collect::<Box<[_]>>())
            .iter()
            .flat_map(|w| from_word(w, data))
            .collect()
    }

    /// Lex a line
    ///
    /// This function simply reads more lines from the [`Lexer::reader`] in a loop until there is a
    /// line with [`Token`]s in it.
    ///
    /// # Errors
    /// If there are no more bytes.
    pub fn lex_line(&mut self) -> Result<Box<[Token]>, LexerError> {
        loop {
            let line = &mut String::new();
            let bytes = self
                .reader
                .read_line(line)
                .map_err(|_| LexerError::EOFReached)?;

            if bytes == 0 {
                return Err(LexerError::EOFReached);
            }

            let line = self.strip_comments(line);
            let tokens = merge_words(&line.split_whitespace().collect::<Box<[_]>>());

            #[cfg(debug_assertions)]
            for token in &tokens {
                print!("Token: {token}  ");
            }

            let result: Result<Box<[Token]>, LexerError> = tokens
                .iter()
                .flat_map(|w| from_word(w, &self.data))
                .collect();

            let tokens = result?;
            if !tokens.is_empty() {
                #[cfg(debug_assertions)]
                println!();
                return Ok(tokens);
            }
        }
    }
}
