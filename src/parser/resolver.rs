use crate::arena::pool::Pools;
use crate::error::report::error;
use crate::lexer::literal::Literal;
use crate::lexer::token::Token;

use std::collections::HashMap;

use super::expression::{Expr, ExprRef};
use super::statement::{Stmt, StmtRef};

pub struct Resolver {
    scopes: Vec<HashMap<String, bool>>,
    locals: HashMap<Token, usize>,
    current_function: FunctionType,
    current_class: ClassType,
}

type ResolverError = Result<(), String>;

#[derive(Debug, PartialEq, Clone, Copy)]
enum FunctionType {
    Function,
    Method,
    Initializer,
    NotAFunction,
}

#[derive(Debug, PartialEq, Clone, Copy)]
enum ClassType {
    NotAClass,
    Class,
    SubClass
}

impl Resolver {
    pub fn new() -> Resolver {
        let scopes = Vec::new();
        let locals = HashMap::new();
        let current_function = FunctionType::NotAFunction;
        let current_class = ClassType::NotAClass;
        Resolver {
            scopes,
            locals,
            current_function,
            current_class,
        }
    }

    pub fn run(mut self, program: Vec<StmtRef>, pools: Pools<Stmt, Expr>) -> Result<(Vec<StmtRef>, Pools<Stmt, Expr>, HashMap<Token, usize>), String> {
        self.resolve(&program, &pools)?;
        Ok((program, pools, self.locals))
    }

    fn resolve(&mut self, statements: &Vec<StmtRef>, pools: &Pools<Stmt, Expr>) -> ResolverError {
        for statement in statements {
            self.visit_stmt(*statement, pools)?;
        }
        Ok(())
    }

    fn visit_stmt(&mut self, statement: StmtRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        match pools.0.get(statement) {
            Stmt::Block(body) => self.visit_block_stmt(body, pools),
            Stmt::Var(name, initializer) => self.visit_var_stmt(name, *initializer, pools),
            Stmt::Expression(expr) => self.visit_expression_stmt(*expr, pools),
            Stmt::If(condition, then_branch, else_branch) => {
                self.visit_if_stmt(*condition, *then_branch, *else_branch, pools)
            }
            Stmt::Print(expr) => self.visit_print_stmt(*expr, pools),
            Stmt::Return(keyword, value) => self.visit_return_stmt(keyword, *value, pools),
            Stmt::While(condition, body) => self.visit_while_stmt(*condition, *body, pools),
            Stmt::Function(name, params, body) => self.visit_function_stmt(name, params, body, pools),
            Stmt::Getter(name, body) => self.visit_getter_stmt(name, body, pools),
            Stmt::Class(name, methods, super_class) => {
                self.visit_class_stmt(name, methods, *super_class, pools)
            }
        }
    }

    fn visit_block_stmt(&mut self, statements: &Vec<StmtRef>, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.begin_scope();
        self.resolve(statements, pools)?;
        self.end_scope();
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.declare(name)?;
        match pools.1.get(initializer) {
            Expr::Literal(Literal::Nothing) => Ok(()),
            _ => self.visit_expr(initializer, pools),
        }?;
        self.define(name);
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expr: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.visit_expr(expr, pools)
    }

    fn visit_if_stmt(
        &mut self,
        condition: ExprRef,
        then_branch: StmtRef,
        else_branch: Option<StmtRef>,
        pools: &Pools<Stmt, Expr>,
    ) -> ResolverError {
        self.visit_expr(condition, pools)?;
        self.visit_stmt(then_branch, pools)?;
        if let Some(statement) = else_branch {
            self.visit_stmt(statement, pools)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.visit_expr(expr, pools)
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        if self.current_function == FunctionType::NotAFunction {
            return Err(error(keyword, "Cannot return from top-level code."));
        }
        if self.current_function == FunctionType::Initializer {
            return Err(error(keyword, "Cannot return a value from an initializer."));
        }
        match pools.1.get(value) {
            Expr::Literal(Literal::Nothing) => Ok(()),
            _ => self.visit_expr(value, pools),
        }
    }

    fn visit_while_stmt(&mut self, condition: ExprRef, body: StmtRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.visit_expr(condition, pools)?;
        self.visit_stmt(body, pools)
    }

    fn visit_function_stmt(
        &mut self,
        name: &Token,
        params: &Vec<Token>,
        body: &Vec<StmtRef>,
        pools: &Pools<Stmt, Expr>,
    ) -> ResolverError {
        self.declare(name)?;
        self.define(name);
        self.resolve_function(params, body, FunctionType::Function, pools)
    }

    fn visit_getter_stmt(&mut self, name: &Token, body: &Vec<StmtRef>, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.declare(name)?;
        self.define(name);
        self.resolve_function(&Vec::new(), body, FunctionType::Method, pools)
    }

    fn visit_class_stmt(
        &mut self,
        name: &Token,
        methods: &Vec<StmtRef>,
        super_class: Option<ExprRef>,
        pools: &Pools<Stmt, Expr>
    ) -> ResolverError {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;
        self.declare(name)?;
        self.define(name);
        if let Some(expr) = super_class {
            if let Expr::Variable(super_class_name) = pools.1.get(expr) {
                if super_class_name.lexeme == name.lexeme {
                    return Err(error(name, "A class can't inherit from itself."));
                }
            }
        }

        if let Some(class) = super_class {
            self.current_class = ClassType::SubClass;
            self.visit_expr(class, pools)?;
        }

        if let Some(_) = super_class {
            self.begin_scope();
            if let Some(scope) = self.scopes.last_mut() {
                scope.insert("super".to_string(), true);
            }
        }

        self.begin_scope();
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert("this".to_string(), true);
        }

        for method in methods {
            let stmt = pools.0.get(*method);
            if let Stmt::Function(_, params, body) = stmt {
                let declaration = if name.lexeme == "init" {
                    FunctionType::Initializer
                } else {
                    FunctionType::Method
                };
                self.resolve_function(&params, &body, declaration, pools)?;
            }
        }
        self.end_scope();
        if let Some(_) = super_class {
            self.end_scope();
        }
        self.current_class = enclosing_class;
        Ok(())
    }

