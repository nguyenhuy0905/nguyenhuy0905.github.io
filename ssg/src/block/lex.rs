#![allow(dead_code)]
#![allow(unused)]
use std::{
    num::{ParseFloatError, ParseIntError},
    str::FromStr,
};
use unicode_segmentation::UnicodeSegmentation;

/// Block lexer
///
/// The current implementation has quite bad debug output. The most we can output is, where the
/// start/end of the block that errs is.
pub struct Lex {
    pub(in crate::block) tokens: Vec<TokenType>,
    state: LexState,
    curr_token: String,
}

impl Lex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lex(&mut self, input: &str) -> Result<(), LexError> {
        for gr in input.graphemes(true) {
            match self.state {
                LexState::Init => {
                    self.lex_init(gr)?;
                }
                LexState::Id => {
                    self.lex_id(gr)?;
                }
                LexState::Int => {
                    self.lex_int(gr)?;
                }
                LexState::Float => {
                    self.lex_float(gr)?;
                }
                LexState::IntExponent => {
                    self.lex_int_exponent(gr)?;
                }
                LexState::FloatExponent => {
                    self.lex_float_exponent(gr)?;
                }
                LexState::Include => {
                    self.lex_include(gr)?;
                }
                LexState::LitStr => {
                    self.lex_lit_str(gr)?;
                }
                LexState::Escape => {
                    self.lex_escape(gr)?;
                }
            }
        }

