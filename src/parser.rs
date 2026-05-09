//! From https://craftinginterpreters.com/parsing-expressions.html
//! Chapter 6.1
//!
//! ```
//! expression     -> equality ;
//! equality       -> comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison     -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term           -> factor ( ( "-" | "+" ) factor )* ;
//! factor         -> unary ( ( "/" | "*" ) unary )* ;
//! unary          -> ( "!" | "-" ) unary
//!                | primary ;
//! primary        -> NUMBER | STRING | "true" | "false" | "nil"
//!                | "(" expression ")" ;
//! ```
//!
//!
//!
use crate::{
    errors::ParserError,
    nodes::{Expr, Lit, Op, Stmt},
    token::{Token, TokenType},
};

pub struct Parser<'t> {
    tokens: &'t [Token],
    cursor: usize,
}

impl<'t> Parser<'t> {
    pub fn new(tokens: &'t [Token]) -> Self {
        Self { tokens, cursor: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Vec<ParserError>> {
        let mut stmts = Vec::new();
        let mut errors = Vec::new();
        while !self.is_eof() {
            match self.statement() {
                Ok(stmt) => stmts.push(stmt),
                Err(err) => {
                    errors.push(err);
                    self.synchronize();
                }
            }
        }
        if errors.is_empty() {
            Ok(stmts)
        } else {
            Err(errors)
        }
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_token(&[TokenType::Print]) {
            return self.print_stmt();
        }
        self.expr_stmt()
    }

    fn print_stmt(&mut self) -> Result<Stmt, ParserError> {
        let value = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            ParserError::UnexpectedToken {
                expected: TokenType::Semicolon,
                got: self.tokens[self.cursor].ty.clone(),
                pos: self.tokens[self.cursor].pos,
            },
        );
        Ok(Stmt::Print(value))
    }

    fn expr_stmt(&mut self) -> Result<Stmt, ParserError> {
        let value = self.expression()?;
        self.consume(
            TokenType::Semicolon,
            ParserError::UnexpectedToken {
                expected: TokenType::Semicolon,
                got: self.tokens[self.cursor].ty.clone(),
                pos: self.tokens[self.cursor].pos,
            },
        );
        Ok(Stmt::Expression(value))
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.equality()
    }
    fn equality(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.comparison()?;
        while self.match_token(&[TokenType::BangEq, TokenType::EqEq]) {
            let op = Op::from(&self.tokens[self.cursor - 1].ty);
            let right = self.comparison()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }
    fn comparison(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.term()?;
        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEq,
            TokenType::Less,
            TokenType::LessEq,
        ]) {
            let op = Op::from(&self.tokens[self.cursor - 1].ty);
            let right = self.term()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let op = Op::from(&self.tokens[self.cursor - 1].ty);
            let right = self.factor()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn factor(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.unary()?;
        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let op = Op::from(&self.tokens[self.cursor - 1].ty);
            let right = self.unary()?;
            left = Expr::Binary {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Expr, ParserError> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let op = Op::from(&self.tokens[self.cursor - 1].ty);
            let right = self.unary()?;
            return Ok(Expr::Unary {
                op,
                right: Box::new(right),
            });
        }
        self.primary()
    }
    fn primary(&mut self) -> Result<Expr, ParserError> {
        if self.match_token(&[TokenType::False]) {
            return Ok(Expr::Literal(Lit::Bool(false)));
        }
        if self.match_token(&[TokenType::True]) {
            return Ok(Expr::Literal(Lit::Bool(true)));
        }
        if self.match_token(&[TokenType::Nil]) {
            return Ok(Expr::Literal(Lit::Nil));
        }
        if let Some(numeric) = self.numeric_maybe() {
            return Ok(Expr::Literal(Lit::Number(numeric)));
        }
        if let Some(data) = self.string_literal_maybe() {
            return Ok(Expr::Literal(Lit::String(data)));
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(
                TokenType::RightParen,
                ParserError::UnexpectedToken {
                    expected: TokenType::RightParen,
                    got: self.tokens[self.cursor].ty.clone(),
                    pos: self.tokens[self.cursor].pos,
                },
            )?;
            return Ok(Expr::Grouping(Box::new(expr)));
        }
        Err(ParserError::UnknwonError {
            last_token: self.tokens[self.cursor].clone(),
        })
    }

    fn peek(&self) -> &Token {
        // NOTE: we check is_eof before cursor gets incremented
        &self.tokens[self.cursor]
    }

    /// checks if the current token is of type [TokenType::Eof]
    fn is_eof(&self) -> bool {
        // NOTE: we check is_eof before cursor gets incremented
        self.tokens[self.cursor].ty == TokenType::Eof
    }

    fn advance(&mut self) -> &Token {
        if !self.is_eof() {
            self.cursor += 1;
        }
        &self.tokens[self.cursor - 1]
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for ty in types {
            if self.check(ty) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn string_literal_maybe(&mut self) -> Option<String> {
        if let TokenType::StringLit(data) = self.peek().ty.clone() {
            self.advance();
            return Some(data);
        }
        None
    }
    fn numeric_maybe(&mut self) -> Option<f64> {
        if let TokenType::Number(data) = self.peek().ty {
            self.advance();
            return Some(data);
        }
        None
    }
    fn check(&self, ty: &TokenType) -> bool {
        std::mem::discriminant(&self.peek().ty) == std::mem::discriminant(ty)
    }

    fn consume(&mut self, ty: TokenType, err: ParserError) -> Result<(), ParserError> {
        if self.check(&ty) {
            self.advance();
            return Ok(());
        }

        Err(err)
    }

    fn synchronize(&mut self) {
        use TokenType::*;
        self.advance();
        while !self.is_eof() {
            if self.tokens[self.cursor - 1].ty == Semicolon {
                return;
            }
            if matches!(
                self.peek().ty,
                Class | Fun | Var | If | While | For | Print | Return
            ) {
                return;
            }
            self.advance();
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{nodes::Stmt, parser::Parser, scanner::Scanner};

    #[test]
    fn ast_expressions() {
        let cases = vec![
            ("5 + 3 * 2", "(5 + (3 * 2))"),
            ("10 / 2 - 1", "((10 / 2) - 1)"),
            ("(5 + 3) * 2", "((group (5 + 3)) * 2)"),
            ("!true", "(! true)"),
            ("-5 + 3", "((- 5) + 3)"),
            ("1 == 1", "(1 == 1)"),
            ("5 != 3", "(5 != 3)"),
            ("5 > 3", "(5 > 3)"),
            ("\"hello\" == \"hello\"", "(hello == hello)"),
            ("true == false", "(true == false)"),
            ("nil", "nil"),
        ];

        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            let _ = scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let res = parser.parse().unwrap();
            if let Stmt::Expression(expr) = &res[0] {
                assert_eq!(expr.to_string(), exp.to_string());
            }
        }
    }
}