    fn visit_expr(&mut self, expr: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        match &pools.1.get(expr) {
            Expr::Variable(var) => self.visit_variable_expr(var),
            Expr::Assign(name, value) => self.visit_assign_expr(name, *value, pools),
            Expr::Binary(left, _, right) => self.visit_binary_expr(*left, *right, pools),
            Expr::Logical(left, _, right) => self.visit_logical_expr(*left, *right, pools),
            Expr::Ternary(left, middle, right) => self.visit_ternary_expr(*left, *middle, *right, pools),
            Expr::Grouping(group) => self.visit_grouping_expr(*group, pools),
            Expr::Unary(_, right) => self.visit_unary_expr(*right, pools),
            Expr::Call(callee, _, arguments) => self.visit_call_expr(*callee, arguments, pools),
            Expr::Lambda(args, body) => self.visit_lambda_expr(args, body, pools),
            Expr::Get(object, _) => self.visit_get_expr(*object, pools),
            Expr::Set(object, _, value) => self.visit_set_expr(*object, *value, pools),
            Expr::This(name) => self.visit_this_expr(name),
            Expr::Super(keyword, _) => self.visit_super_expr(keyword),
            Expr::Literal(_) => self.visit_literal(),
        }
    }

    fn visit_variable_expr(&mut self, name: &Token) -> ResolverError {
        if let Some(scope) = self.scopes.last() {
            if let Some(false) = scope.get(&name.lexeme) {
                return Err(error(
                    name,
                    "Cannot read local variable in its own initializer.",
                ));
            }
        }
        self.resolve_local(name);
        Ok(())
    }

    fn visit_assign_expr(&mut self, name: &Token, value: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.visit_expr(value, pools)?;
        self.resolve_local(name);
        Ok(())
    }

    fn visit_binary_expr(&mut self, left: ExprRef, right: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.visit_expr(left, pools)?;
        self.visit_expr(right, pools)
    }

    fn visit_logical_expr(&mut self, left: ExprRef, right: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.visit_expr(left, pools)?;
        self.visit_expr(right, pools)
    }

    fn visit_ternary_expr(&mut self, left: ExprRef, middle: ExprRef, right: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.visit_expr(left, pools)?;
        self.visit_expr(middle, pools)?;
        self.visit_expr(right, pools)
    }

    fn visit_grouping_expr(&mut self, group: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.visit_expr(group, pools)
    }

    fn visit_unary_expr(&mut self, expr: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.visit_expr(expr, pools)
    }

    fn visit_call_expr(&mut self, callee: ExprRef, arguments: &Vec<ExprRef>, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.visit_expr(callee, pools)?;
        for argument in arguments {
            self.visit_expr(*argument, pools)?;
        }
        Ok(())
    }

    fn visit_lambda_expr(&mut self, params: &Vec<Token>, body: &Vec<StmtRef>, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve(body, pools)?;
        self.end_scope();
        Ok(())
    }

    fn visit_get_expr(&mut self, object: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.visit_expr(object, pools)
    }

    fn visit_set_expr(&mut self, object: ExprRef, value: ExprRef, pools: &Pools<Stmt, Expr>) -> ResolverError {
        self.visit_expr(value, pools)?;
        self.visit_expr(object, pools)?;
        Ok(())
    }

    fn visit_this_expr(&mut self, name: &Token) -> ResolverError {
        if self.current_class == ClassType::NotAClass {
            return Err(error(name, "Cannot use 'this' outside of a class."));
        }
        self.resolve_local(name);
        Ok(())
    }

    fn visit_super_expr(&mut self, keyword: &Token) -> ResolverError {
        return match self.current_class {
            ClassType::Class => Err(error(keyword, "Can't use 'super' in a class with no superclass.")),
            ClassType::NotAClass => Err(error(keyword, "Can't use 'super' outside of a class.")),
            ClassType::SubClass => {
                self.resolve_local(keyword);
                Ok(())
            }
        }
    }

    fn visit_literal(&self) -> ResolverError {
        Ok(())
    }

    fn declare(&mut self, name: &Token) -> ResolverError {
        if let Some(scope) = self.scopes.last_mut() {
            if scope.contains_key(&name.lexeme) {
                return Err(error(
                    name,
                    "Variable with this name already declared in this scope.",
                ));
            }
            scope.insert(name.lexeme.clone(), false);
        }
        Ok(())
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
                    let depth = self.scopes_depth() - i;
                    self.locals.insert(name.clone(), depth);
                }
            }
        }
    }

    fn resolve_function(
        &mut self,
        params: &Vec<Token>,
        body: &Vec<StmtRef>,
        typ: FunctionType,
        pools: &Pools<Stmt, Expr>,
    ) -> ResolverError {
        let enclosing_function = self.current_function;
        self.current_function = typ;
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve(body, pools)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn scopes_depth(&self) -> usize {
        self.scopes.len() - 1
    }
}
