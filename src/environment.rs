use std::collections::HashMap;

use crate::{errors::RuntimeError, nodes::Lit};

pub struct Environment {
    vars: HashMap<String, Lit>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
        }
    }

    pub fn insert(&mut self, k: &str, v: Lit) {
        self.vars.insert(k.to_owned(), v);
    }

    pub fn get(&self, k: &String) -> Result<&Lit, RuntimeError> {
        match self.vars.get(k) {
            Some(v) => Ok(v),
            None => Err(RuntimeError::UndefinedVariable {
                var_name: k.to_owned(),
            }),
        }
    }
    pub fn get_mut(&mut self, k: &String) -> Result<&mut Lit, RuntimeError> {
        match self.vars.get_mut(k) {
            Some(v) => Ok(v),
            None => Err(RuntimeError::UndefinedVariable {
                var_name: k.to_owned(),
            }),
        }
    }

    pub fn contains_key(&self, key: &String) -> bool {
        self.vars.contains_key(key)
    }
}
