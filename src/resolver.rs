use std::collections::HashMap;

use crate::{
    errors::ResolveError,
    nodes::{Expr, Stmt},
};

pub struct Resolver {
    // stack of scopes, each scope maps name to bool (bool = initialized)
    scopes: Vec<HashMap<String, bool>>,
    // track if we're inside a function
    function_type: FunctionType,
    // resolved depths: expr identity -> depth (we use *const ponter to have
    // a stable identity)
    pub locals: HashMap<*const Expr, usize>,
}

#[derive(PartialEq)]
enum FunctionType {
    None,
    Function,
}

impl Default for Resolver {
    fn default() -> Self {
        Self {
            scopes: Default::default(),
            function_type: FunctionType::None,
            locals: Default::default(),
        }
    }
}

impl Resolver {
    fn scope_in(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn scope_out(&mut self) {
        self.scopes.pop();
    }

    fn declare(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), false); // not initialized yet
        }
    }

    fn define(&mut self, name: &str) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), true); // initialized
        }
    }

    fn resolve_local(&mut self, expr: *const Expr, name: &str) {
        for (depth, scope) in self.scopes.iter().rev().enumerate() {
            if scope.contains_key(name) {
                self.locals.insert(expr, depth);
                return;
            }
        }

        // must be global, not recording it
    }

    pub fn resolve_stmt(&mut self, stmt: &Stmt) -> Result<(), ResolveError> {
        match stmt {
            Stmt::Block(stmts) => {
                self.scope_in();
                for stmt in stmts {
                    self.resolve_stmt(stmt)?;
                }
                self.scope_out();
            }
            Stmt::Var { name, initializer } => {
                self.declare(name);
                if let Some(init) = initializer {
                    self.resolve_expr(init)?;
                }
                self.define(name);
            }
            Stmt::Function { name, params, body } => {
                self.declare(name);
                self.define(name);
                self.resolve_function(params, body, FunctionType::Function)?;
            }
            Stmt::Return { value } => {
                if self.function_type == FunctionType::None {
                    return Err(ResolveError::ReturnOutsideFunction);
                }
                if let Some(expr) = value {
                    self.resolve_expr(expr)?;
                }
            }
            Stmt::Expression(expr) | Stmt::Print(expr) => {
                self.resolve_expr(expr)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(then_branch)?;
                if let Some(e) = else_branch {
                    self.resolve_stmt(e)?;
                }
            }
            Stmt::While { condition, body } => {
                self.resolve_expr(condition)?;
                self.resolve_stmt(body)?;
            }
        }
        Ok(())
    }
    fn resolve_function(
        &mut self,
        params: &[String],
        body: &[Stmt],
        fun_type: FunctionType,
    ) -> Result<(), ResolveError> {
        let enclosing = std::mem::replace(&mut self.function_type, fun_type);
        self.scope_in();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        for stmt in body {
            self.resolve_stmt(stmt)?;
        }
        self.scope_out();
        self.function_type = enclosing;
        Ok(())
    }

    pub fn resolve_expr(&mut self, expr: &Expr) -> Result<(), ResolveError> {
        match expr {
            Expr::Variable(name) => {
                // catch var a = a
                if let Some(scope) = self.scopes.last()
                    && scope.get(name.as_str()) == Some(&false)
                {
                    return Err(ResolveError::VariableInOwnInititalizer { name: name.clone() });
                }
                self.resolve_local(expr as *const Expr, name);
            }
            Expr::Assign { name, value } => {
                self.resolve_expr(value)?;
                self.resolve_local(expr as *const Expr, name);
            }
            Expr::Binary { right, left, .. } | Expr::Logical { left, right, .. } => {
                self.resolve_expr(left)?;
                self.resolve_expr(right)?;
            }
            Expr::Unary { right, .. } | Expr::Grouping(right) => {
                self.resolve_expr(right)?;
            }
            Expr::Call {
                callee, arguments, ..
            } => {
                self.resolve_expr(callee)?;
                for arg in arguments {
                    self.resolve_expr(arg)?;
                }
            }
            Expr::Literal(_) => {}
        }
        Ok(())
    }
}
