use crate::lexer::token::{Token, Literal};
use crate::interpreter::interpreter::RuntimeResult;
use crate::error::report::RuntimeError;
use std::collections::HashMap;

pub struct Environment {
    values: HashMap<String, Literal>
}

impl Environment {
    pub fn new() -> Environment {
        let values = HashMap::new();
        Environment {
            values
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Literal> {
        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => Err(RuntimeError::new(name.clone(), &format!("Undefined variable '{}'.", name.lexeme)))
        }
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> RuntimeResult<()> {
        if self.values.contains_key(&name.lexeme) {
            *self.values.get_mut(&name.lexeme).unwrap() = value;
            return Ok(())
        }
        Err(RuntimeError::new(name.clone(), &format!("Undeined variable '{}'.", name.lexeme)))
    }
}