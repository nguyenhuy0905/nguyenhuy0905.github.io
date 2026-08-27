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
    pub(in crate::block) positions: Vec<Pos>,
    state: LexState,
    /// Only really used when ending the Comment state
    curr_token: String,
    curr_pos: Pos,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Pos {
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}:{}", self.line, self.column)
    }
}

impl Lex {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lex(&mut self, input: &str) -> Result<(), LexError> {
        let mut pos = Pos { line: 0, column: 0 };
        for (lnum, line) in input
            .lines()
            .enumerate()
            .map(|(lnum, line)| (lnum + 1, line))
        {
            if matches!(self.state, LexState::Comment) {
                self.state = LexState::Init;
            }
            for (cnum, gr) in line.grapheme_indices(true).map(|(cnum, gr)| (cnum + 1, gr)) {
                pos = Pos {
                    line: lnum,
                    column: cnum,
                };
                match self.state {
                    LexState::Init => {
                        self.lex_init(gr, pos)?;
                    }
                    LexState::Id => {
                        self.lex_id(gr, pos)?;
                    }
                    LexState::Int => {
                        self.lex_int(gr, pos)?;
                    }
                    LexState::Float => {
                        self.lex_float(gr, pos)?;
                    }
                    LexState::IntExponent => {
                        self.lex_int_exponent(gr, pos)?;
                    }
                    LexState::FloatExponent => {
                        self.lex_float_exponent(gr, pos)?;
                    }
                    LexState::LitStr => {
                        self.lex_lit_str(gr, pos)?;
                    }
                    LexState::Escape => {
                        self.lex_escape(gr, pos)?;
                    }
                    LexState::Less => self.lex_less(gr, pos)?,
                    LexState::Greater => self.lex_greater(gr, pos)?,
                    LexState::Colon => self.lex_colon(gr, pos)?,
                    LexState::Comment => {}
                }
            }
        }

        // not yet done...
        match self.state {
            LexState::Init | LexState::Comment => {}
            LexState::Id => {
                if let Some(kw) = Self::check_for_keyword(&self.curr_token) {
                    self.add_token(kw, self.curr_pos);
                } else {
                    let tok = TokenType::Id(String::from_iter(self.curr_token.drain(..)));
                    self.add_token(tok, self.curr_pos);
                }
            }
            LexState::Int | LexState::IntExponent => {
                println!("{}", self.curr_token);
                self.add_token(
                    TokenType::Int(Self::int_from_str(&self.curr_token, self.curr_pos).unwrap()),
                    self.curr_pos,
                );
            }
            LexState::Float | LexState::FloatExponent => {
                self.add_token(
                    TokenType::Float(f64::from_str(&self.curr_token).unwrap()),
                    self.curr_pos,
                );
            }
            LexState::LitStr | LexState::Escape => {
                return Err(LexError {
                    kind: LexErrorKind::UnclosedStr,
                    pos,
                });
            }
            LexState::Less => {
                self.add_token(TokenType::Less, pos);
            }
            LexState::Greater => {
                self.add_token(TokenType::Greater, pos);
            }
            LexState::Colon => {
                return Err(LexError {
                    kind: LexErrorKind::InvalidToken(String::from(":")),
                    pos,
                });
            }
        }
        self.add_token(TokenType::Eof, pos);
        Ok(())
    }

    fn lex_init(&mut self, gr: &str, pos: Pos) -> Result<(), LexError> {
        if gr.chars().all(|c: char| c.is_ascii_whitespace()) {
            return Ok(());
        }
        if gr.chars().all(|c| (c == '_') | c.is_ascii_alphabetic()) {
            assert!(self.curr_token.is_empty(), "{}", self.curr_token);
            self.state = LexState::Id;
            self.curr_token.push_str(gr);
            self.curr_pos = pos;
            return Ok(());
        }
        if let Ok(c) = char::from_str(gr) {
            match c {
                '"' => {
                    self.state = LexState::LitStr;
                    self.curr_pos = pos;
                }
                '=' => self.add_token(TokenType::Eq, pos),
                ':' => {
                    self.state = LexState::Colon;
                    self.curr_pos = pos;
                }
                ',' => self.add_token(TokenType::Comma, pos),
                ';' => self.add_token(TokenType::Semicolon, pos),
                '0'..='9' => {
                    self.curr_token.push(c);
                    self.state = LexState::Int;
                    self.curr_pos = pos;
                }
                '.' => {
                    self.curr_token.push(c);
                    self.state = LexState::Float;
                    self.curr_pos = pos;
                }
                '+' => self.add_token(TokenType::Plus, pos),
                '-' => self.add_token(TokenType::Minus, pos),
                '*' => self.add_token(TokenType::Star, pos),
                '/' => self.add_token(TokenType::Slash, pos),
                '(' => self.add_token(TokenType::LParen, pos),
                ')' => self.add_token(TokenType::RParen, pos),
                '{' => self.add_token(TokenType::LBrace, pos),
                '}' => self.add_token(TokenType::RBrace, pos),
                '[' => self.add_token(TokenType::LBrack, pos),
                ']' => self.add_token(TokenType::RBrack, pos),
                '<' => todo!("Add a less-than state, 'cuz <= exists"),
                '>' => todo!("Add a greater-than state, 'cuz >= exists"),
                '#' => self.add_token(TokenType::Pound, pos),
                '!' => {
                    self.state = LexState::Comment;
                }
                _ => {
                    return Err(LexError {
                        kind: LexErrorKind::InvalidToken(String::from(c)),
                        pos,
                    });
                }
            }
        }

        Ok(())
    }

