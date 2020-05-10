use crate::interpreter::interpreter::{Interpreter, RuntimeResult};
use crate::interpreter::environment::Environment;
use crate::parser::statement::Stmt;
use crate::lexer::literal::Literal;
use crate::lexer::token::Token;
use std::rc::Rc;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub arity: usize,
    name: Token,
    params: Vec<Token>,
    body: Vec<Stmt>
}

impl Function {
    pub fn new(name: Token, params: Vec<Token>, body: Vec<Stmt>) -> Function {
        let arity = params.len();
        Function {
            arity,
            name,
            params,
            body
        }
    }
    
    pub fn call(&self, interpreter: &mut Interpreter, args: &Vec<Literal>) -> RuntimeResult<()> {
        let mut env = Environment::new(Some(Rc::clone(&interpreter.globals)));
        for i in 0..self.arity {
            env.define(self.params[i].lexeme.clone(), args[i].clone());
        }
        interpreter.execute_block_stmt(&self.body, env)?;
        Ok(())
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<fn {}>", self.name.lexeme)
    }
}