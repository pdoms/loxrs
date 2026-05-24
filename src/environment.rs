use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{errors::RuntimeError, nodes::Lit};

#[derive(Clone, Debug)]
pub struct Environment {
    pub values: Rc<RefCell<HashMap<String, Lit>>>,
    pub parent: Option<Rc<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: Rc::new(RefCell::new(HashMap::new())),
            parent: None,
        }
    }

    pub fn new_enclosed(parent: Rc<Environment>) -> Self {
        Self {
            values: Rc::new(RefCell::new(HashMap::new())),
            parent: Some(parent),
        }
    }

    pub fn define(&self, name: &str, value: Lit) {
        self.values.borrow_mut().insert(name.to_string(), value);
    }

    pub fn get(&self, name: &str) -> Result<Lit, RuntimeError> {
        if let Some(val) = self.values.borrow().get(name) {
            return Ok(val.clone());
        }
        match &self.parent {
            Some(parent) => parent.get(name),
            None => Err(RuntimeError::UndefinedVariable {
                var_name: name.to_string(),
            }),
        }
    }

    pub fn set(&self, name: &str, value: Lit) -> Result<Lit, RuntimeError> {
        if self.values.borrow().contains_key(name) {
            self.values
                .borrow_mut()
                .insert(name.to_string(), value.clone());
            return Ok(value);
        }
        match &self.parent {
            Some(parent) => parent.set(name, value),
            None => Err(RuntimeError::UndefinedVariable {
                var_name: name.to_string(),
            }),
        }
    }
    pub fn get_at(&self, name: &str, depth: usize) -> Result<Lit, RuntimeError> {
        if depth == 0 {
            return self.values.borrow().get(name).cloned().ok_or(
                RuntimeError::UndefinedVariable {
                    var_name: name.to_string(),
                },
            );
        }
        match &self.parent {
            Some(parent) => parent.get_at(name, depth - 1),
            None => Err(RuntimeError::UndefinedVariable {
                var_name: name.to_string(),
            }),
        }
    }

    pub fn set_at(&self, name: &str, value: Lit, depth: usize) -> Result<Lit, RuntimeError> {
        if depth == 0 {
            if self.values.borrow().contains_key(name) {
                self.values
                    .borrow_mut()
                    .insert(name.to_string(), value.clone());
                return Ok(value);
            }
            return Err(RuntimeError::UndefinedVariable {
                var_name: name.to_string(),
            });
        }
        match &self.parent {
            Some(parent) => parent.set_at(name, value, depth - 1),
            None => Err(RuntimeError::UndefinedVariable {
                var_name: name.to_string(),
            }),
        }
    }
}