        // not yet done...
        match self.state {
            LexState::Init => {},
            LexState::Id => {
                if let Some(kw) = Self::check_for_keyword(&self.curr_token) {
                    self.tokens.push(kw);
                } else {
                    self.tokens
                        .push(TokenType::Id(String::from_iter(self.curr_token.drain(..))));
                }
            }
            LexState::Int | LexState::IntExponent => {
                println!("{}", self.curr_token);
                self.tokens.push(TokenType::Int(
                    Self::int_from_str(&self.curr_token).unwrap(),
                ));
            }
            LexState::Float | LexState::FloatExponent => {
                self.tokens
                    .push(TokenType::Float(f64::from_str(&self.curr_token).unwrap()));
            }
            LexState::Include => {
                self.tokens.push(TokenType::Include(String::from_iter(
                    self.curr_token.drain(..),
                )));
            }
            LexState::LitStr | LexState::Escape => return Err(LexError::UnclosedStr),
        }
        self.tokens.push(TokenType::Eof);
        Ok(())
    }

    fn lex_init(&mut self, gr: &str) -> Result<(), LexError> {
        if gr.chars().all(|c: char| c.is_ascii_whitespace()) {
            return Ok(());
        }
        if gr.chars().all(|c| (c == '_') | c.is_ascii_alphabetic()) {
            assert!(self.curr_token.is_empty(), "{}", self.curr_token);
            self.state = LexState::Id;
            self.curr_token.push_str(gr);
            return Ok(());
        }
        if let Ok(c) = char::from_str(gr) {
            match c {
                '"' => self.state = LexState::LitStr,
                '#' => self.state = LexState::Include,
                '=' => self.tokens.push(TokenType::Eq),
                ':' => todo!("colon-equal state where?"),
                ';' => self.tokens.push(TokenType::Semicolon),
                '0'..='9' => {
                    self.curr_token.push(c);
                    self.state = LexState::Int;
                }
                '.' => {
                    self.curr_token.push(c);
                    self.state = LexState::Float;
                }
                _ => return Err(LexError::InvalidToken(String::from(c))),
            }
        }

        Ok(())
    }

    fn lex_id(&mut self, gr: &str) -> Result<(), LexError> {
        if gr.chars().all(|c| c.is_ascii_whitespace()) {
            self.state = LexState::Init;
            if let Some(kw) = Self::check_for_keyword(&self.curr_token) {
                self.tokens.push(kw);
                self.curr_token.clear();
            } else {
                self.tokens
                    .push(TokenType::Id(String::from_iter(self.curr_token.drain(..))));
            }
            return Ok(());
        }
        if gr.chars().all(|c| (c == '_') | c.is_ascii_alphanumeric()) {
            assert!(!self.curr_token.is_empty());
            self.curr_token.push_str(gr);
            return Ok(());
        }
        // otherwise, defer to Init
        self.state = LexState::Init;
        self.lex_init(gr)
    }

    fn lex_int(&mut self, gr: &str) -> Result<(), LexError> {
        let c = match char::from_str(gr) {
            Ok(c) => Ok(c),
            Err(e) => Err(LexError::InvalidToken(gr.into())),
        }?;

        if c.is_ascii_whitespace() {
            self.state = LexState::Init;
            self.tokens.push(TokenType::Int(
                u64::from_str(&self.curr_token).map_err(|e| LexError::ParseInt(e))?,
            ));
            self.curr_token.clear();
            return Ok(());
        }
        if !(c.is_digit(10) || matches!(c, '.' | '_' | 'e')) {
            return Err(LexError::InvalidToken(gr.into()));
        }

        match c {
            '0'..='9' => {
                self.curr_token.push(c);
            }
            '.' => {
                self.state = LexState::Float;
                self.curr_token.push(c);
            }
            '_' => {
                // Discard
            }
            'e' => {
                self.state = LexState::IntExponent;
                self.curr_token.push(c);
            }
            _ => unreachable!("Not 0-9, . or e: {c}"),
        }

        Ok(())
    }

    fn lex_float(&mut self, gr: &str) -> Result<(), LexError> {
        let c = match char::from_str(gr) {
            Ok(c) => Ok(c),
            Err(e) => Err(LexError::InvalidToken(gr.into())),
        }?;

        if c.is_ascii_whitespace() {
            self.state = LexState::Init;
            self.tokens.push(TokenType::Float(
                f64::from_str(&self.curr_token).map_err(|e| LexError::ParseFloat(e))?,
            ));
            self.curr_token.clear();
            return Ok(());
        }
        if !(c.is_digit(10) || matches!(c, '_' | 'e')) {
            return Err(LexError::InvalidToken(gr.into()));
        }
        match c {
            '0'..='9' => {
                self.curr_token.push(c);
            }
            '_' => {
                // Discard
            }
            'e' => {
                self.state = LexState::FloatExponent;
                self.curr_token.push(c);
            }
            _ => unreachable!("Not 0-9, . or e: {c}"),
        }

        Ok(())
    }

    fn lex_int_exponent(&mut self, gr: &str) -> Result<(), LexError> {
        let c = match char::from_str(gr) {
            Ok(c) => Ok(c),
            Err(e) => Err(LexError::InvalidToken(gr.into())),
        }?;

        if c.is_ascii_whitespace() {
            assert!(!self.curr_token.is_empty());
            // no digits after 'e'
            if self.curr_token.chars().rev().next().unwrap() == 'e' {
                return Err(LexError::InvalidToken(std::mem::take(&mut self.curr_token)));
            }

            self.state = LexState::Init;
            self.tokens
                .push(TokenType::Int(Self::int_from_str(&self.curr_token)?));
            self.curr_token.clear();
            return Ok(());
        }

        match c {
            '0'..='9' => {
                self.curr_token.push(c);
            }
            '_' => {
                // discard
            }
            _ => {
                return Err(LexError::InvalidToken(c.into()));
            }
        }

        Ok(())
    }

    fn lex_float_exponent(&mut self, gr: &str) -> Result<(), LexError> {
        let c = match char::from_str(gr) {
            Ok(c) => Ok(c),
            Err(e) => Err(LexError::InvalidToken(gr.into())),
        }?;

        if c.is_ascii_whitespace() {
            assert!(!self.curr_token.is_empty());
            // no digits after 'e'
            if self.curr_token.chars().rev().next().unwrap() == 'e' {
                return Err(LexError::InvalidToken(std::mem::take(&mut self.curr_token)));
            }

            self.state = LexState::Init;
            self.tokens.push(TokenType::Float(
                f64::from_str(&self.curr_token).map_err(|e| LexError::ParseFloat(e))?,
            ));
            self.curr_token.clear();
            return Ok(());
        }

        match c {
            '0'..='9' => {
                self.curr_token.push(c);
            }
            '_' => {
                // discard
            }
            _ => {
                return Err(LexError::InvalidToken(c.into()));
            }
        }

        Ok(())
    }

    fn lex_include(&mut self, gr: &str) -> Result<(), LexError> {
        if gr.chars().all(|c| c.is_ascii_whitespace()) {
            self.state = LexState::Init;
            self.tokens.push(TokenType::Include(String::from_iter(
                self.curr_token.drain(..),
            )));
            return Ok(());
        }
        if gr.chars().all(|c| (c == '_') | c.is_ascii_alphanumeric()) {
            assert!(!self.curr_token.is_empty());
            self.curr_token.push_str(gr);
            return Ok(());
        }
        // otherwise, defer to Init
        self.state = LexState::Init;
        self.lex_init(gr)
    }

    fn lex_lit_str(&mut self, gr: &str) -> Result<(), LexError> {
        match char::from_str(gr) {
            Ok('"') => {
                self.state = LexState::Init;
                self.tokens.push(TokenType::LitStr(String::from_iter(
                    self.curr_token.drain(..),
                )));
                return Ok(());
            }
            Ok('\\') => {
                self.state = LexState::Escape;
                return Ok(());
            }
            _ => {}
        }
        self.curr_token.push_str(gr);
        Ok(())
    }

    fn lex_escape(&mut self, gr: &str) -> Result<(), LexError> {
        match char::from_str(gr) {
            Ok('"') => {
                self.curr_token.push('"');
            }
            Ok('\\') => {
                self.curr_token.push('\\');
            }
            _ => {
                self.curr_token.push('\\');
                self.curr_token.push_str(gr);
            }
        }

        self.state = LexState::LitStr;
        Ok(())
    }

    /// Either return a keyword token, None
    fn check_for_keyword(token: &str) -> Option<TokenType> {
        match token {
            "yield" => Some(TokenType::Yield),
            _ => None,
        }
    }

    fn int_from_str(s: &str) -> Result<u64, LexError> {
        let e_idx = s.find('e');

        if let Some(idx) = e_idx {
            let lhs = &s[..idx];
            let rhs = &s[idx + 1..];

            let lhs = u64::from_str(lhs).map_err(|e| LexError::ParseInt(e))?;
            let rhs = u64::from_str(rhs)
                .map_err(|e| LexError::ParseInt(e))
                .and_then(|pow| {
                    10u64
                        .checked_pow(pow as u32)
                        .ok_or_else(|| LexError::Overflow(s.into()))
                })?;
            if let Some(ret) = lhs.checked_mul(rhs) {
                Ok(ret)
            } else {
                Err(LexError::Overflow(s.into()))
            }
        } else {
            u64::from_str(s).map_err(|e| LexError::ParseInt(e))
        }
    }
}

