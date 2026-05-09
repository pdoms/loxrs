use crate::{
    errors::RuntimeError,
    nodes::{Expr, Lit, Op},
};

fn is_truthy(lit: &Lit) -> bool {
    match lit {
        Lit::Nil => false,
        Lit::Bool(b) => *b,
        _ => true,
    }
}

pub struct Interpreter;

impl Interpreter {
    pub fn eval(&mut self, expr: &Expr) -> Result<Lit, RuntimeError> {
        match expr {
            Expr::Literal(lit) => Ok(lit.clone()),
            Expr::Grouping(inner) => self.eval(inner),
            Expr::Unary { op, right } => {
                let right = self.eval(right)?;
                match op {
                    Op::Not => Ok(Lit::Bool(!is_truthy(&right))),
                    Op::Sub => match right {
                        Lit::Number(n) => Ok(Lit::Number(-n)),
                        _ => Err(RuntimeError::TypeError {
                            msg: "operand must be a number".to_string(),
                        }),
                    },
                    _ => Err(RuntimeError::TypeError {
                        msg: "invalid unary operator".to_string(),
                    }),
                }
            }
            Expr::Binary { op, right, left } => {
                let left = self.eval(left)?;
                let right = self.eval(right)?;

                match op {
                    Op::Equal => Ok(Lit::Bool(left == right)),
                    Op::NotEqual => Ok(Lit::Bool(left != right)),
                    Op::LessThan => left.less(right),
                    Op::LessThanEqual => left.less_eq(right),
                    Op::GreaterThan => left.greater(right),
                    Op::GreaterThanEqual => left.greater_eq(right),
                    Op::Add => left.add(right),
                    Op::Sub => left.sub(right),
                    Op::Mul => left.mul(right),
                    Op::Div => left.div(right),
                    _ => Err(RuntimeError::InvalidOperator {
                        msg: format!("'{}' is not a valid binary operator", op),
                    }),
                }
            }
        }
    }
}


#[cfg(test)]
mod test {
    use crate::{
        errors::RuntimeError, interpreter::Interpreter, nodes::{Lit, Stmt}, parser::Parser, scanner::Scanner
    };

    fn do_eval(case: &str) -> Result<Lit, RuntimeError> {
        let mut scanner = Scanner::new(case.as_bytes());
        let _ = scanner.parse().unwrap();
        let mut parser = Parser::new(&scanner.tokens);
        let res = parser.parse().unwrap();
        if let Stmt::Expression(expr) = &res[0] {
            let mut interpreter = Interpreter;
            return interpreter.eval(&expr);
        }
        unreachable!()
    }

    #[test]
    fn eval_arithmetic_expressions() {
        let cases = vec![
            ("1 + 2", Ok(Lit::Number(3.0))),
            ("10 - 3", Ok(Lit::Number(7.0))),
            ("3 * 4", Ok(Lit::Number(12.0))),
            ("10 / 2", Ok(Lit::Number(5.0))),
            ("5 + 3 * 2", Ok(Lit::Number(11.0))),
            ("(5 + 3) * 2", Ok(Lit::Number(16.0))),
            ("10 / 0", Err(RuntimeError::DivisionByZero)),
        ];

        for (case, exp) in cases {
            let result = do_eval(case);
            match exp {
                Ok(lit) => assert!(lit.eq(&result.unwrap())),
                Err(err) => assert!(err == RuntimeError::DivisionByZero),
            }
        }
    }

    #[test]
    fn eval_unary_expressions() {
        let cases = vec![
            ("-5", Lit::Number(-5.0)),
            ("--5", Lit::Number(5.0)),
            ("!true", Lit::Bool(false)),
            ("!false", Lit::Bool(true)),
            ("!nil", Lit::Bool(true)),
        ];

        for (case, exp) in cases {
            let result = do_eval(case);
            assert!(exp.eq(&result.unwrap()));
        }
    }

    #[test]
    fn eval_comparison_expressions() {
        let cases = vec![
            ("5 > 3", Lit::Bool(true)),
            ("3 > 5", Lit::Bool(false)),
            ("5 >= 5", Lit::Bool(true)),
            ("3 < 5", Lit::Bool(true)),
            ("5 <= 4", Lit::Bool(false)),
        ];

        for (case, exp) in cases {
            let result = do_eval(case);
            assert!(exp.eq(&result.unwrap()));
        }
    }

    #[test]
    fn eval_equality_expressions() {
        let cases = vec![
            ("1 == 1", Lit::Bool(true)),
            ("1 == 2", Lit::Bool(false)),
            ("1 != 2", Lit::Bool(true)),
            ("nil == nil", Lit::Bool(true)),
            ("true == true", Lit::Bool(true)),
            ("true == false", Lit::Bool(false)),
        ];

        for (case, exp) in cases {
            let result = do_eval(case);
            assert!(exp.eq(&result.unwrap()));
        }
    }

    #[test]
    fn eval_string_expressions() {
        let cases = vec![
            (
                "\"hello\" + \" world\"",
                Lit::String("hello world".to_string()),
            ),
            ("\"a\" == \"a\"", Lit::Bool(true)),
            ("\"a\" == \"b\"", Lit::Bool(false)),
        ];

        for (case, exp) in cases {
            let result = do_eval(case);
            assert!(exp.eq(&result.unwrap()));
        }
    }

    #[test]
    fn eval_type_errors_expressions() {
        let cases = vec!["\"hello\" - 1", "true + 1", "-true", "\"a\" > \"b\""];

        for case in cases {
            let result = do_eval(case);
            assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
        }
    }
}
