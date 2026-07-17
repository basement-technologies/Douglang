use crate::ast::{BoolOper, SetOper};
use crate::token::TokenKind;

pub fn comp_oper(kind: &TokenKind) -> Option<BoolOper> {
    Some(match kind {
        TokenKind::ComparisonEqual => BoolOper::Equal,
        TokenKind::ComparisonNotEqual => BoolOper::NotEqual,
        TokenKind::ComparisonGreater => BoolOper::Greater,
        TokenKind::ComparisonGreaterEqual => BoolOper::GreaterEqual,
        TokenKind::ComparisonLess => BoolOper::Less,
        TokenKind::ComparisonLessEqual => BoolOper::LessEqual,
        _ => return None,
    })
}

pub fn set_oper(kind: &TokenKind) -> Option<SetOper> {
    Some(match kind {
        TokenKind::Set => SetOper::Set,
        TokenKind::AddSet => SetOper::Add,
        TokenKind::SubSet => SetOper::Sub,
        TokenKind::MulSet => SetOper::Mul,
        TokenKind::DivSet => SetOper::Div,
        TokenKind::ModSet => SetOper::Mod,
        _ => return None,
    })
}

pub fn token_name(kind: &TokenKind) -> &'static str {
    match kind {
        TokenKind::Doug { .. } => "DougToken",
        TokenKind::Bald => "BaldToken",
        TokenKind::Tts => "TTSToken",
        TokenKind::Ttss => "TTSSToken",
        TokenKind::Set => "SetToken",
        TokenKind::AddSet => "AddSetToken",
        TokenKind::SubSet => "SubSetToken",
        TokenKind::MulSet => "MulSetToken",
        TokenKind::DivSet => "DivSetToken",
        TokenKind::ModSet => "ModSetToken",
        TokenKind::Loop => "LoopToken",
        TokenKind::Guod => "GuodToken",
        TokenKind::Rigged => "RiggedToken",
        TokenKind::Prediction => "PredictionToken",
        TokenKind::Believers => "BelieversToken",
        TokenKind::Doubters => "DoubtersToken",
        TokenKind::Win => "WinToken",
        TokenKind::FiveMinuteCodingAdventure => "FiveMinuteCodingAdventureToken",
        TokenKind::FmcaCall(_) => "FmcaCallToken",
        TokenKind::Call => "CallToken",
        TokenKind::End => "EndToken",
        TokenKind::Literal(_) => "LiteralToken",
        TokenKind::LParen => "LParenToken",
        TokenKind::RParen => "RParenToken",
        TokenKind::LSquare => "LSquareToken",
        TokenKind::RSquare => "RSquareToken",
        TokenKind::ComparisonEqual => "ComparisonEqualToken",
        TokenKind::ComparisonNotEqual => "ComparisonNotEqualToken",
        TokenKind::ComparisonGreater => "ComparisonGreaterToken",
        TokenKind::ComparisonLess => "ComparisonLessToken",
        TokenKind::ComparisonGreaterEqual => "ComparisonGreaterEqualToken",
        TokenKind::ComparisonLessEqual => "ComparisonLessEqualToken",
    }
}
