//! From https://craftinginterpreters.com/parsing-expressions.html
//!
//! ==========================================================================
//! program         -> declaration* EOF ;
//! declaration     -> funDecl
//!                 | varDecl
//!                 | statement ;
//! funDecl         -> "fun" IDENTIFIER "(" params? ")" block ;
//! params          -> IDENTIFIER ("," IDENTIFIER)* ;
//! varDecl         -> "var" IDENTIFIER ("=" expression)? ";" ;
//! statement       -> exprStmt
//!                 | forStmt
//!                 | ifStmt
//!                 | printStmt
//!                 | whileStmt
//!                 | block
//!                 | returnStmt ;
//! exprStmt        -> expression ";"
//! forStmt         -> "for" "(" (varDecl | exprStmt | ";" )
//!                     expression? ";"
//!                     expression? ")" statement ;
//! ifStmt          -> "if" "(" expression ")" statement
//!                 ( "else" statement )? ;
//! printStmt       -> "print" expression ";" ;
//! whileStmt       -> "while" "(" expression ")" statement ;
//! block           -> "{" declaration* "}" ;
//! returnStmt      -> "return" expression? ";" ;
//! expression      -> assignment ;
//! assignment      -> IDENTIFIER "=" assignment
//!                 | logic_or ;
//! logic_or        -> logic_and ( "or" logic_and )* ;
//! logic_and       -> equality ( "and" equality )* ;
//! equality        -> comparison ( ( "!=" | "==" ) comparison )* ;
//! comparison      -> term ( ( ">" | ">=" | "<" | "<=" ) term )* ;
//! term            -> factor ( ( "-" | "+" ) factor )* ;
//! factor          -> unary ( ( "/" | "*" ) unary )* ;
//! unary           -> ( "!" | "-" ) unary
//!                 | call ;
//! call            -> primary ( "(" aruments? ")" ) ;
//! arguments       -> expression ( "," expression )* ;
//! primary         -> NUMBER | STRING | "true" | "false" | "nil"
//!                 | "(" expression ")" | IDENTIFIER ;
//! ==========================================================================
//!

use crate::{
    errors::ParserError,
    nodes::{Expr, Lit, Op, Stmt},
    token::{Token, TokenType},
};

