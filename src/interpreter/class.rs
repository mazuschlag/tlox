use crate::interpreter::instance::Instance;
use crate::interpreter::interpreter::{Interpreter, RuntimeResult};
use crate::lexer::literal::Literal;
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
        &self,
        interpreter: &mut Interpreter,
        args: &Vec<Literal>,
    ) -> RuntimeResult<Literal> {
        let instance = Rc::new(RefCell::new(Instance::new(self.clone())));
        if let Some(Literal::Fun(init)) = self.find_method(&"init".to_string()) {
            if let Literal::Fun(bound_init) = init.bind(Rc::clone(&instance)) {
                bound_init.call(interpreter, args)?;
            }
        }
        return Ok(Literal::Instance(instance));
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
