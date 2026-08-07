#![allow(dead_code)]
#![allow(unused)]
use unicode_segmentation::UnicodeSegmentation;

/// Block lexer
///
/// The current implementation has quite bad debug output. The most we can output is, where the
/// start/end of the block that errs is.
struct Lex {
    tokens: Vec<TokenType>,
    state: LexState,
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
                LexState::LitStr => {
                    self.lex_lit_str(gr)?;
                }
            }
        }
        todo!()
    }

    fn lex_init(&mut self, gr: &str) -> Result<(), LexError> {
        todo!()
    }
    fn lex_id(&mut self, gr: &str) -> Result<(), LexError> {
        todo!()
    }
    fn lex_lit_str(&mut self, gr: &str) -> Result<(), LexError> {
        todo!()
    }
}

impl Default for Lex {
    fn default() -> Self {
        Self {
            tokens: Vec::new(),
            state: LexState::Init,
        }
    }
}

enum TokenType {
    Id(String),
    LitStr(String),
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
    LitStr,
}

enum LexError {
    Invalid,
}
