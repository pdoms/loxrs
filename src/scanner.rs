use crate::{
    errors::ScanError,
    tok,
    token::{Token, TokenType},
};

const DEFAULT_LEXEME_CAP: usize = 64;

/// Scans the provided lox code and creates
/// a [`Token`] vec.
pub struct Scanner<'i> {
    /// the raw input given to the scanner
    input: &'i [u8],
    /// the line, column postion we are currently at
    pos: (usize, usize),
    /// the list of parsed tokens
    pub tokens: Vec<Token>,
    // the current lexeme
    lexeme: String,
}

/// scans and tokenizes the input. Input is consumed for now,
/// but this might change (TODO) for better error reporting.
/// We prase everything into a [`String`]. This is ok for now,
/// but using a proper buffer for this could more memory efficient
impl<'i> Scanner<'i> {
    pub fn new(input: &'i [u8]) -> Self {
        Self {
            input,
            pos: (0, 0),
            tokens: Vec::new(),
            lexeme: String::with_capacity(DEFAULT_LEXEME_CAP),
        }
    }

    pub fn parse(&mut self) -> Result<(), Vec<ScanError>> {
        let mut errors: Vec<ScanError> = Vec::new();

        while let Some(ch) = self.peek() {
            match ch {
                ' ' | '\t' => {
                    self.advance();
                }
                '\n' => {
                    self.advance();
                    self.pos.0 += 1;
                    self.pos.1 = 0;
                }
                ';' | '(' | ')' | '{' | '}' | '-' | '+' | '*' | '.' | ',' => {
                    self.advance();
                    match Token::new_from_char(ch, self.pos) {
                        Ok(t) => self.tokens.push(t),
                        Err(_) => unreachable!("implmentation error: can't create token from {ch}"),
                    }
                }
                '>' => {
                    // peek next for greater equal
                    if let Some('=') = self.peek_next() {
                        self.advance();
                        self.advance();
                        self.tokens.push(Token::new_do_offset(
                            TokenType::GreaterEq,
                            ">=".to_string(),
                            self.pos,
                        ));
                    } else {
                        self.advance();
                        self.tokens.push(Token::new_do_offset(
                            TokenType::Greater,
                            ">".to_string(),
                            self.pos,
                        ));
                    }
                }
                '<' => {
                    // peek next for less equal
                    if let Some('=') = self.peek_next() {
                        self.advance();
                        self.advance();
                        self.tokens.push(Token::new_do_offset(
                            TokenType::LessEq,
                            "<=".to_string(),
                            self.pos,
                        ));
                    } else {
                        self.advance();
                        self.tokens.push(Token::new_do_offset(
                            TokenType::Less,
                            "<".to_string(),
                            self.pos,
                        ));
                    }
                }
                '/' => {
                    //peek next for comments
                    if let Some('/') = self.peek_next() {
                        self.advance();
                        self.advance();
                        while let Some(ch) = self.peek() {
                            match ch {
                                '\n' => break,
                                _ => self.advance(),
                            }
                        }
                    } else {
                        self.advance();
                        self.tokens.push(Token::new_do_offset(
                            TokenType::Slash,
                            String::from("/"),
                            self.pos,
                        ));
                    }
                }
                '=' => {
                    //peek next for equal equal
                    if let Some('=') = self.peek_next() {
                        self.advance();
                        self.advance();
                        self.tokens.push(Token::new_do_offset(
                            TokenType::EqEq,
                            String::from("=="),
                            self.pos,
                        ));
                    } else {
                        self.advance();
                        self.tokens.push(Token::new_do_offset(
                            TokenType::Eq,
                            String::from("="),
                            self.pos,
                        ));
                    }
                }
                '!' => {
                    //peek next for BangEq
                    if let Some('=') = self.peek_next() {
                        self.advance();
                        self.advance();
                        self.tokens.push(Token::new_do_offset(
                            TokenType::BangEq,
                            String::from("!="),
                            self.pos,
                        ));
                    } else {
                        self.advance();
                        self.tokens.push(Token::new_do_offset(
                            TokenType::Bang,
                            String::from("!"),
                            self.pos,
                        ));
                    }
                }
                '"' => {
                    self.advance();
                    if let Err(err) = self.parse_string_lit() {
                        errors.push(err);
                    }
                }
                '0'..='9' => {
                    //parse number
                    self.push_lexeme(ch);
                    self.advance();
                    if let Err(err) = self.parse_number() {
                        errors.push(err);
                    }
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    self.push_lexeme(ch);
                    self.advance();

                    // parse identifier
                    if let Err(err) = self.parse_identifier() {
                        errors.push(err);
                    }
                }
                _ => {
                    self.advance();
                    errors.push(ScanError::UnexpectedCharacter { ch, pos: self.pos });
                }
            }
        }

        self.tokens
            .push(tok!(TokenType::Eof, String::new(), self.pos));

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn parse_string_lit(&mut self) -> Result<(), ScanError> {
        //note: opening quotation mark is already consumed (but not on
        //lexeme stack)
        while let Some(ch) = self.peek() {
            match ch {
                '"' => {
                    // we done
                    self.tokens.push(Token::new_do_offset(
                        TokenType::StringLit(self.lexeme.clone()),
                        self.lexeme.clone(),
                        self.pos,
                    ));
                    self.advance();
                    self.lexeme.clear();
                    return Ok(());
                }
                _ => {
                    // whatever it is, we push it
                    self.advance();
                    self.push_lexeme(ch);
                }
            }
        }
        Err(ScanError::UnterminatedStringLiteral { pos: self.pos })
    }

    fn parse_number(&mut self) -> Result<(), ScanError> {
        while let Some(ch) = self.peek() {
            if (ch == '.' && self.peek_next().is_some_and(|c| c.is_ascii_digit()))
                || ch.is_ascii_digit()
            {
                self.advance();
                self.push_lexeme(ch);
            } else {
                break;
            }
        }
        if !self.lexeme.is_empty() {
            match self.lexeme.parse::<f64>() {
                Ok(v) => {
                    self.tokens.push(Token::new_do_offset(
                        TokenType::Number(v),
                        self.lexeme.clone(),
                        self.pos,
                    ));
                    self.lexeme.clear();
                }
                Err(_) => {
                    let lexeme = self.lexeme.clone();
                    self.lexeme.clear();
                    return Err(ScanError::InvalidNumber {
                        lexeme,
                        pos: self.pos,
                    });
                }
            }
        }
        Ok(())
    }

    fn parse_identifier(&mut self) -> Result<(), ScanError> {
        while let Some(ch) = self.peek() {
            match ch {
                'a'..='z' | 'A'..='Z' | '_' | '0'..='9' => {
                    self.advance();
                    self.lexeme.push(ch);
                }
                _ => {
                    break;
                }
            }
        }
        if !self.lexeme.is_empty() {
            let mut identifier = Token::new_do_offset(
                TokenType::Identifier(self.lexeme.clone()),
                self.lexeme.clone(),
                self.pos,
            );
            identifier.check_identifier_is_keyword();
            self.tokens.push(identifier);
            self.lexeme.clear();
        }
        Ok(())
    }

    /// return next character if there is one
    /// does not advance the parser
    fn peek(&self) -> Option<char> {
        if self.input.is_empty() {
            return None;
        }
        Some(self.input[0] as char)
    }
    fn peek_next(&self) -> Option<char> {
        if self.input.len() <= 1 {
            return None;
        }
        Some(self.input[1] as char)
    }

    // consumes the next character pushes it to
    // advances the col by 1
    fn advance(&mut self) {
        self.input = &self.input[1..];
        self.pos.1 += 1;
    }
    fn push_lexeme(&mut self, ch: char) {
        self.lexeme.push(ch);
    }
}

#[cfg(test)]
mod test {
    use crate::{scanner::Scanner, tok, token::TokenType};

