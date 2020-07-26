use crate::interpreter::interpreter::RuntimeResult;
use crate::error::report::RuntimeError;
use crate::interpreter::class::Class;
use crate::lexer::literal::Literal;
use crate::lexer::token::Token;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Instance {
    class: Rc<RefCell<Class>>,
    pub fields: HashMap<String, Literal>
}

impl Instance {
    pub fn new(class: Class) -> Instance {
        Instance {
            class: Rc::new(RefCell::new(class)),
            fields: HashMap::new()
        }
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Literal> {
        match self.fields.get(&name.lexeme) {
            Some(value) => Ok(value.clone()),
            None => Err(RuntimeError::new(name.clone(), &format!("Undefined property '{}'.", name.lexeme)))
        }
    }

    pub fn set(&mut self, name: &Token, value: Literal) -> RuntimeResult<Literal> {
        self.fields.insert(name.lexeme.clone(), value.clone());
        Ok(value)
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut string = "<object {}>".to_string();
        for (key, value) in &self.fields {
            string += &format!("{}: {},", key, value);
        }
        write!(f, "<object {}>", self.class.borrow().name)
    }
}