use crate::lexer::token::Token;
use crate::lexer::literal::Literal;
use crate::parser::expression::Expr;
use crate::interpreter::interpreter::{Interpreter};
use crate::parser::statement::{Stmt, Declarations};
use crate::error::report::error;
use std::collections::HashMap;

pub struct Resolver<'a> {
    interpreter: &'a mut Interpreter,
    scopes: Vec<HashMap<String, bool>>
}

type ResolverError = Result<(), String>;

impl<'a> Resolver<'a> {
    pub fn new(interpreter: &mut Interpreter) -> Resolver {
        let scopes = Vec::new();
        Resolver {
            interpreter,
            scopes
        }
    }

    pub fn resolve(&mut self, statements: &Declarations) -> ResolverError {
        for statement in statements {
            self.visit_stmt(statement)?;
        }
        Ok(())
    }

    fn visit_stmt(&mut self, statement: &Stmt) -> ResolverError {
        match statement {
            Stmt::Block(body) => self.visit_block_stmt(body),
            Stmt::Var(name, initializer) => self.visit_var_stmt(name, initializer),
            Stmt::Expression(expr) => self.visit_expression_stmt(expr),
            Stmt::If(condition, then_branch, else_branch) => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::Print(expr) => self.visit_print_stmt(expr),
            Stmt::Return(keyword, value) => self.visit_return_stmt(keyword, value),
            Stmt::While(condition, body) => self.visit_while_stmt(condition, body),
            Stmt::Function(name, params, body) => self.visit_function_stmt(name, params, body),
            Stmt::Class(name, _) => self.visit_class_stmt(name)
        }
    }

    fn visit_block_stmt(&mut self, statements: &Declarations) -> ResolverError {
        self.begin_scope();
        self.resolve(statements)?;
        self.end_scope();
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

    fn visit_expression_stmt(&mut self, expr: &Expr) -> ResolverError {
        self.visit_expr(expr)
    }

    fn visit_if_stmt(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: &Option<Stmt>) -> ResolverError {
        self.visit_expr(condition)?;
        self.visit_stmt(then_branch)?;
        if let Some(statement) = else_branch {
            self.visit_stmt(statement)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> ResolverError {
        self.visit_expr(expr)
    }

    #[allow(unused_variables)]
    fn visit_return_stmt(&mut self, keyword: &Token, value: &Expr) -> ResolverError {
        match value {
            Expr::Literal(Literal::Nothing) => Ok(()),
            _ => self.visit_expr(value)
        }
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> ResolverError {
        self.visit_expr(condition)?;
        self.visit_stmt(body)
    }

    fn visit_function_stmt(&mut self, name: &Token, params: &Vec<Token>, body: &Declarations) -> ResolverError {
        self.declare(name);
        self.define(name);
        self.resolve_function(params, body)
    }

    fn visit_class_stmt(&mut self, name: &Token) -> ResolverError {
        self.declare(name);
        self.define(name);
        Ok(())
    }

    fn visit_expr(&mut self, expr: &Expr) -> ResolverError {
        match expr {
            Expr::Variable(var) => self.visit_variable_expr(var),
            Expr::Assign(name, value) => self.visit_assign_expr(name, value),
            Expr::Binary(left, _, right) => self.visit_binary_expr(left, right),
            Expr::Logical(left, _, right) => self.visit_logical_expr(left, right),
            Expr::Ternary(left, middle, right) => self.visit_ternary_expr(left, middle, right),
            Expr::Grouping(group) => self.visit_grouping_expr(group),
            Expr::Unary(_, right) => self.visit_unary_expr(right),
            Expr::Call(callee, _, arguments) => self.visit_call_expr(callee, arguments),
            Expr::Lambda(args, body) => self.visit_lambda_expr(args, body),
            Expr::Literal(_) => self.visit_literal()
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

    fn visit_assign_expr(&mut self, name: &Token, value: &Expr) -> ResolverError {
        self.visit_expr(value)?;
        self.resolve_local(name);
        Ok(())
    }

    fn visit_binary_expr(&mut self, left: &Expr, right: &Expr) -> ResolverError {
        self.visit_expr(left)?;
        self.visit_expr(right)
    }

    fn visit_logical_expr(&mut self, left: &Expr, right: &Expr) -> ResolverError {
        self.visit_expr(left)?;
        self.visit_expr(right)
    }

    fn visit_ternary_expr(&mut self, left: &Expr, middle: &Expr, right: &Expr) -> ResolverError {
        self.visit_expr(left)?;
        self.visit_expr(middle)?;
        self.visit_expr(right)
    }

    fn visit_grouping_expr(&mut self, group: &Expr) -> ResolverError {
        self.visit_expr(&group)
    }

    fn visit_unary_expr(&mut self, expr: &Expr) -> ResolverError {
        self.visit_expr(expr)
    }

    fn visit_call_expr(&mut self, callee: &Expr, arguments: &Vec<Box<Expr>>) -> ResolverError {
        self.visit_expr(callee)?;
        for argument in arguments {
            self.visit_expr(argument)?;
        }
        Ok(())
    }

    fn visit_lambda_expr(&mut self, params: &Vec<Token>, body: &Declarations) -> ResolverError {
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve(body)?;
        self.end_scope();
        Ok(())
    }

    fn visit_literal(&self) -> ResolverError {
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
        if self.scopes.len() > 0 {
            for i in (0..=self.scopes_depth()).rev() {
                if self.scopes[i].contains_key(&name.lexeme) {
                    self.interpreter.resolve(name, self.scopes_depth() - i)
                }
            }
        }   
    }

    fn resolve_function(&mut self, params: &Vec<Token>, body: &Declarations) -> ResolverError {
        self.begin_scope();
        for param in params {
            self.declare(param);
            self.define(param);
        }
        self.resolve(body)?;
        self.end_scope();
        Ok(())
    }

    fn scopes_depth(&self) -> usize {
        self.scopes.len() - 1
    }
}