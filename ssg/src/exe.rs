#![allow(dead_code)]
#![allow(unused)]
use std::{
    collections::HashMap,
    io::{BufRead, Read, Write},
    rc::Rc,
};

use unicode_segmentation::UnicodeSegmentation;

/// Lexes, parses, and executes blocks; copy-paste verbatim otherwise.
pub struct Runner {
    // input: BufReader<InR>,
    // output: BufWriter<OutR>,
    curr_block: String,
    scopes: Vec<Scope>,
    lex_state: LexState,
}

struct Scope {
    vars: HashMap<Rc<str>, Rc<str>>,
    parent: Option<usize>,
}

impl Default for Runner {
    fn default() -> Self {
        return Self {
            curr_block: String::new(),
            scopes: Vec::new(),
            lex_state: LexState::Init,
        };
    }
}

impl Runner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, input: impl Read, output: impl Write) {
        todo!()
    }

    fn run_scope(
        &mut self,
        curr_scope: usize,
        input: impl BufRead,
        mut output: impl Write,
    ) -> Result<(), std::io::Error> {
        for (row, line) in input.lines().enumerate() {
            for (col, gr) in line?.grapheme_indices(true) {
                match self.lex_state {
                    LexState::Init => self.lex_init(gr, &mut output)?,
                    _ => todo!(),
                }
            }
            todo!()
        }
        Ok(())
    }

    fn lex_init(&mut self, gr: &str, output: &mut impl Write) -> Result<(), std::io::Error> {
        match gr {
            "{" => {
                self.lex_state = LexState::FirstLBrace;
            }
            "\\" => {
                self.lex_state = LexState::Escape;
            }
            _ => {
                output.write_all(gr.as_bytes())?;
            }
        }
        Ok(())
    }
    fn lex_first_lbrace(
        &mut self,
        gr: &str,
        output: &mut impl Write,
    ) -> Result<(), std::io::Error> {
        match gr {
            "{" => {
                self.lex_state = LexState::InBlock;
            }
            "\\" => {
                output.write_all(b"{")?;
                self.lex_state = LexState::Escape;
            }
            _ => {
                output.write_all(b"{")?;
                output.write_all(gr.as_bytes())?;
            }
        }
        Ok(())
    }
    fn lex_in_block(&mut self, gr: &str, output: &mut impl Write) -> Result<(), std::io::Error> {
        match gr {
            "}" => {
                self.lex_state = LexState::FirstRBrace;
            }
            _ => todo!()
        }
        todo!()
    }
    fn lex_first_rbrace(
        &mut self,
        gr: &str,
        output: &mut impl Write,
    ) -> Result<(), std::io::Error> {
        todo!()
    }
    fn lex_escape(&mut self, gr: &str, output: &mut impl Write) -> Result<(), std::io::Error> {
        match gr {
            "\\" => {
                output.write_all(b"\\")?;
            }
            "\"" => {
                output.write_all(b"\"")?;
            }
            // technically, the only thing really matters to escape is the right-brace
            "{" => {
                output.write_all(b"{")?;
            }"}" => {
                output.write_all(b"}")?;
            }
            _ => {
                output.write_all(b"\\")?;
                output.write_all(gr.as_bytes())?;
            }
        }
        Ok(())
    }
    fn lex_in_string(&mut self, gr: &str, output: &mut impl Write) -> Result<(), std::io::Error> {
        todo!()
    }
}

/// Think of the lexer as a state machine.
enum LexState {
    // No block funsies
    Init,
    // Receive one "{"
    FirstLBrace,
    // Receive second "{"
    InBlock,
    // Receive one "}"
    FirstRBrace,
    // Second "}" to get out of block
    // Receive a "\\" while in block
    Escape,
    // In block, receive a "\\"
    InBlockEscape,
    // In block, receive a "\""
    InString,
    // In string, receive a "\\"
    EscapeInString,
}