const MAX_ARGS: usize = 255;

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
            match self.declaration() {
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

    fn declaration(&mut self) -> Result<Stmt, ParserError> {
        if self.match_token(&[TokenType::Fun]) {
            return self.fun_declaration();
        }

        if self.match_token(&[TokenType::Var]) {
            return self.var_declaration();
        }
        self.statement()
    }

    fn fun_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = match &self.peek().ty {
            TokenType::Identifier(n) => n.clone(),
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: TokenType::Identifier(String::new()),
                    got: self.peek().ty.clone(),
                    pos: self.peek().pos,
                });
            }
        };
        self.advance();

        self.consume_and_unexpected(TokenType::LeftParen)?;

        let mut params = vec![];

        if !self.check(&TokenType::RightParen) {
            loop {
                if params.len() >= MAX_ARGS {
                    return Err(ParserError::TooManyArguments {
                        pos: self.peek().pos,
                    });
                }
                match &self.peek().ty {
                    TokenType::Identifier(s) => params.push(s.clone()),
                    _ => {
                        return Err(ParserError::UnexpectedToken {
                            expected: TokenType::Identifier(String::new()),
                            got: self.peek().ty.clone(),
                            pos: self.peek().pos,
                        });
                    }
                }
                self.advance();
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume_and_unexpected(TokenType::RightParen)?;
        self.consume_and_unexpected(TokenType::LeftCurly)?;
        let body = match self.block()? {
            Stmt::Block(stmts) => stmts,
            _ => unreachable!(),
        };

        Ok(Stmt::Function { name, params, body })
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParserError> {
        let name = match &self.peek().ty {
            TokenType::Identifier(ident) => ident.clone(),
            _ => {
                return Err(ParserError::UnexpectedToken {
                    expected: TokenType::Identifier(String::new()),
                    got: self.peek().ty.clone(),
                    pos: self.peek().pos,
                });
            }
        };

        self.advance();

        let initializer = if self.match_token(&[TokenType::Eq]) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume_and_unexpected(TokenType::Semicolon)?;

        Ok(Stmt::Var { name, initializer })
    }

    fn statement(&mut self) -> Result<Stmt, ParserError> {
        if self.match_token(&[TokenType::Return]) {
            return self.return_statement();
        }
        if self.match_token(&[TokenType::Print]) {
            return self.print_stmt();
        }

        if self.match_token(&[TokenType::While]) {
            return self.while_statement();
        }

        if self.match_token(&[TokenType::For]) {
            return self.for_statement();
        }

        if self.match_token(&[TokenType::If]) {
            return self.if_statement();
        }

        if self.match_token(&[TokenType::LeftCurly]) {
            return self.block();
        }
        self.expr_stmt()
    }

    fn return_statement(&mut self) -> Result<Stmt, ParserError> {
        let value = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume_and_unexpected(TokenType::Semicolon)?;
        Ok(Stmt::Return { value })
    }

    fn for_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume_and_unexpected(TokenType::LeftParen)?;

        let initializer = if self.match_token(&[TokenType::Semicolon]) {
            None
        } else if self.match_token(&[TokenType::Var]) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expr_stmt()?)
        };

        let condition = if !self.check(&TokenType::Semicolon) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume_and_unexpected(TokenType::Semicolon)?;

        let increment = if !self.check(&TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume_and_unexpected(TokenType::RightParen)?;

        let mut body = self.statement()?;

        if let Some(inc) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(inc)])
        }

        body = Stmt::While {
            condition: condition.unwrap_or(Expr::Literal(Lit::Bool(true))),
            body: Box::new(body),
        };

        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body])
        }

        Ok(body)
    }

    fn while_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume_and_unexpected(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.consume_and_unexpected(TokenType::RightParen)?;
        let body = self.statement()?;
        Ok(Stmt::While {
            condition,
            body: Box::new(body),
        })
    }
    fn if_statement(&mut self) -> Result<Stmt, ParserError> {
        self.consume_and_unexpected(TokenType::LeftParen)?;
        let condition = self.expression()?;
        self.consume_and_unexpected(TokenType::RightParen)?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&[TokenType::Else]) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };
        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Stmt, ParserError> {
        let mut stmts = Vec::new();
        while !self.is_eof() && self.peek().ty != TokenType::RightCurly {
            stmts.push(self.declaration()?);
        }
        self.consume_and_unexpected(TokenType::RightCurly)?;
        Ok(Stmt::Block(stmts))
    }

    fn print_stmt(&mut self) -> Result<Stmt, ParserError> {
        let value = self.expression()?;
        self.consume_and_unexpected(TokenType::Semicolon)?;
        Ok(Stmt::Print(value))
    }

    fn expr_stmt(&mut self) -> Result<Stmt, ParserError> {
        let value = self.expression()?;
        self.consume_and_unexpected(TokenType::Semicolon)?;
        Ok(Stmt::Expression(value))
    }

    fn expression(&mut self) -> Result<Expr, ParserError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParserError> {
        let expr = self.or()?;

        if self.match_token(&[TokenType::Eq]) {
            let value = self.assignment()?; // right-associative
            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }
            return Err(ParserError::InvalidAssignmentTarget {
                pos: self.peek().pos,
            });
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.and()?;
        while self.match_token(&[TokenType::Or]) {
            let op = Op::from(&self.tokens[self.cursor - 1].ty);
            let right = self.and()?;
            left = Expr::Logical {
                left: Box::new(left),
                op,
                right: Box::new(right),
            }
        }
        Ok(left)
    }

    fn and(&mut self) -> Result<Expr, ParserError> {
        let mut left = self.equality()?;
        while self.match_token(&[TokenType::And]) {
            let op = Op::from(&self.tokens[self.cursor - 1].ty);
            let right = self.equality()?;
            left = Expr::Logical {
                left: Box::new(left),
                op,
                right: Box::new(right),
            }
        }
        Ok(left)
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
        self.call()
    }

    fn call(&mut self) -> Result<Expr, ParserError> {
        let mut expr = self.primary()?;
        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break Ok(expr);
            }
        }
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParserError> {
        let mut arguments = vec![];
        if !self.check(&TokenType::RightParen) {
            loop {
                if arguments.len() >= MAX_ARGS {
                    return Err(ParserError::TooManyArguments {
                        pos: self.peek().pos,
                    });
                }
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        let paren = self.peek().clone();
        self.consume_and_unexpected(TokenType::RightParen)?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            paren,
            arguments,
        })
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

        if let TokenType::Identifier(name) = &self.peek().ty.clone() {
            self.advance();
            return Ok(Expr::Variable(name.clone()));
        }

        if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume_and_unexpected(TokenType::RightParen)?;
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

    //    fn consume(&mut self, ty: TokenType, err: ParserError) -> Result<(), ParserError> {
    //        if self.check(&ty) {
    //            self.advance();
    //            return Ok(());
    //        }
    //
    //        Err(err)
    //    }

    fn consume_and_unexpected(&mut self, expected: TokenType) -> Result<(), ParserError> {
        if self.check(&expected) {
            self.advance();
            return Ok(());
        }

        Err(ParserError::UnexpectedToken {
            expected,
            got: self.peek().ty.clone(),
            pos: self.peek().pos,
        })
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
            ("5 + 3 * 2;", "(5 + (3 * 2))"),
            ("10 / 2 - 1;", "((10 / 2) - 1)"),
            ("(5 + 3) * 2;", "((group (5 + 3)) * 2)"),
            ("!true;", "(! true)"),
            ("-5 + 3;", "((- 5) + 3)"),
            ("1 == 1;", "(1 == 1)"),
            ("5 != 3;", "(5 != 3)"),
            ("5 > 3;", "(5 > 3)"),
            ("\"hello\" == \"hello\";", "(hello == hello)"),
            ("true == false;", "(true == false)"),
            ("nil;", "nil"),
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
