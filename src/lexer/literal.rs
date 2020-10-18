use crate::interpreter::class::Class;
use crate::interpreter::function::Function;
use crate::interpreter::instance::Instance;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Str(String),
    Number(f64),
    Bool(bool),
    Fun(Function),
    Class(Class),
    Instance(Rc<RefCell<Instance>>),
    Nothing,
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Str(s) => write!(f, "\"{}\"", s),
            Literal::Number(n) => write!(f, "{}", n),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Fun(function) => write!(f, "{}", function.to_string()),
            Literal::Class(class) => write!(f, "{}", class.to_string()),
            Literal::Instance(instance) => write!(f, "{}", instance.borrow().to_string()),
            Literal::Nothing => write!(f, "nil"),
        }
    }
}
