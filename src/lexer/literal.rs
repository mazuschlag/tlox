use std::fmt;
use crate::interpreter::function::Function;
#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Str(String),
    Number(f64),
    Bool(bool),
    Fun(Function),
    Nothing
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Literal::Str(s) => write!(f, "\"{}\"", s),
            Literal::Number(n) => write!(f, "{}", n),
            Literal::Bool(b) => write!(f, "{}", b),
            Literal::Fun(function) => write!(f, "{}", function.to_string()),
            Literal::Nothing => write!(f, "nil")
        }
    }
}