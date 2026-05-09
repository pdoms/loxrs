
use crate::{
    environment::Environment, errors::RuntimeError, nodes::{Expr, Lit, Op, Stmt}
};

fn is_truthy(lit: &Lit) -> bool {
    match lit {
        Lit::Nil => false,
        Lit::Bool(b) => *b,
        _ => true,
    }
}

/// We use `W` (output) to test print statements.
pub struct Interpreter<W: std::io::Write> {
    output: W,
    environment: Environment 
}

impl<W: std::io::Write> Interpreter<W> {
    pub fn new(output:  W) -> Self {
        Self {output, environment: Environment::new()}

    }

    pub fn interpret(&mut self, stmts: &[Stmt]) -> Result<(), RuntimeError> {
        for stmt in stmts {
            self.execute(stmt)?;
        }
        Ok(())
    }

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
            Expr::Variable(name) => {
                self.environment.get(name).cloned()
            }
        }
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Print(expr) => {
                let value = self.eval(expr)?;
                let _ = writeln!(self.output, "{}", value);
                self.output.flush().expect("flushing output");
                Ok(())
            },
            Stmt::Expression(expr) => {
                self.eval(expr)?;
                Ok(())
            },
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.eval(expr)?,
                    None => Lit::Nil,
                };
                self.environment.insert(name.as_str(), value);
                Ok(())
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
            let output = Vec::new();
            let mut interpreter = Interpreter::new(output);
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

    #[test]
    fn simple_stmts() {
        let cases = vec![
            ("print 5 + 3 * 2;"      , "11\n"),
            ("print \"hello world\";", "hello world\n"),
            ("1 + 2;"                , "")
        ];
        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            let _ = scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();
            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            assert!(interpreter.interpret(&stmts).is_ok());
            assert_eq!(str::from_utf8(&out).unwrap(), exp);
        }
    }

    #[test]
    fn variables() {
        let cases = vec![
            ("var x = 5; print x;"      , "5\n"),
            ("var x; print x;",         "nil\n"),
            ("var x = 5 + 3; print x;",  "8\n")
        ];
        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            let _ = scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();
            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            assert!(interpreter.interpret(&stmts).is_ok());
            assert_eq!(str::from_utf8(&out).unwrap(), exp);
        }

        let mut scanner = Scanner::new("print x;".as_bytes());
        let _ = scanner.parse().unwrap();
        let mut parser = Parser::new(&scanner.tokens);
        let stmts = parser.parse().unwrap();
        let mut out = Vec::new();
        let mut interpreter = Interpreter::new(&mut out);

        if let Err(RuntimeError::UndefinedVariable { var_name }) = interpreter.interpret(&stmts) {
            assert!(var_name.as_str() == "x");
        } else {
            assert!(false, "unreachable at variables")
        }

    





    }
}