    #[test]
    fn parse_numeric_assignment() {
        let case = b"var x = 5";
        let mut scanner = Scanner::new(case);
        assert!(scanner.parse().is_ok());
        let lex1 = "var".to_string();
        let lex2 = "x".to_string();
        let expected = [
            tok!(TokenType::Var, lex1, (0, 0)),
            tok!(TokenType::Identifier(lex2.to_string()), lex2, (0, 4)),
            tok!(TokenType::Eq, String::from("="), (0, 6)),
            tok!(TokenType::Number(5.0), String::from("5"), (0, 8)),
            tok!(TokenType::Eof, String::from(""), (0, 9)),
        ];
        assert_eq!(scanner.tokens.len(), expected.len(), "num tokens mismatch");
        for (i, exp) in expected.iter().enumerate() {
            let e_tok = &exp;
            let r_tok = &scanner.tokens[i];
            assert_eq!(e_tok.ty, r_tok.ty, "type mismatch at index {i}");
            assert_eq!(e_tok.pos, r_tok.pos, "pos mismatch at index {i}");
        }
    }

    #[test]
    fn parse_numeric_float_assignment() {
        let case = b"var pi = 3.14;";
        let mut scanner = Scanner::new(case);
        assert!(scanner.parse().is_ok());
        let lex1 = "var".to_string();
        let lex2 = "pi".to_string();
        #[allow(clippy::approx_constant)]
        let pi = 3.14;
        let expected = [
            tok!(TokenType::Var, lex1, (0, 0)),
            tok!(TokenType::Identifier(lex2.to_string()), lex2, (0, 4)),
            tok!(TokenType::Eq, String::from("="), (0, 7)),
            tok!(TokenType::Number(pi), String::from("3.14"), (0, 9)),
            tok!(TokenType::Semicolon, String::from(";"), (0, 13)),
            tok!(TokenType::Eof, String::from(""), (0, 14)),
        ];
        assert_eq!(scanner.tokens.len(), expected.len(), "num tokens mismatch");
        for (i, exp) in expected.iter().enumerate() {
            let e_tok = &exp;
            let r_tok = &scanner.tokens[i];
            assert_eq!(e_tok.ty, r_tok.ty, "type mismatch at index {i}");
            assert_eq!(e_tok.pos, r_tok.pos, "pos mismatch at index {i}");
        }
    }
    #[test]
    fn parse_string_literal_assignment() {
        let case = b"var x = \"foo\";";
        let mut scanner = Scanner::new(case);
        assert!(scanner.parse().is_ok());
        let lex1 = "var".to_string();
        let lex2 = "x".to_string();
        let expected = [
            tok!(TokenType::Var, lex1, (0, 0)),
            tok!(TokenType::Identifier(lex2.to_string()), lex2, (0, 4)),
            tok!(TokenType::Eq, String::from("="), (0, 6)),
            tok!(
                TokenType::StringLit("foo".to_string()),
                String::from("foo"),
                (0, 9)
            ),
            tok!(TokenType::Semicolon, String::from(";"), (0, 13)),
            tok!(TokenType::Eof, String::from(""), (0, 14)),
        ];
        assert_eq!(scanner.tokens.len(), expected.len(), "num tokens mismatch");
        for (i, exp) in expected.iter().enumerate() {
            let e_tok = &exp;
            let r_tok = &scanner.tokens[i];
            assert_eq!(e_tok.ty, r_tok.ty, "type mismatch at index {i}");
            assert_eq!(e_tok.pos, r_tok.pos, "pos mismatch at index {i}");
        }
    }

