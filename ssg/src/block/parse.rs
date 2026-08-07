#![allow(dead_code)]
#![allow(unused)]
use super::lex::TokenType;

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
            Some(TokenType::Semicolon) => {},
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
            Some(TokenType::Include(s)) => return Ok(ExprKind::Include(s)),
            Some(TokenType::Int(i)) => return Ok(ExprKind::Int(i)),
            Some(TokenType::Float(f)) => return Ok(ExprKind::Float(f)),
            None => unreachable!(),
            Some(tok) => return Err(ParseError::Unexpected(tok)),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ExprKind {
    Id(String),
    LitStr(String),
    Include(String),
    Int(u64),
    Float(f64),
}

pub enum StmtKind {
    // an expr, followed by semicolon
    Expr(ExprKind),
    Yield(ExprKind),
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
            TokenType::Include("stdio.html".into()),
        ];
        let expect = [
            ExprKind::Id("hello".into()),
            ExprKind::LitStr("goodbye".into()),
            ExprKind::Int(69),
            ExprKind::Float(42.0),
            ExprKind::Include("stdio.html".into()),
        ];

        for (tok, expect) in tokens.into_iter().zip(expect.into_iter()) {
            let mut tokvec = vec![tok];
            let prim = Parse::parse_expr_primary(&mut tokvec).unwrap();
            assert_eq!(prim, expect);
        }
    }
}