    fn lex_id(&mut self, gr: &str, pos: Pos) -> Result<(), LexError> {
        if gr.chars().all(|c| (c == '_') | c.is_ascii_alphanumeric()) {
            assert!(!self.curr_token.is_empty());
            self.curr_token.push_str(gr);
            return Ok(());
        }

        // otherwise, defer to Init
        assert!(!self.curr_token.is_empty());
        match Self::check_for_keyword(&self.curr_token) {
            Some(kw) => {
                self.add_token(kw, self.curr_pos);
                self.curr_token.clear();
            }
            None => {
                let tok = TokenType::Id(String::from_iter(self.curr_token.drain(..)));
                self.add_token(tok, self.curr_pos);
            }
        }
        self.state = LexState::Init;
        self.lex_init(gr, pos)
    }

    fn lex_int(&mut self, gr: &str, pos: Pos) -> Result<(), LexError> {
        let c = match char::from_str(gr) {
            Ok(c) => Ok(c),
            Err(e) => Err(LexError {
                kind: LexErrorKind::InvalidToken(gr.into()),
                pos: self.curr_pos,
            }),
        }?;

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
            _ => {
                self.state = LexState::Init;
                self.add_token(
                    TokenType::Int(u64::from_str(&self.curr_token).map_err(|e| LexError {
                        kind: LexErrorKind::ParseInt(e),
                        pos: self.curr_pos,
                    })?),
                    self.curr_pos,
                );
                self.curr_token.clear();
                return self.lex_init(gr, pos);
            }
        }