    #[test]
    fn parse_print() {
        let case = b"print \"hello world\";";
        let mut scanner = Scanner::new(case);
        assert!(scanner.parse().is_ok());
        let lex1 = "print".to_string();
        let expected = [
            tok!(TokenType::Print, lex1, (0, 0)),
            tok!(
                TokenType::StringLit("hello world".to_string()),
                String::from("hello world"),
                (0, 7)
            ),
            tok!(TokenType::Semicolon, String::from(";"), (0, 19)),
            tok!(TokenType::Eof, String::from(""), (0, 20)),
        ];
        assert_eq!(scanner.tokens.len(), expected.len(), "num tokens mismatch");
        for (i, exp) in expected.iter().enumerate() {
            let e_tok = &exp;
            let r_tok = &scanner.tokens[i];
            assert_eq!(e_tok.ty, r_tok.ty, "type mismatch at index {i}");
            assert_eq!(e_tok.pos, r_tok.pos, "pos mismatch at index {i}");
        }
    }

    #[test]
    fn parse_if_then() {
        let case = b"if (x == 10) {print x; }";
        let mut scanner = Scanner::new(case);
        assert!(scanner.parse().is_ok());
        let expected = [
            tok!(TokenType::If, "if".to_string(), (0, 0)),
            tok!(TokenType::LeftParen, "(".to_string(), (0, 3)),
            tok!(
                TokenType::Identifier("x".to_string()),
                String::from("x"),
                (0, 4)
            ),
            tok!(TokenType::EqEq, String::from("=="), (0, 6)),
            tok!(TokenType::Number(10.0), String::from("10"), (0, 9)),
            tok!(TokenType::RightParen, ")".to_string(), (0, 11)),
            tok!(TokenType::LeftCurly, "{".to_string(), (0, 13)),
            tok!(TokenType::Print, "print".to_string(), (0, 14)),
            tok!(
                TokenType::Identifier("x".to_string()),
                String::from("x"),
                (0, 20)
            ),
            tok!(TokenType::Semicolon, String::from(";"), (0, 21)),
            tok!(TokenType::RightCurly, "}".to_string(), (0, 23)),
            tok!(TokenType::Eof, String::from(""), (0, 24)),
        ];
        assert_eq!(scanner.tokens.len(), expected.len(), "num tokens mismatch");
        for (i, exp) in expected.iter().enumerate() {
            let e_tok = &exp;
            let r_tok = &scanner.tokens[i];
            assert_eq!(e_tok.ty, r_tok.ty, "type mismatch at index {i}");
            assert_eq!(e_tok.pos, r_tok.pos, "pos mismatch at index {i}");
        }
    }

