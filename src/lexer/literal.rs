use crate::interpreter::class::Class;
use crate::interpreter::function::Function;
use crate::interpreter::object::Object;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Str(String),
    Number(f64),
    Bool(bool),
    Fun(Function),
    Get(Function),
    Class(Class),
    Instance(Instance),
    Nothing,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instance {
    Static(Rc<RefCell<Class>>),
    Dynamic(Rc<RefCell<Object>>),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Str(s) => write!(f, "\"{}\"", s),
            Literal::Number(n) => write!(f, "{}", n),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Fun(function) => write!(f, "{}", function.to_string()),
            Literal::Get(function) => write!(f, "getter {}", function.to_string()),
            Literal::Class(class) => write!(f, "{}", class.to_string()),
            Literal::Instance(instance) => match instance {
                Instance::Static(class) => write!(f, "{}", class.borrow().to_string()),
                Instance::Dynamic(object) => write!(f, "{}", object.borrow().to_string()),
            },
            Literal::Nothing => write!(f, "nil"),
        }
    }
}
