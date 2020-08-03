use crate::interpreter::interpreter::{Interpreter, RuntimeResult};
use crate::interpreter::instance::Instance;
use crate::lexer::literal::Literal;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;
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

    pub fn call(self, _interpreter: &mut Interpreter) -> RuntimeResult<Literal> {
        Ok(Literal::Instance(Rc::new(RefCell::new(Instance::new(self)))))
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}