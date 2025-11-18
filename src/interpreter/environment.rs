use crate::error::report::RuntimeError;
use crate::interpreter::interpreter::RuntimeResult;
use crate::lexer::literal::Literal;
use crate::lexer::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

type Enclosing = Option<Rc<RefCell<Environment>>>;

#[derive(Debug, Clone, PartialEq)]
pub struct Environment {
    pub values: HashMap<String, Literal>,
    pub outer_scope: Enclosing,
}

impl Environment {
    pub fn new(outer_scope: Enclosing) -> Environment {
        let values = HashMap::new();
        Environment {
            values,
            outer_scope,
        }
    }

    pub fn define(&mut self, name: String, value: Literal) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &String) -> Option<Literal> {
        match self.values.get(name) {
            Some(value) => Some(value.clone()),
            None => {
                if let Some(enclosing) = &self.outer_scope {
                    return enclosing.borrow().get(name);
                }
                None
            }
        }
    }

    pub fn get_at(&self, name: &String, distance: usize) -> Option<Literal> {
        match self.ancestor(distance)?.borrow().values.get(name) {
            Some(value) => Some(value.clone()),
            None => None,
        }
    }

    pub fn assign(&mut self, name: &Token, value: Literal) -> RuntimeResult<()> {
        if self.values.contains_key(&name.lexeme) {
            self.values.insert(name.lexeme.clone(), value);
            return Ok(());
        }
        if let Some(enclosing) = &self.outer_scope {
            return enclosing.borrow_mut().assign(name, value);
        }
        Err(RuntimeError::new(
            name.clone(),
            &format!("Undefined variable '{}'.", name.lexeme),
        ))
    }

    pub fn assign_at(
        &mut self,
        name: &Token,
        value: Literal,
        distance: usize,
    ) -> RuntimeResult<()> {
        if let Some(env) = self.ancestor(distance) {
            env.borrow_mut().values.insert(name.lexeme.clone(), value);
            return Ok(());
        }
        return Err(RuntimeError::new(
            name.clone(),
            &format!("Undefined variable '{}'.", name.lexeme),
        ));
    }

    fn ancestor(&self, distance: usize) -> Option<Rc<RefCell<Environment>>> {
        let mut environment = Rc::new(RefCell::new(self.clone()));
        for _ in 0..distance {
            environment = match &Rc::clone(&environment).borrow().outer_scope {
                Some(e) => Rc::clone(e),
                None => return None,
            };
        }
        Some(environment)
    }
}
