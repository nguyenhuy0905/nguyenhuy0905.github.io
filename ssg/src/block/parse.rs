#![allow(dead_code)]
#![allow(unused)]
use super::lex::Pos;
use super::lex::TokenType;
use std::cell::Cell;

#[derive(Default)]
pub struct Parse {
    stmts: Vec<StmtKind>,
    exprs: Vec<ExprKind>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum StmtKind {
    // expression-statement (e.g. `call_a_function();`), index to an element in `exprs`.
    Expr(usize),
    // yield-statement (e.g. `yield call_a_function();`), index to an element in `exprs`.
    Yield(usize),
}

#[derive(Debug, PartialEq)]
pub enum ExprKind {
    // `expr` indexes the `exprs` array.
    Unary { op: UnaryOp, expr: usize },
    Id(String),
    LitStr(String),
    Int(u64),
    Float(f64),
}

impl Parse {
    fn new() -> Self {
        Self::default()
    }

    /// `rev` the two vectors for use in the parser functions inside...
    pub fn parse(
        &mut self,
        mut tokens: Vec<TokenType>,
        mut positions: Vec<Pos>,
    ) -> Result<(), ParseError> {
        todo!()
    }

    fn parse_expr(
        &mut self,
        tokens: &mut Vec<TokenType>,
        positions: &mut Vec<Pos>,
    ) -> Result<usize, ParseError> {
        // Add more once we get more parse expr methods.
        self.parse_expr_primary(tokens, positions)
    }

    fn parse_expr_unary(
        &mut self,
        tokens: &mut Vec<TokenType>,
        positions: &mut Vec<Pos>,
    ) -> Result<usize, ParseError> {
        todo!()
    }

    fn parse_expr_primary(
        &mut self,
        tokens: &mut Vec<TokenType>,
        positions: &mut Vec<Pos>,
    ) -> Result<usize, ParseError> {
        match tokens.pop() {
            Some(TokenType::Id(s)) => {
                self.exprs.push(ExprKind::Id(s));
                return Ok(self.exprs.len());
            }
            Some(TokenType::LitStr(s)) => {
                self.exprs.push(ExprKind::LitStr(s));
                return Ok(self.exprs.len());
            }
            Some(TokenType::Int(i)) => {
                self.exprs.push(ExprKind::Int(i));
                return Ok(self.exprs.len());
            }
            Some(TokenType::Float(f)) => {
                self.exprs.push(ExprKind::Float(f));
                return Ok(self.exprs.len());
            }
            Some(TokenType::LParen) => {
                let expr = self.parse_expr(tokens, positions)?;
                match tokens.pop() {
                    Some(TokenType::RParen) => return Ok(expr),
                    None => {
                        return Err(ParseError {
                            kind: ParseErrorKind::ExpectToken,
                            pos: { positions.pop().expect("Empty positions??") },
                        });
                    }
                    Some(tok) => {
                        return Err(ParseError {
                            kind: ParseErrorKind::Unexpected {
                                expect: &[")"],
                                actual: tok,
                            },
                            pos: { positions.pop().expect("Empty positions??") },
                        });
                    }
                }
            }
            None => unreachable!(),
            Some(tok) => {
                return Err(ParseError {
                    kind: ParseErrorKind::Unexpected {
                        expect: &["expression"],
                        actual: tok,
                    },
                    pos: positions.pop().expect("Empty positions??"),
                });
            }
        }

        todo!()
    }
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    // "or" keyword
    Not,
    // '#' symbol
    Include,
}

#[derive(Debug)]
pub struct ParseError {
    kind: ParseErrorKind,
    pos: Pos,
}

#[derive(Debug)]
pub enum ParseErrorKind {
    Unexpected {
        expect: &'static [&'static str],
        actual: TokenType,
    },
    ExpectToken,
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn primary() {
        let tokens = [
            TokenType::Id("hello".into()),
            TokenType::LitStr("goodbye".into()),
            TokenType::Int(69),
            TokenType::Float(42.0),
        ];
        let positions = [
            Pos { line: 1, column: 1 },
            Pos { line: 1, column: 7 },
            Pos {
                line: 1,
                column: 15,
            },
            Pos {
                line: 1,
                column: 17,
            },
        ];
        let expect = [
            ExprKind::Id("hello".into()),
            ExprKind::LitStr("goodbye".into()),
            ExprKind::Int(69),
            ExprKind::Float(42.0),
        ];
        let mut parse = Parse::new();

        for (tok, pos) in tokens.into_iter().zip(positions.into_iter()) {
            let mut tokvec = vec![tok];
            let mut posvec = vec![pos];
            let prim = parse.parse_expr_primary(&mut tokvec, &mut posvec).unwrap();
        }
        assert_eq!(&parse.stmts, &[]);
        assert_eq!(&parse.exprs, &expect);
    }

    #[test]
    fn parens() {
        let mut tokens = vec![
            TokenType::LParen,
            TokenType::LitStr("hello".into()),
            TokenType::RParen,
        ];
        let mut poss = vec![
            Pos { line: 1, column: 1 },
            Pos { line: 1, column: 2 },
            Pos { line: 1, column: 7 },
        ];
        tokens.reverse();
        // let expect = ExprKind::LitStr("hello".into());
        let mut parse = Parse::default();
        parse.parse_expr_primary(&mut tokens, &mut poss).unwrap();
        assert_eq!(&parse.exprs, &[ExprKind::LitStr("hello".into())]);
    }
}
