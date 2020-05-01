use crate::lexer::token::Token;
use crate::lexer::literal::Literal;
use crate::interpreter::interpreter::RuntimeResult;
use crate::error::report::RuntimeError;
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;

type Enclosing = Option<Rc<RefCell<Environment>>>;

#[derive(Clone)]
pub struct Environment {
    values: HashMap<String, Literal>,
    outer_scope: Enclosing
}

impl Environment {
    pub fn new(outer_scope: Enclosing) -> Environment {
        let values = HashMap::new();
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