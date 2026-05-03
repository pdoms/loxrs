use std::{error::Error, fmt::Display};

#[derive(Debug)]
pub enum ScanError {
    UnexpectedCharacter { ch: char, pos: (usize, usize) },
    UnterminatedStringLiteral { pos: (usize, usize) },
    InvalidNumber { lexeme: String, pos: (usize, usize) },
}

impl Error for ScanError {}

impl Display for ScanError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ScanError::UnexpectedCharacter { ch, pos } => {
                write!(f, "{}:{} unexpected character '{}'", pos.0, pos.1, ch)
            }
            ScanError::UnterminatedStringLiteral { pos } => {
                write!(f, "{}:{} unterminated string literal", pos.0, pos.1)
            }
            ScanError::InvalidNumber { lexeme, pos } => {
                write!(f, "{}:{} invalid number '{}'", pos.0, pos.1, lexeme)
            }
        }
    }
}
