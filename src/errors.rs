use std::{error::Error, fmt::Display};

use crate::{
    nodes::Unwind,
    token::{Token, TokenType},
};

pub const NUMERIC_OPERANDS_NEEDED_ERR: &str = "operands must be numeric types";

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

#[derive(Debug)]
pub enum ParserError {
    UnexpectedToken {
        expected: TokenType,
        got: TokenType,
        pos: (usize, usize),
    },
    UnknwonError {
        last_token: Token,
    },
    InvalidAssignmentTarget {
        pos: (usize, usize),
    },
    TooManyArguments {
        pos: (usize, usize),
    },
}
impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::UnexpectedToken { expected, got, pos } => write!(
                f,
                "{}:{} unexpected token. Expected: {} got: {}",
                pos.0, pos.1, expected, got
            ),
            ParserError::UnknwonError { last_token } => write!(
                f,
                "{}:{} unknown error. Last token: {}",
                last_token.pos.0, last_token.pos.1, last_token.ty
            ),
            ParserError::InvalidAssignmentTarget { pos } => {
                write!(f, "{}:{} invalid assigment target", pos.0, pos.1)
            }
            ParserError::TooManyArguments { pos } => {
                write!(
                    f,
                    "{}:{} this implementation only allows a maximum of 255 arguments to a function",
                    pos.0, pos.1
                )
            }
        }
    }
}
impl Error for ParserError {}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    TypeError { msg: String },
    InvalidOperator { msg: String },
    DivisionByZero,
    UndefinedVariable { var_name: String },
    Unwind(Unwind),
    ArityMismatch { expected: usize, got: usize },
    NotCallable((usize, usize)),
    Io { msg: String },
}
impl Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RuntimeError::TypeError { msg } => write!(f, "TypeError: {msg}"),
            RuntimeError::InvalidOperator { msg } => write!(f, "InvalidOperator: {msg}"),
            RuntimeError::DivisionByZero => write!(f, "DivisionByZero"),
            RuntimeError::UndefinedVariable { var_name } => {
                write!(f, "UndefinedVariable: '{var_name}'")
            }
            RuntimeError::Unwind(unwind) => {
                write!(f, "Unwind: '{unwind}'")
            }
            RuntimeError::ArityMismatch { expected, got } => {
                write!(f, "ArityMismatch - expectd: {expected}; got {got}")
            }
            RuntimeError::NotCallable(pos) => write!(
                f,
                "NotCallable: entity is not callable at ({}):({})",
                pos.0, pos.1
            ),
            RuntimeError::Io { msg } => write!(f, "IO: {msg}"),
        }
    }
}
impl Error for RuntimeError {}

#[derive(Debug, PartialEq)]
pub enum ResolveError {
    VariableInOwnInititalizer { name: String },
    ReturnOutsideFunction,
}

impl Display for ResolveError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ResolveError::VariableInOwnInititalizer { name } => {
                write!(f, "ResolverError: variable in onw inititalizer: {name}")
            }
            Self::ReturnOutsideFunction => write!(f, "ResolverError: return outside function"),
        }
    }
}

impl Error for ResolveError {}
