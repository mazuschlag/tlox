use crate::interpreter::environment::Environment;
use crate::interpreter::instance::Instance;
use crate::interpreter::interpreter::{Interpreter, RuntimeResult};
use crate::lexer::literal::Literal;
use crate::lexer::token::Token;
use crate::parser::statement::Stmt;
use std::cell::RefCell;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub struct Function {
    pub arity: usize,
    name: Option<Token>,
    params: Vec<Token>,
    body: Vec<Stmt>,
    closure: Rc<RefCell<Environment>>,
    is_initializer: bool,
}

impl Function {
    pub fn new(
        name: Option<Token>,
        params: Vec<Token>,
        body: Vec<Stmt>,
        parent: &Rc<RefCell<Environment>>,
        is_initializer: bool,
    ) -> Function {
        let arity = params.len();
        let closure = Rc::clone(parent);
        Function {
            arity,
            name,
            params,
            body,
            closure,
            is_initializer,
        }
    }

    pub fn call(
        &self,
        interpreter: &mut Interpreter,
        args: &Vec<Literal>,
    ) -> RuntimeResult<Literal> {
        let mut env = Environment::new(Some(Rc::clone(&self.closure)));
        for i in 0..self.arity {
            env.define(self.params[i].lexeme.clone(), args[i].clone());
        }
        interpreter.in_initializer = self.is_initializer;
        interpreter.visit_block_stmt(&self.body, Some(env))?;

        if self.is_initializer {
            if let Some(instance) = self.closure.borrow().get_at(&"this".to_string(), 0) {
                return Ok(instance);
            }
        }
        Ok(Literal::Nothing)
    }

    pub fn bind(&self, instance: Rc<RefCell<Instance>>) -> Literal {
        let mut env = Environment::new(Some(Rc::clone(&self.closure)));
        env.define("this".to_string(), Literal::Instance(instance));
        Literal::Fun(Function::new(
            self.name.clone(),
            self.params.clone(),
            self.body.clone(),
            &Rc::new(RefCell::new(env)),
            self.is_initializer,
        ))
    }
}

impl fmt::Display for Function {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(name) = &self.name {
            return write!(f, "<fn {}>", name.lexeme);
        }
        write!(f, "<lambda>")
    }
}
