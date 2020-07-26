use crate::interpreter::interpreter::{Interpreter, RuntimeResult};
use crate::interpreter::class::Class;
use crate::lexer::literal::Literal;
use std::fmt;
#[derive(Debug, PartialEq, Clone)]
pub struct Instance {
    class: Class
}

impl Instance {
    pub fn new(class: &Class) -> Instance {
        Instance {
            class: class.clone()
        }
    }
}

impl fmt::Display for Instance {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<object {}>", self.class.name)
    }
}