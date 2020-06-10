use crate::lexer::token::Token;
use crate::lexer::literal::Literal;
use crate::parser::expression::Expr;
use crate::interpreter::interpreter::{Interpreter};
use crate::parser::statement::{Stmt, Declarations};
use crate::error::report::error;
use std::collections::HashMap;

pub struct Resolver {
    interpreter: Interpreter,
    scopes: Vec<HashMap<String, bool>>
}

type ResolverError = Result<(), String>;

impl Resolver {
    fn new(&self, interpreter: Interpreter) -> Resolver {
        let scopes = Vec::new();
        Resolver {
            interpreter,
            scopes
        }
    }

    fn visit_block_stmt(&mut self, statements: &Declarations) -> ResolverError {
        self.begin_scope();
        self.resolve(statements)?;
        self.end_scope();
        Ok(())
    }

    fn resolve(&mut self, statements: &Declarations) -> ResolverError {
        for statement in statements {
            match statement {
                Stmt::Var(name, initializer) => self.visit_var_stmt(name, initializer)?,
                _ => ()
            };
        }
        Ok(())
    }
    
    fn visit_var_stmt(&mut self, name: &Token, initializer: &Expr) -> ResolverError {
        self.declare(name);
        match initializer {
            Expr::Literal(Literal::Nothing) => Ok(()),
            _ => self.visit_expr(initializer)
        }?;
        self.define(name);
        Ok(())
    }

    fn visit_expr(&mut self, expr: &Expr) -> ResolverError {
        match expr {
            Expr::Variable(name) => self.visit_variable_expr(name),
            _ => Ok(())
        }
    }

    fn visit_variable_expr(&mut self, name: &Token) -> ResolverError {
        if let Some(scope) = self.scopes.last() {
            if let Some(false) = scope.get(&name.lexeme) {
                return Err(error(name, "Cannot read local variable in its own initializer"))
            }
        }
        self.resolve_local(name);
        Ok(())
    }

    fn declare(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), false);
        }
    }

    fn define(&mut self, name: &Token) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.lexeme.clone(), true);
        }
    }

    fn begin_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn end_scope(&mut self) {
        self.scopes.pop();
    }

    fn resolve_local(&mut self, name: &Token) {
        for scope in self.scopes.iter().rev() {
            if scope.contains_key(&name.lexeme) {
                // Do something
            }
        }
    }
}