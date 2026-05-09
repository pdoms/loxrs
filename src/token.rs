use std::fmt::Display;

#[derive(PartialEq, Debug, Clone)]
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
            _ => return Err(()),
        };
        let mut pos = pos;
        pos.1 = pos.1.saturating_sub(1);
        Ok(Self {
            ty,
            lexeme: ch.to_string(),
            pos,
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
                "nil" => self.ty = TokenType::Nil,
                "and" => self.ty = TokenType::And,
                "or" => self.ty = TokenType::Or,
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

#[derive(Debug, PartialEq, Clone)]
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

impl Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenType::Semicolon => write!(f, "Semicolon"),
            TokenType::LeftParen => write!(f, "LeftParen"),
            TokenType::LeftCurly => write!(f, "LeftCurly"),
            TokenType::RightParen => write!(f, "RightParen"),
            TokenType::RightCurly => write!(f, "RightCurly"),
            TokenType::Dot => write!(f, "Dot"),
            TokenType::Comma => write!(f, "Comma"),
            TokenType::Eq => write!(f, "Eq"),
            TokenType::Plus => write!(f, "Plus"),
            TokenType::Minus => write!(f, "Minus"),
            TokenType::Star => write!(f, "Star"),
            TokenType::Slash => write!(f, "Slash"),
            TokenType::EqEq => write!(f, "EqEq"),
            TokenType::Greater => write!(f, "Greater"),
            TokenType::GreaterEq => write!(f, "GreaterEq"),
            TokenType::Less => write!(f, "Less"),
            TokenType::LessEq => write!(f, "LessEq"),
            TokenType::BangEq => write!(f, "BangEq"),
            TokenType::Bang => write!(f, "Bang"),
            TokenType::Number(num) => write!(f, "Number({})", num),
            TokenType::Identifier(i) => write!(f, "Identifier({})", i),
            TokenType::StringLit(s) => write!(f, "StringLit({})", s),
            TokenType::Var => write!(f, "Var"),
            TokenType::True => write!(f, "True"),
            TokenType::False => write!(f, "False"),
            TokenType::If => write!(f, "If"),
            TokenType::Print => write!(f, "Print"),
            TokenType::While => write!(f, "While"),
            TokenType::Nil => write!(f, "Nil"),
            TokenType::And => write!(f, "And"),
            TokenType::Or => write!(f, "Or"),
            TokenType::Else => write!(f, "Else"),
            TokenType::For => write!(f, "For"),
            TokenType::Fun => write!(f, "Fun"),
            TokenType::Return => write!(f, "Return"),
            TokenType::Class => write!(f, "Class"),
            TokenType::Super => write!(f, "Super"),
            TokenType::This => write!(f, "This"),
            TokenType::Eof => write!(f, "Eof"),
        }
    }
}

#[macro_export]
macro_rules! tok {
    ($t:expr, $l:expr, $p:expr) => {
        $crate::token::Token {
            ty: $t,
            lexeme: $l,
            pos: $p,
        }
    };
}
