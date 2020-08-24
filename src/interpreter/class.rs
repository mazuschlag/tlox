use crate::interpreter::interpreter::{Interpreter, RuntimeResult};
use crate::interpreter::instance::Instance;
use crate::lexer::literal::Literal;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub struct Class {
    pub name: String,
    pub methods: HashMap<String, Literal>,
    arity: usize
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Literal>) -> Class {
        Class {
            name,
            methods,
            arity: 0
        }
    }

    pub fn call(self, _interpreter: &mut Interpreter) -> RuntimeResult<Literal> {
        Ok(Literal::Instance(Rc::new(RefCell::new(Instance::new(self)))))
    }

    pub fn find_method(&self, name: &String) -> Option<Literal> {
        match self.methods.get(name) {
            Some(method) => Some(method.clone()),
            None => None
        }
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}