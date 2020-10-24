use crate::error::report::RuntimeError;
use crate::interpreter::interpreter::{Interpreter, RuntimeResult};
use crate::interpreter::object::Object;
use crate::lexer::literal::{Instance, Literal};
use crate::lexer::token::Token;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt;
use std::rc::Rc;

#[derive(Debug, PartialEq, Clone)]
pub struct Class {
    pub name: String,
    pub methods: HashMap<String, Literal>,
}

impl Class {
    pub fn new(name: String, methods: HashMap<String, Literal>) -> Class {
        Class { name, methods }
    }

    #[allow(dead_code)]
    pub fn arity(&self) -> usize {
        if let Some(Literal::Fun(init)) = self.find_method(&"init".to_string()) {
            return init.arity;
        }
        return 0;
    }

    pub fn call(
        self,
        interpreter: &mut Interpreter,
        args: &Vec<Literal>,
    ) -> RuntimeResult<Literal> {
        let instance = Object::new(self);
        let init_function = instance.class.borrow().find_method(&"init".to_string());
        let wrapped_instance = Rc::new(RefCell::new(instance));
        if let Some(Literal::Fun(init)) = init_function {
            if let Literal::Fun(bound_init) =
                init.bind(Instance::Dynamic(Rc::clone(&wrapped_instance)))
            {
                bound_init.call(interpreter, args)?;
            }
        }
        return Ok(Literal::Instance(Instance::Dynamic(wrapped_instance)));
    }

    pub fn get(&self, name: &Token) -> RuntimeResult<Literal> {
        if let Some(Literal::Fun(method)) = self.find_method(&name.lexeme) {
            return Ok(method.bind(Instance::Static(Rc::new(RefCell::new(self.clone())))));
        }
        Err(RuntimeError::new(
            name.clone(),
            &format!("Undefined property '{}'.", name.lexeme),
        ))
    }

    pub fn find_method(&self, name: &String) -> Option<Literal> {
        match self.methods.get(name) {
            Some(method) => Some(method.clone()),
            None => None,
        }
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "<class {}>", self.name)
    }
}
