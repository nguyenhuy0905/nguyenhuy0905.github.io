#![allow(dead_code)]
#![allow(unused)]
use std::str::FromStr;
use unicode_segmentation::UnicodeSegmentation;

/// Block lexer
///
/// The current implementation has quite bad debug output. The most we can output is, where the
/// start/end of the block that errs is.
pub struct Lex {
    tokens: Vec<TokenType>,
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
            LexState::Init => Ok(()),
            LexState::Id => {
                if let Some(kw) = Self::check_for_keyword(&self.curr_token) {
                    self.tokens.push(kw);
                } else {
                    self.tokens
                        .push(TokenType::Id(String::from_iter(self.curr_token.drain(..))));
                }
                Ok(())
            }
            LexState::Include => {
                self.tokens.push(TokenType::Include(String::from_iter(
                    self.curr_token.drain(..),
                )));
                Ok(())
            }
            LexState::LitStr | LexState::Escape => Err(LexError::UnclosedStr),
        }
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
}

enum LexState {
    Init,
    Id,
    Include,
    LitStr,
    // During LitStr, meet a "\\"
    Escape,
}

#[derive(Debug, PartialEq)]
pub enum LexError {
    InvalidToken(String),
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
                TokenType::Semicolon
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
                TokenType::Semicolon
            ]
        );
    }
}