        Ok(())
    }

    fn lex_float(&mut self, gr: &str, pos: Pos) -> Result<(), LexError> {
        let c = match char::from_str(gr) {
            Ok(c) => Ok(c),
            Err(e) => Err(LexError {
                kind: LexErrorKind::InvalidToken(gr.into()),
                pos: self.curr_pos,
            }),
        }?;

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
            _ => {
                self.state = LexState::Init;
                self.add_token(
                    TokenType::Float(f64::from_str(&self.curr_token).map_err(|e| LexError {
                        kind: LexErrorKind::ParseFloat(e),
                        pos: self.curr_pos,
                    })?),
                    self.curr_pos,
                );
                self.curr_token.clear();
                return self.lex_init(gr, pos);
                // return Ok(());
            }
        }

        Ok(())
    }

    fn lex_int_exponent(&mut self, gr: &str, pos: Pos) -> Result<(), LexError> {
        let c = match char::from_str(gr) {
            Ok(c) => Ok(c),
            Err(e) => Err(LexError {
                kind: LexErrorKind::InvalidToken(gr.into()),
                pos: self.curr_pos,
            }),
        }?;

        match c {
            '0'..='9' => {
                self.curr_token.push(c);
            }
            '_' => {
                // discard
            }
            _ => {
                assert!(!self.curr_token.is_empty());
                // no digits after 'e'
                if self.curr_token.chars().rev().next().unwrap() == 'e' {
                    return Err(LexError {
                        kind: LexErrorKind::InvalidToken(std::mem::take(&mut self.curr_token)),
                        pos,
                    });
                }

                self.state = LexState::Init;
                self.add_token(
                    TokenType::Int(Self::int_from_str(&self.curr_token, self.curr_pos)?),
                    self.curr_pos,
                );
                self.curr_token.clear();
                return self.lex_init(gr, self.curr_pos);
            }
        }

        Ok(())
    }

    fn lex_float_exponent(&mut self, gr: &str, pos: Pos) -> Result<(), LexError> {
        let c = match char::from_str(gr) {
            Ok(c) => Ok(c),
            Err(e) => Err(LexError {
                kind: LexErrorKind::InvalidToken(gr.into()),
                pos,
            }),
        }?;

        if c.is_ascii_whitespace() {
            assert!(!self.curr_token.is_empty());
            // no digits after 'e'
            if self.curr_token.chars().rev().next().unwrap() == 'e' {
                return Err(LexError {
                    kind: LexErrorKind::InvalidToken(std::mem::take(&mut self.curr_token)),
                    pos,
                });
            }

            self.state = LexState::Init;
            self.add_token(
                TokenType::Float(f64::from_str(&self.curr_token).map_err(|e| LexError {
                    kind: LexErrorKind::ParseFloat(e),
                    pos,
                })?),
                self.curr_pos,
            );
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
                return Err(LexError {
                    kind: LexErrorKind::InvalidToken(c.into()),
                    pos,
                });
            }
        }

        Ok(())
    }

    fn lex_lit_str(&mut self, gr: &str, pos: Pos) -> Result<(), LexError> {
        match char::from_str(gr) {
            Ok('"') => {
                self.state = LexState::Init;
                let litstr = String::from_iter(self.curr_token.drain(..));
                self.add_token(TokenType::LitStr(litstr), self.curr_pos);
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

    fn lex_escape(&mut self, gr: &str, pos: Pos) -> Result<(), LexError> {
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

    fn lex_less(&mut self, gr: &str, pos: Pos) -> Result<(), LexError> {
        match char::from_str(gr) {
            Ok('=') => {
                self.add_token(TokenType::LessEq, pos);
                self.state = LexState::Init;
                Ok(())
            }
            Ok('<') => {
                self.add_token(TokenType::LessLess, pos);
                self.state = LexState::Init;
                Ok(())
            }
            _ => {
                self.add_token(TokenType::Less, pos);
                self.state = LexState::Init;
                self.lex_init(gr, pos)
            }
        }
    }

    fn lex_greater(&mut self, gr: &str, pos: Pos) -> Result<(), LexError> {
        match char::from_str(gr) {
            Ok('=') => {
                self.add_token(TokenType::GreaterEq, self.curr_pos);
                self.state = LexState::Init;
                Ok(())
            }
            Ok('>') => {
                self.add_token(TokenType::GreaterGreater, self.curr_pos);
                self.state = LexState::Init;
                Ok(())
            }
            _ => {
                self.add_token(TokenType::Greater, self.curr_pos);
                self.state = LexState::Init;
                self.lex_init(gr, pos)
            }
        }
    }

    /// At the moment, can only be followed by "=". Even whitespaces are not valid.
    fn lex_colon(&mut self, gr: &str, pos: Pos) -> Result<(), LexError> {
        if gr != "=" {
            return Err(LexError {
                kind: LexErrorKind::InvalidToken(gr.into()),
                pos,
            });
        }
        self.add_token(TokenType::ColonEq, self.curr_pos);

        Ok(())
    }

    /// Either return a keyword token, None
    fn check_for_keyword(token: &str) -> Option<TokenType> {
        match token {
            "yield" => Some(TokenType::Yield),
            "and" => Some(TokenType::Yield),
            "or" => Some(TokenType::Yield),
            "not" => Some(TokenType::Yield),
            "if" => Some(TokenType::If),
            "else" => Some(TokenType::Else),
            "while" => Some(TokenType::While),
            _ => None,
        }
    }

    /// So, according to Rust's u64::from_str, 12e3 is not a valid integer.
    fn int_from_str(s: &str, pos: Pos) -> Result<u64, LexError> {
        let e_idx = s.find('e');

        if let Some(idx) = e_idx {
            let lhs = &s[..idx];
            let rhs = &s[idx + 1..];

            let lhs = u64::from_str(lhs).map_err(|e| LexError {
                kind: LexErrorKind::ParseInt(e),
                pos,
            })?;
            let rhs = u64::from_str(rhs)
                .map_err(|e| LexError {
                    kind: LexErrorKind::ParseInt(e),
                    pos,
                })
                .and_then(|pow| {
                    10u64.checked_pow(pow as u32).ok_or_else(|| LexError {
                        kind: LexErrorKind::Overflow(s.into()),
                        pos,
                    })
                })?;
            if let Some(ret) = lhs.checked_mul(rhs) {
                Ok(ret)
            } else {
                Err(LexError {
                    kind: LexErrorKind::Overflow(s.into()),
                    pos,
                })
            }
        } else {
            u64::from_str(s).map_err(|e| LexError {
                kind: LexErrorKind::ParseInt(e),
                pos,
            })
        }
    }

    fn add_token(&mut self, tok: TokenType, pos: Pos) {
        self.tokens.push(tok);
        self.positions.push(pos);
    }
}

impl Default for Lex {
    fn default() -> Self {
        Self {
            tokens: Vec::new(),
            positions: Vec::new(),
            state: LexState::Init,
            curr_token: String::new(),
            curr_pos: Pos { line: 0, column: 0 },
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum TokenType {
    Id(String),
    LitStr(String),
    Int(u64),
    Float(f64),
    /// symbols
    /// :=
    ColonEq,
    /// =
    Eq,
    /// ,
    Comma,
    /// ;
    Semicolon,
    /// +
    Plus,
    /// -
    Minus,
    /// *
    Star,
    /// /
    Slash,
    /// &
    Amper,
    /// |
    Bar,
    /// ^
    Caret,
    /// (
    LParen,
    /// )
    RParen,
    /// {
    LBrace,
    /// }
    RBrace,
    /// [
    LBrack,
    /// ]
    RBrack,
    /// <
    Less,
    /// <<
    LessLess,
    /// >
    Greater,
    /// >
    GreaterGreater,
    /// <=
    LessEq,
    /// >=
    GreaterEq,
    /// #
    Pound,
    // keywords
    /// "yield"
    Yield,
    /// "and"
    And,
    /// "or"
    Or,
    /// "not"
    Not,
    /// "if"
    If,
    /// "else"
    Else,
    /// "while"
    While,
    Eof,
}

enum LexState {
    Init,
    Id,
    Int,
    Float,
    IntExponent,
    FloatExponent,
    LitStr,
    Less,
    Greater,
    Colon,
    // During LitStr, meet a "\\"
    Escape,
    Comment,
}

#[derive(Debug, PartialEq)]
pub struct LexError {
    kind: LexErrorKind,
    pos: Pos,
}

#[derive(Debug, PartialEq)]
pub enum LexErrorKind {
    InvalidToken(String),
    ParseInt(ParseIntError),
    ParseFloat(ParseFloatError),
    Overflow(String),
    UnclosedStr,
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match &self.kind {
            LexErrorKind::InvalidToken(s) => write!(f, "{}: invalid token {s}", self.pos),
            LexErrorKind::ParseInt(e) => write!(f, "{}: int parsing, {e}", self.pos),
            LexErrorKind::ParseFloat(e) => write!(f, "{}: float parsing, {e}", self.pos),
            LexErrorKind::Overflow(s) => write!(f, "{}: numerical value overflow {s}", self.pos),
            LexErrorKind::UnclosedStr => write!(f, "{}: unclosed string", self.pos),
        }
    }
}

impl std::error::Error for LexError {}

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
        assert_eq!(
            lex.positions,
            [
                Pos { line: 1, column: 1 },
                Pos { line: 1, column: 7 },
                Pos { line: 1, column: 9 },
                Pos {
                    line: 1,
                    column: 16
                },
                Pos {
                    line: 1,
                    column: 16
                }
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
        assert_eq!(
            lex.positions,
            [
                Pos { line: 1, column: 1 },
                Pos { line: 1, column: 7 },
                Pos {
                    line: 1,
                    column: 14
                },
                Pos {
                    line: 1,
                    column: 14
                }
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
        assert_eq!(
            lex.positions,
            [
                Pos { line: 1, column: 1 },
                Pos { line: 1, column: 5 },
                Pos {
                    line: 1,
                    column: 10
                },
                Pos {
                    line: 1,
                    column: 15
                },
                Pos {
                    line: 1,
                    column: 22
                },
                Pos {
                    line: 1,
                    column: 33
                },
                Pos {
                    line: 1,
                    column: 39
                },
                Pos {
                    line: 1,
                    column: 42
                },
            ]
        );
    }
    #[test]
    fn parse_expr() {
        let test_str = "1.0+2.0";
        let mut lex = Lex::new();
        assert_eq!(lex.lex(test_str), Ok(()));
        assert_eq!(
            lex.tokens,
            [
                TokenType::Float(1.0),
                TokenType::Plus,
                TokenType::Float(2.0),
                TokenType::Eof,
            ]
        );
        assert_eq!(
            lex.positions,
            [
                Pos { line: 1, column: 1 },
                Pos { line: 1, column: 4 },
                Pos { line: 1, column: 5 },
                Pos { line: 1, column: 7 }
            ]
        );
    }
    #[test]
    fn comment() {
        let test_str = "! 1+2=3\n\"hello\"";
        let mut lex = Lex::new();
        assert_eq!(lex.lex(test_str), Ok(()));
        assert_eq!(
            lex.tokens,
            [TokenType::LitStr("hello".into()), TokenType::Eof]
        );
        assert_eq!(
            lex.positions,
            [Pos { line: 2, column: 1 }, Pos { line: 2, column: 7 }]
        );
    }
}
