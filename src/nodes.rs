use std::fmt::Display;

use crate::{
    errors::{NUMERIC_OPERANDS_NEEDED_ERR, RuntimeError},
    token::TokenType,
};

pub enum Stmt {
    Print(Expr),
    Expression(Expr),
}

pub enum Expr {
    Literal(Lit),
    Unary {
        op: Op,
        right: Box<Expr>,
    },
    Binary {
        op: Op,
        right: Box<Expr>,
        left: Box<Expr>,
    },
    Grouping(Box<Expr>),
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{}", lit),
            Expr::Unary { op, right } => write!(f, "({} {})", op, right),
            Expr::Binary { op, right, left } => write!(f, "({} {} {})", left, op, right),
            Expr::Grouping(expr) => write!(f, "(group {})", expr),
        }
    }
}

#[derive(Clone)]
pub enum Lit {
    Number(f64),
    String(String),
    Bool(bool),
    Nil,
}

impl PartialEq for Lit {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Number(lhs), Self::Number(rhs)) => lhs == rhs,
            (Self::String(lhs), Self::String(rhs)) => lhs == rhs,
            (Self::Bool(lhs), Self::Bool(rhs)) => lhs == rhs,
            _ => core::mem::discriminant(self) == core::mem::discriminant(other),
        }
    }
}

impl Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Lit::Number(n) => write!(f, "{n}"),
            Lit::String(s) => write!(f, "{s}"),
            Lit::Bool(b) => write!(f, "{b}"),
            Lit::Nil => write!(f, "nil"),
        }
    }
}

impl Lit {
    #[inline]
    pub fn add(self, other: Lit) -> Result<Lit, RuntimeError> {
        match (self, other) {
            (Lit::Number(lhs), Lit::Number(rhs)) => Ok(Lit::Number(lhs + rhs)),
            (Lit::String(lhs), Lit::String(rhs)) => Ok(Lit::String(lhs + &rhs)),
            _ => Err(RuntimeError::TypeError {
                msg: "operands must be of same type and either string or numeric".to_string(),
            }),
        }
    }

    #[inline]
    pub fn sub(self, other: Lit) -> Result<Lit, RuntimeError> {
        match (self, other) {
            (Lit::Number(lhs), Lit::Number(rhs)) => Ok(Lit::Number(lhs - rhs)),
            _ => Err(RuntimeError::TypeError {
                msg: NUMERIC_OPERANDS_NEEDED_ERR.to_string(),
            }),
        }
    }

    #[inline]
    pub fn mul(self, other: Lit) -> Result<Lit, RuntimeError> {
        match (self, other) {
            (Lit::Number(lhs), Lit::Number(rhs)) => Ok(Lit::Number(lhs * rhs)),
            _ => Err(RuntimeError::TypeError {
                msg: NUMERIC_OPERANDS_NEEDED_ERR.to_string(),
            }),
        }
    }

    #[inline]
    pub fn div(self, other: Lit) -> Result<Lit, RuntimeError> {
        match (self, other) {
            (Lit::Number(lhs), Lit::Number(rhs)) => {
                if rhs == 0.0 {
                    return Err(RuntimeError::DivisionByZero);
                }
                Ok(Lit::Number(lhs / rhs))
            }
            _ => Err(RuntimeError::TypeError {
                msg: NUMERIC_OPERANDS_NEEDED_ERR.to_string(),
            }),
        }
    }

    #[inline]
    pub fn less(self, other: Lit) -> Result<Lit, RuntimeError> {
        match (self, other) {
            (Lit::Number(lhs), Lit::Number(rhs)) => Ok(Lit::Bool(lhs < rhs)),
            _ => Err(RuntimeError::TypeError {
                msg: NUMERIC_OPERANDS_NEEDED_ERR.to_string(),
            }),
        }
    }

    #[inline]
    pub fn less_eq(self, other: Lit) -> Result<Lit, RuntimeError> {
        match (self, other) {
            (Lit::Number(lhs), Lit::Number(rhs)) => Ok(Lit::Bool(lhs <= rhs)),
            _ => Err(RuntimeError::TypeError {
                msg: NUMERIC_OPERANDS_NEEDED_ERR.to_string(),
            }),
        }
    }

    #[inline]
    pub fn greater(self, other: Lit) -> Result<Lit, RuntimeError> {
        match (self, other) {
            (Lit::Number(lhs), Lit::Number(rhs)) => Ok(Lit::Bool(lhs > rhs)),
            _ => Err(RuntimeError::TypeError {
                msg: NUMERIC_OPERANDS_NEEDED_ERR.to_string(),
            }),
        }
    }

    #[inline]
    pub fn greater_eq(self, other: Lit) -> Result<Lit, RuntimeError> {
        match (self, other) {
            (Lit::Number(lhs), Lit::Number(rhs)) => Ok(Lit::Bool(lhs >= rhs)),
            _ => Err(RuntimeError::TypeError {
                msg: NUMERIC_OPERANDS_NEEDED_ERR.to_string(),
            }),
        }
    }
}

pub enum Op {
    Equal,
    NotEqual,
    LessThan,
    LessThanEqual,
    GreaterThan,
    GreaterThanEqual,
    Add,
    Sub,
    Mul,
    Div,
    Not,
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Equal => f.write_str("=="),
            Op::NotEqual => f.write_str("!="),
            Op::LessThan => f.write_str("="),
            Op::LessThanEqual => f.write_str("<="),
            Op::GreaterThan => f.write_str(">"),
            Op::GreaterThanEqual => f.write_str(">="),
            Op::Add => f.write_str("+"),
            Op::Sub => f.write_str("-"),
            Op::Mul => f.write_str("*"),
            Op::Div => f.write_str("/"),
            Op::Not => f.write_str("!"),
        }
    }
}

impl From<&TokenType> for Op {
    fn from(tok_ty: &TokenType) -> Self {
        match tok_ty {
            TokenType::Plus => Op::Add,
            TokenType::Minus => Op::Sub,
            TokenType::Star => Op::Mul,
            TokenType::Slash => Op::Div,
            TokenType::EqEq => Op::Equal,
            TokenType::Greater => Op::GreaterThan,
            TokenType::GreaterEq => Op::GreaterThanEqual,
            TokenType::Less => Op::LessThan,
            TokenType::LessEq => Op::LessThanEqual,
            TokenType::BangEq => Op::NotEqual,
            TokenType::Bang => Op::Not,
            _ => unimplemented!("attempted to convert {:?} to operator", tok_ty),
        }
    }
}
