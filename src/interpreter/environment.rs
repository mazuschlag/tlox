use crate::lexer::token::{Token, Literal};
use crate::interpreter::interpreter::RuntimeResult;
use crate::error::report::RuntimeError;
use std::collections::HashMap;
use std::rc::Rc;
use std::cell::RefCell;

type Enclosing = Rc<RefCell<Environment>>;

#[derive(Clone)]
pub struct Environment {
    values: HashMap<String, Literal>,
    outer_scope: Option<Enclosing>
}

impl Environment {
    pub fn new(enclosing: Option<Environment>) -> Environment {
        let values = HashMap::new();
        let outer_scope = match enclosing {
            Some(env) => Some(Rc::new(RefCell::new(env))),
            None => None
        };
        Environment {
            values,
            outer_scope
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Literal> {
        match self.values.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => {
                if let Some(enclosing) = &self.outer_scope {
                    return enclosing.borrow().get(&name)
                } 
                Err(RuntimeError::new(name.clone(), &format!("Undefined variable '{}'.", name.lexeme)))
            }
        }
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> RuntimeResult<()> {
        if self.values.contains_key(&name.lexeme) {
            *self.values.get_mut(&name.lexeme).unwrap() = value;
            return Ok(())
        }
        if let Some(enclosing) = &self.outer_scope {
            return enclosing.borrow_mut().assign(name, value)
        }
        Err(RuntimeError::new(name.clone(), &format!("Undeined variable '{}'.", name.lexeme)))
    }
}