    #[test]
    fn parse_while() {
        let case = b"while (x > 0) { x = x - 1; }";
        let mut scanner = Scanner::new(case);
        assert!(scanner.parse().is_ok());
        let expected = [
            tok!(TokenType::While, "while".to_string(), (0, 0)),
            tok!(TokenType::LeftParen, "(".to_string(), (0, 6)),
            tok!(
                TokenType::Identifier("x".to_string()),
                String::from("x"),
                (0, 7)
            ),
            tok!(TokenType::Greater, String::from(">"), (0, 9)),
            tok!(TokenType::Number(0.0), String::from("0"), (0, 11)),
            tok!(TokenType::RightParen, ")".to_string(), (0, 12)),
            tok!(TokenType::LeftCurly, "{".to_string(), (0, 14)),
            tok!(
                TokenType::Identifier("x".to_string()),
                "x".to_string(),
                (0, 16)
            ),
            tok!(TokenType::Eq, String::from("="), (0, 18)),
            tok!(
                TokenType::Identifier("x".to_string()),
                String::from("x"),
                (0, 20)
            ),
            tok!(TokenType::Minus, String::from("-"), (0, 22)),
            tok!(TokenType::Number(1.0), String::from("1"), (0, 24)),
            tok!(TokenType::Semicolon, String::from(";"), (0, 25)),
            tok!(TokenType::RightCurly, "}".to_string(), (0, 27)),
            tok!(TokenType::Eof, String::from(""), (0, 28)),
        ];
        assert_eq!(scanner.tokens.len(), expected.len(), "num tokens mismatch");
        for (i, exp) in expected.iter().enumerate() {
            let e_tok = &exp;
            let r_tok = &scanner.tokens[i];
            assert_eq!(e_tok.ty, r_tok.ty, "type mismatch at index {i}");
            assert_eq!(e_tok.pos, r_tok.pos, "pos mismatch at index {i}");
        }
    }

