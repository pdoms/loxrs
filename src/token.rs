#[derive(PartialEq, Debug)]
pub struct Token {
    /// type of the token
    pub ty: TokenType,
    /// the raw text representation as parsed
    pub lexeme: String,
    // a tuple containeing line number and offset
    pub pos: (usize, usize),
}


impl Token {
    /// takes a [`TokenType`], lexeme and postion and constructs a new [`Token`]
    /// The pos provided is the current position of the parser, hence, 
    /// the length of the lexeme is subtracted to get the beginning of the 
    /// lexeme in the source code
    pub fn new_do_offset(ty: TokenType, lexeme: String, pos: (usize, usize)) -> Self {
        let lex_len = lexeme.len();
        let mut pos = pos;
        pos.1 = pos.1.saturating_sub(lex_len);
        Self { ty, lexeme, pos }
    }
    
    /// shorthand constructuctor for tokens that can be represented by one char
    /// Returns the constructed token on success. (Calculates postion like
    /// new_do_offset.
    pub fn new_from_char(ch: char, pos: (usize, usize)) -> Result<Self, ()> {
        let ty = match ch {
            ';' => TokenType::Semicolon,
            '(' => TokenType::LeftParen,
            ')' => TokenType::RightParen,
            '{' => TokenType::LeftCurly,
            '}' => TokenType::RightCurly,
            '-' => TokenType::Minus,
            '+' => TokenType::Plus,
            '*' => TokenType::Star,
            '/' => TokenType::Slash,
            '!' => TokenType::Bang,
            '=' => TokenType::Eq,
            ',' => TokenType::Comma,
            '.' => TokenType::Dot,
            _ => return Err(())
        };
        let mut pos = pos;
        pos.1 = pos.1.saturating_sub(1);
        Ok(Self {
            ty,
            lexeme: ch.to_string(),
            pos
        })
    }
    /// checks if the parsed identifier matches a
    /// keyword and changes the type of the token 
    /// if that is true. Otherwise leaves 
    /// token untouched
    pub fn check_identifier_is_keyword(&mut self) {
        if let TokenType::Identifier(_) = self.ty {
            match self.lexeme.as_str() {
                "var" => self.ty = TokenType::Var,
                "true" => self.ty = TokenType::True,
                "false" => self.ty = TokenType::False,
                "if" => self.ty = TokenType::If,
                "print" => self.ty = TokenType::Print,
                "while" => self.ty = TokenType::While,
                "nil"  => self.ty = TokenType::Nil,
                "and" => self.ty = TokenType::And,
                "or"    => self.ty = TokenType::Or,
                "else" => self.ty = TokenType::Else, 
                "for" => self.ty = TokenType::For,
                "fun" => self.ty = TokenType::Fun,
                "return" => self.ty = TokenType::Return,
                "class" => self.ty = TokenType::Class,
                "super" => self.ty = TokenType::Super,
                "this" => self.ty = TokenType::This,
                _ => {}
            }
        }
    }
}


#[derive(Debug, PartialEq)]
pub enum TokenType {
    // Punctuations
    Semicolon,
    LeftParen,
    LeftCurly,
    RightParen,
    RightCurly,
    Dot,
    Comma,
    // Ops
    Eq,
    Plus,
    Minus,
    Star,
    Slash,
    EqEq,
    Greater,
    GreaterEq,
    Less,
    LessEq,
    BangEq,
    Bang,

    // Literals
    /// Lox only has double-precision floats
    Number(f64),
    Identifier(String),
    StringLit(String),

    // Keywords
    Var,
    True,
    False,
    If,
    Print,
    While,
    Nil,

    And,
    Or,
    Else, 
    For,
    Fun,
    Return,
    Class,
    Super,
    This,

    //EOF
    Eof,
}



#[macro_export]
macro_rules! tok {
    ($t:expr, $l:expr, $p:expr) => {
        crate::token::Token {
            ty: $t,
            lexeme: $l,
            pos: $p
        }
    }
}

pub fn dbg_print_tokens_seq(tokens: &[Token]) {
    for t in tokens {
        println!("{:?}", t.ty);
    }
}

