use std::{fmt::Display, rc::Rc};

use crate::{
    environment::Environment,
    errors::{NUMERIC_OPERANDS_NEEDED_ERR, RuntimeError},
    token::{Token, TokenType},
};

#[derive(Debug, Clone)]
pub enum Stmt {
    Print(Expr),
    Var {
        name: String,
        initializer: Option<Expr>,
    }, // initializer is optional
    While {
        condition: Expr,
        body: Box<Stmt>,
    },
    Block(Vec<Stmt>),
    If {
        condition: Expr,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Function {
        name: String,
        params: Vec<String>,
        body: Vec<Stmt>,
    },
    Return {
        value: Option<Expr>,
    },
    Expression(Expr),
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Lit),
    Logical {
        left: Box<Expr>,
        op: Op,
        right: Box<Expr>,
    },
    Unary {
        op: Op,
        right: Box<Expr>,
    },
    Binary {
        op: Op,
        right: Box<Expr>,
        left: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        paren: Token,
        arguments: Vec<Expr>,
    },
    Grouping(Box<Expr>),
    Variable(String),
    Assign {
        name: String,
        value: Box<Expr>,
    },
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Literal(lit) => write!(f, "{}", lit),
            Expr::Logical { op, right, left } => write!(f, "({} {} {})", left, op, right),
            Expr::Unary { op, right } => write!(f, "({} {})", op, right),
            Expr::Binary { op, right, left } => write!(f, "({} {} {})", left, op, right),
            Expr::Grouping(expr) => write!(f, "(group {})", expr),
            Expr::Variable(name) => write!(f, "<{}>", name),
            Expr::Assign { name, value } => write!(f, "<{}> = {}", name, value),
            Expr::Call {
                callee,
                paren: _,
                arguments,
            } => write!(
                f,
                "{}({})",
                callee,
                arguments
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub enum Lit {
    Number(f64),
    String(String),
    Bool(bool),
    #[default]
    Nil,
    Function(LoxFunction),
    NativeFunction(NativeFunction),
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
            Lit::Function(fun) => write!(f, "<fn {}>", fun.name),
            Lit::NativeFunction(fun) => write!(f, "<fn {}>", fun.name),
        }
    }
}

impl Lit {
    #[inline]
    pub fn lox_add(self, other: Lit) -> Result<Lit, RuntimeError> {
        match (self, other) {
            (Lit::Number(lhs), Lit::Number(rhs)) => Ok(Lit::Number(lhs + rhs)),
            (Lit::String(lhs), Lit::String(rhs)) => Ok(Lit::String(lhs + &rhs)),
            _ => Err(RuntimeError::TypeError {
                msg: "operands must be of same type and either string or numeric".to_string(),
            }),
        }
    }

    #[inline]
    pub fn lox_sub(self, other: Lit) -> Result<Lit, RuntimeError> {
        match (self, other) {
            (Lit::Number(lhs), Lit::Number(rhs)) => Ok(Lit::Number(lhs - rhs)),
            _ => Err(RuntimeError::TypeError {
                msg: NUMERIC_OPERANDS_NEEDED_ERR.to_string(),
            }),
        }
    }

    #[inline]
    pub fn lox_mul(self, other: Lit) -> Result<Lit, RuntimeError> {
        match (self, other) {
            (Lit::Number(lhs), Lit::Number(rhs)) => Ok(Lit::Number(lhs * rhs)),
            _ => Err(RuntimeError::TypeError {
                msg: NUMERIC_OPERANDS_NEEDED_ERR.to_string(),
            }),
        }
    }

    #[inline]
    pub fn lox_div(self, other: Lit) -> Result<Lit, RuntimeError> {
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

#[derive(Debug, Clone)]
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
    And,
    Or,
}

impl Display for Op {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Op::Equal => f.write_str("=="),
            Op::NotEqual => f.write_str("!="),
            Op::LessThan => f.write_str("<"),
            Op::LessThanEqual => f.write_str("<="),
            Op::GreaterThan => f.write_str(">"),
            Op::GreaterThanEqual => f.write_str(">="),
            Op::Add => f.write_str("+"),
            Op::Sub => f.write_str("-"),
            Op::Mul => f.write_str("*"),
            Op::Div => f.write_str("/"),
            Op::Not => f.write_str("!"),
            Op::And => f.write_str("and"),
            Op::Or => f.write_str("or"),
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
            TokenType::And => Op::And,
            TokenType::Or => Op::Or,
            _ => unimplemented!("attempted to convert {:?} to operator", tok_ty),
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum Unwind {
    Return(Lit),
}

impl Display for Unwind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Unwind::Return(lit) => write!(f, "{lit}"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct LoxFunction {
    pub name: String,
    pub params: Vec<String>,
    pub body: Vec<Stmt>,
    pub closure: Rc<Environment>,
}

#[derive(Clone, Debug)]
pub struct NativeFunction {
    pub name: &'static str,
    pub arity: usize,
    pub func: fn(&Vec<Lit>) -> Result<Lit, RuntimeError>,
}
