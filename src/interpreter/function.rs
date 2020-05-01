use crate::lexer::literal::Literal;
use crate::interpreter::interpreter::Interpreter;

pub trait Callable {
    fn call<T: Callable>(&self, interpreter: &Interpreter, args: &Vec<Literal>) -> Result<Literal, String>;
    fn arity(&self) -> usize;
}