    #[test]
    fn parse_comment() {
        let case = b"// this is a comment\nvar y = 2;";

        let mut scanner = Scanner::new(case);
        assert!(scanner.parse().is_ok());
        let expected = [
            tok!(TokenType::Var, "var".to_string(), (1, 0)),
            tok!(
                TokenType::Identifier("y".to_string()),
                "y".to_string(),
                (1, 4)
            ),
            tok!(TokenType::Eq, String::from("="), (1, 6)),
            tok!(TokenType::Number(2.0), String::from("2"), (1, 8)),
            tok!(TokenType::Semicolon, String::from(";"), (1, 9)),
            tok!(TokenType::Eof, String::from(""), (1, 10)),
        ];
        assert_eq!(scanner.tokens.len(), expected.len(), "num tokens mismatch");
        for (i, exp) in expected.iter().enumerate() {
            let e_tok = &exp;
            let r_tok = &scanner.tokens[i];
            assert_eq!(e_tok.ty, r_tok.ty, "type mismatch at index {i}");
            assert_eq!(e_tok.pos, r_tok.pos, "pos mismatch at index {i}");
        }
    }
    #[test]
    fn two_char_ops() {
        let case = b"!= == <= >= < >";

        let mut scanner = Scanner::new(case);
        assert!(scanner.parse().is_ok());
        let expected = [
            tok!(TokenType::BangEq, "!=".to_string(), (0, 0)),
            tok!(TokenType::EqEq, "==".to_string(), (0, 3)),
            tok!(TokenType::LessEq, String::from("<="), (0, 6)),
            tok!(TokenType::GreaterEq, String::from(">="), (0, 9)),
            tok!(TokenType::Less, String::from("<"), (0, 12)),
            tok!(TokenType::Greater, String::from(">"), (0, 14)),
            tok!(TokenType::Eof, String::from(""), (0, 15)),
        ];
        assert_eq!(scanner.tokens.len(), expected.len(), "num tokens mismatch");
        for (i, exp) in expected.iter().enumerate() {
            let e_tok = &exp;
            let r_tok = &scanner.tokens[i];
            assert_eq!(e_tok.ty, r_tok.ty, "type mismatch at index {i}");
            assert_eq!(e_tok.pos, r_tok.pos, "pos mismatch at index {i}");
        }
    }

    #[test]
    fn parse_empty_string_assignment() {
        let case = b"var x = \"\";";
        let mut scanner = Scanner::new(case);
        assert!(scanner.parse().is_ok());
        let lex1 = "var".to_string();
        let lex2 = "x".to_string();
        let expected = [
            tok!(TokenType::Var, lex1, (0, 0)),
            tok!(TokenType::Identifier(lex2.to_string()), lex2, (0, 4)),
            tok!(TokenType::Eq, String::from("="), (0, 6)),
            tok!(
                TokenType::StringLit("".to_string()),
                String::from(""),
                (0, 9)
            ),
            tok!(TokenType::Semicolon, String::from(";"), (0, 10)),
            tok!(TokenType::Eof, String::from(""), (0, 11)),
        ];
        assert_eq!(scanner.tokens.len(), expected.len(), "num tokens mismatch");
        for (i, exp) in expected.iter().enumerate() {
            let e_tok = &exp;
            let r_tok = &scanner.tokens[i];
            assert_eq!(e_tok.ty, r_tok.ty, "type mismatch at index {i}");
            assert_eq!(e_tok.pos, r_tok.pos, "pos mismatch at index {i}");
        }
    }
}
