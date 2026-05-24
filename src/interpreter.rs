use std::{collections::HashMap, rc::Rc};

use crate::{
    environment::Environment,
    errors::RuntimeError,
    native::native_functions,
    nodes::{Expr, Lit, LoxFunction, Op, Stmt, Unwind},
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
    environments: Rc<Environment>,
    locals: HashMap<*const Expr, usize>,
}

impl<W: std::io::Write> Interpreter<W> {
    pub fn new(output: W) -> Self {
        let interpreter = Self {
            output,
            environments: Rc::new(Environment::new()),
            locals: HashMap::new(),
        };
        for (name, func) in native_functions() {
            interpreter.environments.define(&name, func);
        }
        interpreter
    }

    pub fn resolve(&mut self, locals: HashMap<*const Expr, usize>) {
        self.locals = locals;
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
            Expr::Logical { left, op, right } => {
                let left = self.eval(left)?;
                match op {
                    Op::And => {
                        if !is_truthy(&left) {
                            return Ok(left);
                        }
                    }
                    Op::Or => {
                        if is_truthy(&left) {
                            return Ok(left);
                        }
                    }
                    _ => {
                        unreachable!("parsers should never produce on-logical op in Expr::Logical")
                    }
                }
                let right = self.eval(right)?;
                Ok(right)
            }
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
            Expr::Variable(name) => self.get_var(expr, name),
            Expr::Assign { name, value } => {
                let value = self.eval(value)?;
                self.set(expr, name, value)
            }
            Expr::Call {
                callee,
                paren,
                arguments,
            } => {
                let callee = self.eval(callee)?;
                let args = arguments
                    .iter()
                    .map(|a| self.eval(a))
                    .collect::<Result<Vec<_>, _>>()?;
                match callee {
                    Lit::Function(func) => self.call_function(func, args),
                    Lit::NativeFunction(func) => {
                        if args.len() != func.arity {
                            return Err(RuntimeError::ArityMismatch {
                                expected: func.arity,
                                got: args.len(),
                            });
                        }
                        (func.func)(&args)
                    }
                    _ => Err(RuntimeError::NotCallable(paren.pos)),
                }
            }
        }
    }

    fn enter_scope(&mut self) {
        let new_env = Rc::new(Environment::new_enclosed(self.environments.clone()));
        self.environments = new_env;
    }

    fn exit_scope(&mut self) {
        let parent = self.environments.parent.clone().expect("no parent scope");
        self.environments = parent;
    }

    fn get_var(&self, expr: &Expr, name: &str) -> Result<Lit, RuntimeError> {
        match self.locals.get(&(expr as *const Expr)) {
            Some(depth) => self.environments.get_at(name, *depth),
            None => self.environments.get(name), // global
        }
    }

    /// is used for x = 5, after var has been already defined
    fn set(&mut self, expr: &Expr, name: &str, value: Lit) -> Result<Lit, RuntimeError> {
        match self.locals.get(&(expr as *const Expr)) {
            Some(depth) => self.environments.set_at(name, value, *depth),
            None => self.environments.set(name, value),
        }
    }

    /// is used at var x = 5
    fn define(&mut self, name: &str, value: Lit) {
        self.environments.define(name, value);
    }

    pub fn execute(&mut self, stmt: &Stmt) -> Result<(), RuntimeError> {
        match stmt {
            Stmt::Print(expr) => {
                let value = self.eval(expr)?;
                let _ = writeln!(self.output, "{}", value);
                self.output.flush().expect("flushing output");
                Ok(())
            }
            Stmt::Expression(expr) => {
                self.eval(expr)?;
                Ok(())
            }
            Stmt::Var { name, initializer } => {
                let value = match initializer {
                    Some(expr) => self.eval(expr)?,
                    None => Lit::Nil,
                };
                self.define(name.as_str(), value);
                Ok(())
            }
            Stmt::Block(stmts) => {
                self.enter_scope();
                let result = stmts.iter().try_for_each(|s| self.execute(s));
                self.exit_scope();
                result
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                if is_truthy(&self.eval(condition)?) {
                    self.execute(then_branch)?;
                } else if let Some(else_branch) = else_branch {
                    self.execute(else_branch)?;
                }
                Ok(())
            }
            Stmt::While { condition, body } => {
                while is_truthy(&self.eval(condition)?) {
                    self.execute(body)?;
                }
                Ok(())
            }
            Stmt::Function { name, params, body } => {
                let func = LoxFunction {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                    closure: self.environments.clone(), // capture environment
                };
                self.define(name, Lit::Function(func));
                Ok(())
            }
            Stmt::Return { value } => {
                let val = match value {
                    Some(expr) => self.eval(expr)?,
                    None => Lit::Nil,
                };
                Err(RuntimeError::Unwind(Unwind::Return(val)))
            }
        }
    }
    fn execute_block(&mut self, stmts: &[Stmt]) -> Result<(), RuntimeError> {
        stmts.iter().try_for_each(|stmt| self.execute(stmt))
    }

    fn call_function(&mut self, func: LoxFunction, args: Vec<Lit>) -> Result<Lit, RuntimeError> {
        if args.len() != func.params.len() {
            return Err(RuntimeError::ArityMismatch {
                expected: func.params.len(),
                got: args.len(),
            });
        }

        let saved = self.environments.clone();
        self.environments = Rc::new(Environment::new_enclosed(func.closure.clone()));
        for (param, arg) in func.params.iter().zip(args) {
            self.environments.define(param, arg);
        }

        let result = self.execute_block(&func.body);

        self.environments = saved;
        match result {
            Ok(()) => Ok(Lit::Nil),
            Err(RuntimeError::Unwind(Unwind::Return(val))) => Ok(val),
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        errors::{ResolveError, RuntimeError},
        interpreter::Interpreter,
        nodes::{Lit, Stmt},
        parser::Parser,
        resolver::Resolver,
        scanner::Scanner,
    };

    fn do_eval(case: &str) -> Result<Lit, RuntimeError> {
        let mut scanner = Scanner::new(case.as_bytes());
        scanner.parse().unwrap();
        let mut parser = Parser::new(&scanner.tokens);
        let res = parser.parse().unwrap();
        if let Stmt::Expression(expr) = &res[0] {
            let output = Vec::new();
            let mut interpreter = Interpreter::new(output);
            return interpreter.eval(expr);
        }
        unreachable!()
    }

    #[test]
    fn eval_arithmetic_expressions() {
        let cases = [
            ("1 + 2;", Ok(Lit::Number(3.0))),
            ("10 - 3;", Ok(Lit::Number(7.0))),
            ("3 * 4;", Ok(Lit::Number(12.0))),
            ("10 / 2;", Ok(Lit::Number(5.0))),
            ("5 + 3 * 2;", Ok(Lit::Number(11.0))),
            ("(5 + 3) * 2;", Ok(Lit::Number(16.0))),
            ("10 / 0;", Err(RuntimeError::DivisionByZero)),
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
        let cases = [
            ("-5;", Lit::Number(-5.0)),
            ("--5;", Lit::Number(5.0)),
            ("!true;", Lit::Bool(false)),
            ("!false;", Lit::Bool(true)),
            ("!nil;", Lit::Bool(true)),
        ];

        for (case, exp) in cases {
            let result = do_eval(case);
            assert!(exp.eq(&result.unwrap()));
        }
    }

    #[test]
    fn eval_comparison_expressions() {
        let cases = [
            ("5 > 3;", Lit::Bool(true)),
            ("3 > 5;", Lit::Bool(false)),
            ("5 >= 5;", Lit::Bool(true)),
            ("3 < 5;", Lit::Bool(true)),
            ("5 <= 4;", Lit::Bool(false)),
        ];

        for (case, exp) in cases {
            let result = do_eval(case);
            assert!(exp.eq(&result.unwrap()));
        }
    }

    #[test]
    fn eval_equality_expressions() {
        let cases = [
            ("1 == 1;", Lit::Bool(true)),
            ("1 == 2;", Lit::Bool(false)),
            ("1 != 2;", Lit::Bool(true)),
            ("nil == nil;", Lit::Bool(true)),
            ("true == true;", Lit::Bool(true)),
            ("true == false;", Lit::Bool(false)),
        ];

        for (case, exp) in cases {
            let result = do_eval(case);
            assert!(exp.eq(&result.unwrap()));
        }
    }

    #[test]
    fn eval_string_expressions() {
        let cases = [
            (
                "\"hello\" + \" world\";",
                Lit::String("hello world".to_string()),
            ),
            ("\"a\" == \"a\";", Lit::Bool(true)),
            ("\"a\" == \"b\";", Lit::Bool(false)),
        ];

        for (case, exp) in cases {
            let result = do_eval(case);
            assert!(exp.eq(&result.unwrap()));
        }
    }

    #[test]
    fn eval_type_errors_expressions() {
        let cases = ["\"hello\" - 1;", "true + 1;", "-true;", "\"a\" > \"b\";"];

        for case in cases {
            let result = do_eval(case);
            assert!(matches!(result, Err(RuntimeError::TypeError { .. })));
        }
    }

    #[test]
    fn simple_stmts() {
        let cases = [
            ("print 5 + 3 * 2;", "11\n"),
            ("print \"hello world\";", "hello world\n"),
            ("1 + 2;", ""),
        ];
        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();
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
        let cases = [
            ("var x = 5; print x;", "5\n"),
            ("var x; print x;", "nil\n"),
            ("var x = 5 + 3; print x;", "8\n"),
        ];
        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();
            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            assert!(interpreter.interpret(&stmts).is_ok());
            assert_eq!(str::from_utf8(&out).unwrap(), exp);
        }

        let mut scanner = Scanner::new("print x;".as_bytes());
        scanner.parse().unwrap();
        let mut parser = Parser::new(&scanner.tokens);
        let stmts = parser.parse().unwrap();
        let mut out = Vec::new();
        let mut interpreter = Interpreter::new(&mut out);

        if let Err(RuntimeError::UndefinedVariable { var_name }) = interpreter.interpret(&stmts) {
            assert!(var_name.as_str() == "x");
        } else {
            unreachable!("unreachable at variables")
        }
    }

    #[test]
    fn assignments() {
        let cases = [
            ("var x = 5; x = 10; print x;", "10\n"), // basic assignement
            ("var x = 5; print x = 10;", "10\n"),    // assignment is an expression!
            (
                "var a = 1; var b = 2; a = b = 3; print a; print b;",
                "3\n3\n",
            ), // right associative
            ("var x = 0; x = 5 + 3; print x;", "8\n"),
            ("var x; x = 2; print x; x = 3; print x;", "2\n3\n"),
            ("var x = 5; var y = 0; y = x; print y;", "5\n"),
            ("var x = 5; print x; x = \"hello\"; print x;", "5\nhello\n"),
        ];
        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();
            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            assert!(interpreter.interpret(&stmts).is_ok());
            assert_eq!(str::from_utf8(&out).unwrap(), exp);
        }
        let error_case = "x = 5;";

        let mut scanner = Scanner::new(error_case.as_bytes());
        scanner.parse().unwrap();
        let mut parser = Parser::new(&scanner.tokens);
        let stmts = parser.parse().unwrap();
        let mut out = Vec::new();
        let mut interpreter = Interpreter::new(&mut out);

        if let Err(RuntimeError::UndefinedVariable { var_name }) = interpreter.interpret(&stmts) {
            assert!(var_name.as_str() == "x");
        } else {
            unreachable!("unreachable at variables")
        }
    }

    #[test]
    fn scope() {
        let cases = [
            // inner scope shadows outer
            ("var x = 1; { var x = 2; print x; } print x;", "2\n1\n"),
            // inner scope sees outer variable
            ("var x = 1; { print x; }", "1\n"),
            // inner assignment affects outer
            ("var x = 1; { x = 2; } print x;", "2\n"),
            // nested scopes
            (
                "var x = 1; { var x = 2; { var x = 3; print x; } print x; } print x;",
                "3\n2\n1\n",
            ),
            // nested scope inner assign affects nearest declaration
            (
                "var x = 1; { var x = 2; { x = 3; } print x; } print x;",
                "3\n1\n",
            ),
            // multiple variables in scope
            (
                "var x = 1; var y = 2; { var y = 3; print x; print y; } print y;",
                "1\n3\n2\n",
            ),
            // declare in scope, assign from outer
            ("var x = 1; { var y = x + 1; print y; }", "2\n"),
        ];
        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();
            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            assert!(interpreter.interpret(&stmts).is_ok());
            assert_eq!(str::from_utf8(&out).unwrap(), exp);
        }
        // variable does not leak out of scope
        let error_case = "{ var x = 1; } print x;";

        let mut scanner = Scanner::new(error_case.as_bytes());
        scanner.parse().unwrap();
        let mut parser = Parser::new(&scanner.tokens);
        let stmts = parser.parse().unwrap();
        let mut out = Vec::new();
        let mut interpreter = Interpreter::new(&mut out);

        if let Err(RuntimeError::UndefinedVariable { var_name }) = interpreter.interpret(&stmts) {
            assert!(var_name.as_str() == "x");
        } else {
            unreachable!("unreachable at variables")
        }
    }

    #[test]
    fn if_statements() {
        let cases = [
            ("if (true) print 1;", "1\n"),
            ("if (false) print 1;", ""),
            ("if (true) print 1; else print 2;", "1\n"),
            ("if (false) print 1; else print 2;", "2\n"),
            ("var x = 5; if (x > 3) print x;", "5\n"),
            ("if (1 == 1) { print 1; print 2; }", "1\n2\n"),
            ("if (false) print 1; else { print 2; print 3; }", "2\n3\n"),
            // dangling else — else binds to nearest if
            ("if (true) if (false) print 1; else print 2;", "2\n"),
        ];
        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();
            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            assert!(interpreter.interpret(&stmts).is_ok());
            assert_eq!(str::from_utf8(&out).unwrap(), exp);
        }
    }

    #[test]
    fn logical_and_or() {
        let cases = [
            // and
            ("print true and true;", "true\n"),
            ("print true and false;", "false\n"),
            ("print false and true;", "false\n"),
            ("print false and false;", "false\n"),
            // or
            ("print true or false;", "true\n"),
            ("print false or true;", "true\n"),
            ("print false or false;", "false\n"),
            ("print true or true;", "true\n"),
            // short circuit and — right side never evaluated
            ("print false and (1/0);", "false\n"), // no DivisionByZero error
            // short circuit or — right side never evaluated
            ("print true or (1/0);", "true\n"), // no DivisionByZero error
            // with variables
            ("var x = true; var y = false; print x and y;", "false\n"),
            ("var x = true; var y = false; print x or y;", "true\n"),
            // truthiness
            ("print nil and true;", "nil\n"), // nil is falsy, short circuits
            ("print nil or true;", "true\n"),
            ("print 0 or false;", "0\n"),    // 0 is truthy in Lox!
            ("print 0 and true;", "true\n"), // 0 is truthy in Lox!
            // chained
            ("print true and true and false;", "false\n"),
            ("print false or false or true;", "true\n"),
        ];
        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();
            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            assert!(interpreter.interpret(&stmts).is_ok());
            assert_eq!(str::from_utf8(&out).unwrap(), exp, "case: {case}");
        }
    }

    #[test]
    fn while_loops() {
        let cases = [
            // basic
            ("var x = 0; while (x < 3) { x = x + 1; } print x;", "3\n"),
            // never executes
            ("var x = 0; while (false) { x = x + 1; } print x;", "0\n"),
            // countdown
            (
                "var x = 3; while (x > 0) { print x; x = x - 1; }",
                "3\n2\n1\n",
            ),
            // accumulator
            (
                "var sum = 0; var i = 1; while (i <= 3) { sum = sum + i; i = i + 1; } print sum;",
                "6\n",
            ),
        ];
        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();
            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            assert!(interpreter.interpret(&stmts).is_ok());
            assert_eq!(str::from_utf8(&out).unwrap(), exp);
        }
    }

    #[test]
    fn for_loops() {
        let cases = [
            // basic counting
            ("for (var i = 0; i < 3; i = i + 1) print i;", "0\n1\n2\n"),
            // accumulator
            (
                "var sum = 0; for (var i = 1; i <= 3; i = i + 1) { sum = sum + i; } print sum;",
                "6\n",
            ),
            // no initializer
            ("var i = 0; for (; i < 3; i = i + 1) print i;", "0\n1\n2\n"),
            // no increment
            (
                "for (var i = 0; i < 3;) { print i; i = i + 1; }",
                "0\n1\n2\n",
            ),
        ];
        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();
            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            assert!(interpreter.interpret(&stmts).is_ok());
            assert_eq!(str::from_utf8(&out).unwrap(), exp);
        }
    }

    #[test]
    fn fibonacci_for_loop() {
        let code = r#"
       var a = 0;
       var temp;

       for (var b = 1; a < 10000; b = temp + b) {
        print a;
        temp = a;
        a = b;

       }
           "#;
        let expect = "0\n1\n1\n2\n3\n5\n8\n13\n21\n34\n55\n89\n144\n233\n377\n610\n987\n1597\n2584\n4181\n6765\n";
        let mut scanner = Scanner::new(code.as_bytes());
        scanner.parse().unwrap();
        let mut parser = Parser::new(&scanner.tokens);
        let stmts = parser.parse().unwrap();
        let mut out = Vec::new();
        let mut interpreter = Interpreter::new(&mut out);
        assert!(interpreter.interpret(&stmts).is_ok());
        assert_eq!(str::from_utf8(&out).unwrap(), expect);
    }

    #[test]
    fn funcs() {
        let cases = [
            ("fun greet() { print \"hello\"; } greet();", "hello\n"),
            ("fun add(a, b) { return a + b; } print add(1, 2);", "3\n"),
            ("fun square(x) { return x * x; } print square(4);", "16\n"),
            ("fun nothing() {} print nothing();", "nil\n"),
            (
                "fun sign(x) { if (x > 0) { return 1; } if (x < 0) { return -1; } return 0; } print sign(5); print sign(-3); print sign(0);",
                "1\n-1\n0\n",
            ),
            (
                "var x = 1; fun f() { var x = 2; return x; } print f(); print x;",
                "2\n1\n",
            ),
            ("var x = 10; fun f() { return x; } print f();", "10\n"),
            (
                "fun fib(n) { if (n <= 1) { return n; } return fib(n - 1) + fib(n - 2); } print fib(7);",
                "13\n",
            ),
            (
                "fun fact(n) { if (n <= 1) { return 1; } return n * fact(n - 1); } print fact(5);",
                "120\n",
            ),
            ("print clock() > 0;", "true\n"),
            (
                "var t1 = clock(); var t2 = clock(); print t2 >= t1;",
                "true\n",
            ),
        ];
        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();
            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            assert!(interpreter.interpret(&stmts).is_ok());
            assert_eq!(str::from_utf8(&out).unwrap(), exp, "{case}");
        }

        //errors
        let cases = [
            (
                "fun f(a) {} f();",
                Err(RuntimeError::ArityMismatch {
                    expected: 1,
                    got: 0,
                }),
            ),
            (
                "fun f() {} f(1);",
                Err(RuntimeError::ArityMismatch {
                    expected: 0,
                    got: 1,
                }),
            ),
        ];
        for (case, err) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();
            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            assert_eq!(interpreter.interpret(&stmts), err, "{case}");
        }
    }

    #[test]
    fn closures() {
        let cases = [
            // basic closure
            (
                "fun makeCounter() { var count = 0; fun increment() { count = count + 1; return count; } return increment; } var counter = makeCounter(); print counter(); print counter();",
                "1\n2\n",
            ),
            // closure captures value at definition time
            ("var x = 1; fun f() { return x; } x = 2; print f();", "2\n"), // sees the updated x since it captures the environment by reference
            // each closure is independent
            (
                "fun makeCounter() { var count = 0; fun increment() { count = count + 1; return count; } return increment; } var c1 = makeCounter(); var c2 = makeCounter(); print c1(); print c1(); print c2();",
                "1\n2\n1\n",
            ),
        ];
        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();
            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();
            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            assert!(interpreter.interpret(&stmts).is_ok());
            assert_eq!(str::from_utf8(&out).unwrap(), exp, "case: {case}");
        }
    }

    #[test]
    fn resolving() {
        let cases = [
            ("var a = 1; print a;", "1\n"),
            // classic closure test
            ("var a = 1; { var a = 2; print a; } print a;", "2\n1\n"),
            // counter still works
            (
                "fun makeCounter() { var count = 0; fun increment() { count = count + 1; return count; } return increment; } var c = makeCounter(); print c(); print c();",
                "1\n2\n",
            ),
            // each counter independent
            (
                "fun makeCounter() { var count = 0; fun increment() { count = count + 1; return count; } return increment; } var c1 = makeCounter(); var c2 = makeCounter(); print c1(); print c1(); print c2();",
                "1\n2\n1\n",
            ),
            // valid return inside function
            ("fun f() { return 1; } print f();", "1\n"),
            (
                "var x = 1; fun f() { var x = 2; fun g() { return x; } return g(); } print f();",
                "2\n",
            ), // g sees f's x, not global x
            (
                "var x = 1; fun f() { fun g() { return x; } return g(); } print f();",
                "1\n",
            ), // g sees global x through f
        ];

        for (case, exp) in cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();

            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();

            let mut resolver = Resolver::new();
            for stmt in &stmts {
                resolver.resolve_stmt(stmt).unwrap();
            }

            let mut out = Vec::new();
            let mut interpreter = Interpreter::new(&mut out);
            interpreter.resolve(resolver.locals);
            assert!(interpreter.interpret(&stmts).is_ok());
            assert_eq!(str::from_utf8(&out).unwrap(), exp, "case: {case}");
        }

        let error_cases = [
            (
                "var a = a;",
                ResolveError::VariableInOwnInititalizer {
                    name: String::from("a"),
                },
            ),
            ("return 5;", ResolveError::ReturnOutsideFunction),
            // return outside function
            ("fun f() { } return 1;", ResolveError::ReturnOutsideFunction),
        ];

        for (case, exp) in error_cases {
            let mut scanner = Scanner::new(case.as_bytes());
            scanner.parse().unwrap();

            let mut parser = Parser::new(&scanner.tokens);
            let stmts = parser.parse().unwrap();

            let mut resolver = Resolver::new();
            if let Err(err) = resolver.resolve_stmt(&stmts[0]) {
                assert_eq!(err, exp, "error case: {case}");
            }
        }
    }
}
