#![allow(dead_code)]
#![allow(unused)]
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, BufWriter, Read, Write},
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

impl Scope {
    pub fn new_root() -> Self {
        Self {
            vars: HashMap::new(),
            parent: None,
        }
    }
}

impl Default for Runner {
    fn default() -> Self {
        return Self {
            curr_block: String::new(),
            scopes: vec![Scope::new_root()],
            lex_state: LexState::Init,
        };
    }
}

impl Runner {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&mut self, input: impl Read, output: impl Write) -> Result<(), RunnerError> {
        self.run_scope(0, BufReader::new(input), BufWriter::new(output))
    }

    fn run_scope(
        &mut self,
        curr_scope: usize,
        mut input: impl BufRead,
        mut output: impl Write,
    ) -> Result<(), RunnerError> {
        let mut row: usize = 0;
        let mut line = String::new();
        while input.read_line(&mut line).map_err(|e| RunnerError::Io(e))? > 0 {
            for gr in line.graphemes(true) {
                match self.lex_state {
                    LexState::Init => self.lex_init(gr, &mut output)?,
                    LexState::EnterBlock => self.lex_enter_block(gr, &mut output)?,
                    LexState::InBlock => self.lex_in_block(gr, &mut output)?,
                    LexState::LeaveBlock => self.lex_leave_block(gr, &mut output)?,
                    LexState::Escape => self.lex_escape(gr, &mut output)?,
                }
            }
            row = row + 1;
            line.clear();
        }
        if matches!(self.lex_state, LexState::InBlock) {
            return Err(RunnerError::UnclosedBlock { line: row });
        }
        Ok(())
    }

    fn lex_init(&mut self, gr: &str, output: &mut impl Write) -> Result<(), RunnerError> {
        match gr {
            "{" => {
                self.lex_state = LexState::EnterBlock;
            }
            "\\" => {
                self.lex_state = LexState::Escape;
            }
            _ => {
                Self::output_write_all(output, gr.as_bytes())?;
            }
        }
        Ok(())
    }
    fn lex_enter_block(&mut self, gr: &str, output: &mut impl Write) -> Result<(), RunnerError> {
        match gr {
            "{" => self.lex_state = LexState::InBlock,
            "\\" => {
                Self::output_write_all(output, b"{")?;
                self.lex_state = LexState::Escape;
            }
            _ => {
                Self::output_write_all(output, b"{")?;
                Self::output_write_all(output, gr.as_bytes())?;
                self.lex_state = LexState::Init;
            }
        }
        Ok(())
    }
    fn lex_escape(&mut self, gr: &str, output: &mut impl Write) -> Result<(), RunnerError> {
        match gr {
            "\\" => {
                Self::output_write_all(output, b"\\")?;
            }
            "{" => {
                Self::output_write_all(output, b"{")?;
            }
            // technically this one is not needed, but it's just for symmetry with the escape above
            "}" => {
                Self::output_write_all(output, b"}")?;
            }
            _ => {
                Self::output_write_all(output, b"\\")?;
                Self::output_write_all(output, gr.as_bytes())?;
            }
        }
        self.lex_state = LexState::Init;
        Ok(())
    }
    fn lex_in_block(&mut self, gr: &str, output: &mut impl Write) -> Result<(), RunnerError> {
        match gr {
            "}" => {
                self.lex_state = LexState::LeaveBlock;
            }
            _ => {
                self.curr_block.push_str(gr);
            }
        }
        Ok(())
    }
    fn lex_leave_block(&mut self, gr: &str, output: &mut impl Write) -> Result<(), RunnerError> {
        match gr {
            "}" => {
                self.lex_state = LexState::Init;
                todo!("Execute block:\n{}", self.curr_block);
                self.curr_block.clear();
            }
            "\\" => {
                Self::output_write_all(output, b"}")?;
                self.lex_state = LexState::InBlock;
            }
            _ => {
                Self::output_write_all(output, b"}")?;
                Self::output_write_all(output, gr.as_bytes())?;
                self.lex_state = LexState::InBlock;
            }
        }
        Ok(())
    }

    fn output_write_all(output: &mut impl Write, value: &[u8]) -> Result<(), RunnerError> {
        output.write_all(value).map_err(|e| RunnerError::Io(e))
    }
}

#[derive(Debug)]
pub enum RunnerError {
    Io(std::io::Error),
    UnclosedBlock { line: usize },
}

/// Think of the lexer as a state machine.
enum LexState {
    // No block funsies
    Init,
    // Receive first "{"
    EnterBlock,
    // Receive second "{"
    InBlock,
    // Receive first "}"
    LeaveBlock,
    // Basically, "\{" or "\}" or "\\".
    Escape,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_literal() {
        let mut exe = Runner::new();
        let mut out: Vec<u8> = Vec::new();
        exe.run("<h1>yield hello;</h1>".as_bytes(), &mut out);
        assert_eq!("<h1>yield hello;</h1>", String::from_utf8(out).unwrap());
    }
    #[test]
    fn test_escape() {
        let mut exe = Runner::new();
        let mut out: Vec<u8> = Vec::new();
        exe.run("<h1>{\\{yield hello;}\\}</h1>".as_bytes(), &mut out);
        assert_eq!("<h1>{{yield hello;}}</h1>", String::from_utf8(out).unwrap());
    }
    #[test]
    fn test_unclosed_block() {
        let mut exe = Runner::new();
        let mut out: Vec<u8> = Vec::new();
        let ret = exe.run("<h1>{{yield hello;}</h1>".as_bytes(), &mut out);
        if !matches!(ret, Err(RunnerError::UnclosedBlock { line: 1 })) {
            panic!("{ret:?} does not error or does not match error")
        }
        // assert_matches!(ret, Err(RunnerError::UnclosedBlock {line: 0}));
    }
    // there's not much else to test at the moment
}
