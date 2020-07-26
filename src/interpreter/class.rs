use crate::interpreter::interpreter::{Interpreter, RuntimeResult};
use crate::interpreter::instance::Instance;
use crate::lexer::literal::Literal;
use std::fmt;
#[derive(Debug, PartialEq, Clone)]
pub struct Class {
    pub name: String,
    arity: usize
}

impl Class {
    pub fn new(name: String) -> Class {
        Class {
            name,
            arity: 0
        }
    }
    pub fn call(&self, interpreter: &mut Interpreter) -> RuntimeResult<Literal> {
        Ok(Literal::Instance(Instance::new(self)))
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}