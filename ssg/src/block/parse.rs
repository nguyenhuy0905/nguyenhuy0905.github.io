#![allow(dead_code)]
#![allow(unused)]
use super::lex::TokenType;
use std::cell::Cell;

pub struct Parse {
    stmts: Vec<StmtKind>,
}

impl Default for Parse {
    fn default() -> Self {
        Self { stmts: Vec::new() }
    }
}

impl Parse {
    pub fn new() -> Self {
        Self::default()
    }

    /// `tokens` are in reverse order; that is, `tokens.pop` should return the first token.
    pub fn parse(&mut self, mut tokens: Vec<TokenType>) -> Result<(), ParseError> {
        tokens.reverse();
        while (tokens.iter().rev().next() != Some(&TokenType::Eof)) {
            self.stmts.push(Self::parse_stmt(&mut tokens)?);
        }

        Ok(())
    }

    fn parse_stmt(tokens: &mut Vec<TokenType>) -> Result<StmtKind, ParseError> {
        match tokens.pop() {
            None => unreachable!(),
            Some(TokenType::Eof) => Err(ParseError::ExpectToken),
            Some(TokenType::Yield) => Self::parse_yield(tokens),
            Some(tok) => {
                tokens.push(tok);
                Self::parse_stmt_expr(tokens)
            }
        }
        // todo!()
    }

    fn parse_yield(tokens: &mut Vec<TokenType>) -> Result<StmtKind, ParseError> {
        let yield_result = Self::parse_expr(tokens)?;
        match tokens.pop() {
            Some(TokenType::Semicolon) => {}
            Some(tok) => return Err(ParseError::Unexpected(tok)),
            None => return Err(ParseError::ExpectToken),
        }
        Ok(StmtKind::Yield(yield_result))
    }

    fn parse_stmt_expr(tokens: &mut Vec<TokenType>) -> Result<StmtKind, ParseError> {
        todo!()
    }

    fn parse_expr(tokens: &mut Vec<TokenType>) -> Result<ExprKind, ParseError> {
        // TODO: for now, there's only this expression...
        Self::parse_expr_primary(tokens)
    }

    fn parse_expr_primary(tokens: &mut Vec<TokenType>) -> Result<ExprKind, ParseError> {
        assert!(!tokens.is_empty());

        match tokens.pop() {
            Some(TokenType::Id(s)) => return Ok(ExprKind::Id(s)),
            Some(TokenType::LitStr(s)) => return Ok(ExprKind::LitStr(s)),
            Some(TokenType::Int(i)) => return Ok(ExprKind::Int(i)),
            Some(TokenType::Float(f)) => return Ok(ExprKind::Float(f)),
            Some(TokenType::LParen) => {
                let exp = Self::parse_expr(tokens)?;
                match tokens.pop() {
                    Some(TokenType::RParen) => return Ok(exp),
                    None => return Err(ParseError::ExpectToken),
                    Some(tok) => return Err(ParseError::Unexpected(tok)),
                }
            }
            None => unreachable!(),
            Some(tok) => return Err(ParseError::Unexpected(tok)),
        }
    }

    /// If `token` is an operator, returns a pair of numbers indicating the operator's precedence.
    /// Higher number means higher precedence.
    /// If the right-hand-side number is bigger, the operator is right-associated, and
    /// left-associated otherwise. There isn't really a case where the two numbers are equal.
    /// Unary operators will be (0, 1).
    fn pratt_order(token: &TokenType) -> Option<(usize, usize)> {
        todo!();
        match token {
            &TokenType::ColonEq | &TokenType::Eq => Some((0, 1)),
            // &TokenType::Plus | &TokenType::Minus => Some((2, 1)),
            // &TokenType::Star | &TokenType::Slash => Some((4, 3)),
            _ => None
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ExprKind {
    Unary(UnaryOp, Box<ExprKind>),
    Id(String),
    LitStr(String),
    Int(u64),
    Float(f64),
}

pub enum StmtKind {
    // an expr, followed by semicolon
    Expr(ExprKind),
    Yield(ExprKind),
}

#[derive(Debug, PartialEq)]
pub enum UnaryOp {
    // "or" keyword
    Or,
    // '#' symbol
    Include,
}

#[derive(Debug)]
pub enum ParseError {
    Unexpected(TokenType),
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
        let expect = [
            ExprKind::Id("hello".into()),
            ExprKind::LitStr("goodbye".into()),
            ExprKind::Int(69),
            ExprKind::Float(42.0),
        ];

        for (tok, expect) in tokens.into_iter().zip(expect.into_iter()) {
            let mut tokvec = vec![tok];
            let prim = Parse::parse_expr_primary(&mut tokvec).unwrap();
            assert_eq!(prim, expect);
        }
    }

    #[test]
    fn parens() {
        let mut tokens = vec![
            TokenType::LParen,
            TokenType::LitStr("hello".into()),
            TokenType::RParen,
        ];
        tokens.reverse();
        // let expect = ExprKind::LitStr("hello".into());
        let prim = Parse::parse_expr_primary(&mut tokens).unwrap();
        assert_eq!(prim, ExprKind::LitStr("hello".into()));
    }
}
