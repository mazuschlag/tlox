use crate::error::report::RuntimeError;
use crate::interpreter::class::Class;
use crate::interpreter::interpreter::RuntimeResult;
use crate::lexer::literal::{Instance, Literal};
use crate::lexer::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    pub class: Rc<RefCell<Class>>,
    pub fields: HashMap<String, Literal>,
}

impl Object {
    pub fn new(class: Class) -> Object {
        Object {
            class: Rc::new(RefCell::new(class)),
            fields: HashMap::new(),
        }
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Literal> {
        if let Some(value) = self.fields.get(&name.lexeme) {
            return Ok(value.clone());
        }
        if let Some(Literal::Fun(method)) = self.class.borrow().find_method(&name.lexeme) {
            return Ok(method.bind(Instance::Dynamic(Rc::new(RefCell::new(self.clone())))));
            // this seems dangerous
        }
        Err(RuntimeError::new(
            name.clone(),
            &format!("Undefined property '{}'.", name.lexeme),
        ))
    }

    pub fn set(&mut self, name: &Token, value: Literal) -> RuntimeResult<Literal> {
        self.fields.insert(name.lexeme.clone(), value.clone());
        Ok(value)
    }
}

impl fmt::Display for Object {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string = "<object {}>".to_string();
        for (key, value) in &self.fields {
            string += &format!("{}: {},", key, value);
        }
        write!(f, "<object {}>", self.class.borrow().name)
    }
}
