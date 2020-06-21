use crate::lexer::token::Token;
use crate::lexer::literal::Literal;
use crate::interpreter::interpreter::RuntimeResult;
use crate::error::report::RuntimeError;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    values: HashMap<String, Literal>,
    depth: usize
}

impl Environment {
    pub fn new(depth: usize) -> Environment {
        let values = HashMap::new();
        Environment {
            values,
            depth
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token, outer_scopes: &Vec<Environment>) -> RuntimeResult<Literal> {
        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => {
                if self.depth > 0 {
                    for i in (0..=self.depth).rev() {
                        if let Some(value) = outer_scopes[i].values.get(&name.lexeme) {
                            return Ok(value.clone());
                        }   
                    }
                }
                Err(RuntimeError::new(name.clone(), &format!("Undefined variable '{}'.", name.lexeme)))
            }
        }
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> bool {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return true
        }
        false
    }
}