impl Default for Lex {
    fn default() -> Self {
        Self {
            tokens: Vec::new(),
            state: LexState::Init,
            curr_token: String::new(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Id(String),
    LitStr(String),
    Int(u64),
    Float(f64),
    // like an Id, but starts with a pound (#)
    Include(String),
    // :=
    ColonEq,
    // =
    Eq,
    // ;
    Semicolon,
    // keywords
    // "yield"
    Yield,
    Eof,
}

enum LexState {
    Init,
    Id,
    Int,
    Float,
    IntExponent,
    FloatExponent,
    Include,
    LitStr,
    // During LitStr, meet a "\\"
    Escape,
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    InvalidToken(String),
    ParseInt(ParseIntError),
    ParseFloat(ParseFloatError),
    Overflow(String),
    UnclosedStr,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn simple_assignment() {
        let test_str = "hello = \"hello\";";
        let mut lex = Lex::new();
        assert_eq!(lex.lex(test_str), Ok(()));
        assert_eq!(
            lex.tokens,
            [
                TokenType::Id(String::from("hello")),
                TokenType::Eq,
                TokenType::LitStr(String::from("hello")),
                TokenType::Semicolon,
                TokenType::Eof,
            ]
        );
    }
    #[test]
    fn keyword_detection() {
        let test_str = "yield \"hello\";";
        let mut lex = Lex::new();
        assert_eq!(lex.lex(test_str), Ok(()));
        assert_eq!(
            lex.tokens,
            [
                TokenType::Yield,
                TokenType::LitStr(String::from("hello")),
                TokenType::Semicolon,
                TokenType::Eof,
            ]
        );
    }

    #[test]
    fn parse_nums() {
        let test_str = "123 12.3 12e3 12.3e4 0123456789 1_2_3 .456";
        let mut lex = Lex::new();
        assert_eq!(lex.lex(test_str), Ok(()));
        assert_eq!(
            lex.tokens,
            [
                TokenType::Int(123),
                TokenType::Float(12.3),
                TokenType::Int(12000),
                TokenType::Float(123000.0),
                TokenType::Int(123456789),
                TokenType::Int(123),
                TokenType::Float(0.456),
                TokenType::Eof,
            ]
        );
    }
}
