use crate::arena::pool::Pool;
use crate::error::report::error;
use crate::lexer::literal::Literal;
use crate::lexer::token::Token;

use std::collections::HashMap;

use super::expression::{Expr, ExprRef};
use super::statement::{Stmt, StmtRef};

pub struct Resolver {
    stmt_pool: Pool<Stmt>,
    expr_pool: Pool<Expr>,
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

pub struct ResolverOutput(pub HashMap<Token, usize>, pub Pool<Stmt>, pub Pool<Expr>, pub Vec<StmtRef>);

impl Resolver {
    pub fn new(stmt_pool: Pool<Stmt>, expr_pool: Pool<Expr>) -> Resolver {
        let scopes = Vec::new();
        let locals = HashMap::new();
        let current_function = FunctionType::NotAFunction;
        let current_class = ClassType::NotAClass;
        Resolver {
            stmt_pool,
            expr_pool,
            scopes,
            locals,
            current_function,
            current_class,
        }
    }

    pub fn run(mut self, program: Vec<StmtRef>) -> Result<ResolverOutput, String> {
        self.resolve(&program)?;
        Ok(ResolverOutput(self.locals, self.stmt_pool, self.expr_pool, program))
    }

    fn resolve(&mut self, statements: &Vec<StmtRef>) -> ResolverError {
        for statement in statements {
            self.visit_stmt(*statement)?;
        }
        Ok(())
    }

    fn visit_stmt(&mut self, statement: StmtRef) -> ResolverError {
        match &self.stmt_pool.get(statement) {
            Stmt::Block(body) => self.visit_block_stmt(body),
            Stmt::Var(name, initializer) => self.visit_var_stmt(name, *initializer),
            Stmt::Expression(expr) => self.visit_expression_stmt(*expr),
            Stmt::If(condition, then_branch, else_branch) => {
                self.visit_if_stmt(*condition, *then_branch, *else_branch)
            }
            Stmt::Print(expr) => self.visit_print_stmt(*expr),
            Stmt::Return(keyword, value) => self.visit_return_stmt(keyword, *value),
            Stmt::While(condition, body) => self.visit_while_stmt(*condition, *body),
            Stmt::Function(name, params, body) => self.visit_function_stmt(name, params, body),
            Stmt::Getter(name, body) => self.visit_getter_stmt(name, body),
            Stmt::Class(name, methods, super_class) => {
                self.visit_class_stmt(name, methods, *super_class)
            }
        }
    }

    fn visit_block_stmt(&mut self, statements: &Vec<StmtRef>) -> ResolverError {
        self.begin_scope();
        self.resolve(statements)?;
        self.end_scope();
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: ExprRef) -> ResolverError {
        self.declare(name)?;
        match self.expr_pool.get(initializer) {
            Expr::Literal(Literal::Nothing) => Ok(()),
            _ => self.visit_expr(initializer),
        }?;
        self.define(name);
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expr: ExprRef) -> ResolverError {
        self.visit_expr(expr)
    }

    fn visit_if_stmt(
        &mut self,
        condition: ExprRef,
        then_branch: StmtRef,
        else_branch: Option<StmtRef>,
    ) -> ResolverError {
        self.visit_expr(condition)?;
        self.visit_stmt(then_branch)?;
        if let Some(statement) = else_branch {
            self.visit_stmt(statement)?;
        }
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: ExprRef) -> ResolverError {
        self.visit_expr(expr)
    }

    fn visit_return_stmt(&mut self, keyword: &Token, value: ExprRef) -> ResolverError {
        if self.current_function == FunctionType::NotAFunction {
            return Err(error(keyword, "Cannot return from top-level code."));
        }
        if self.current_function == FunctionType::Initializer {
            return Err(error(keyword, "Cannot return a value from an initializer."));
        }
        match self.expr_pool.get(value) {
            Expr::Literal(Literal::Nothing) => Ok(()),
            _ => self.visit_expr(value),
        }
    }

    fn visit_while_stmt(&mut self, condition: ExprRef, body: StmtRef) -> ResolverError {
        self.visit_expr(condition)?;
        self.visit_stmt(body)
    }

    fn visit_function_stmt(
        &mut self,
        name: &Token,
        params: &Vec<Token>,
        body: &Vec<StmtRef>,
    ) -> ResolverError {
        self.declare(name)?;
        self.define(name);
        self.resolve_function(params, body, FunctionType::Function)
    }

    fn visit_getter_stmt(&mut self, name: &Token, body: &Vec<StmtRef>) -> ResolverError {
        self.declare(name)?;
        self.define(name);
        self.resolve_function(&Vec::new(), body, FunctionType::Method)
    }

    fn visit_class_stmt(
        &mut self,
        name: &Token,
        methods: &Vec<StmtRef>,
        super_class: Option<ExprRef>,
    ) -> ResolverError {
        let enclosing_class = self.current_class;
        self.current_class = ClassType::Class;
        self.declare(name)?;
        self.define(name);
        if let Some(expr) = super_class {
            if let Expr::Variable(super_class_name) = self.expr_pool.get(expr) {
                if super_class_name.lexeme == name.lexeme {
                    return Err(error(name, "A class can't inherit from itself."));
                }
            }
        }

        if let Some(class) = super_class {
            self.current_class = ClassType::SubClass;
            self.visit_expr(class)?;
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
            let stmt = self.stmt_pool.get(*method);
            if let Stmt::Function(_, params, body) = stmt {
                let declaration = if name.lexeme == "init" {
                    FunctionType::Initializer
                } else {
                    FunctionType::Method
                };
                self.resolve_function(&params, &body, declaration)?;
            }
        }
        self.end_scope();
        if let Some(_) = super_class {
            self.end_scope();
        }
        self.current_class = enclosing_class;
        Ok(())
    }

    fn visit_expr(&mut self, expr: ExprRef) -> ResolverError {
        match &self.expr_pool.get(expr) {
            Expr::Variable(var) => self.visit_variable_expr(var),
            Expr::Assign(name, value) => self.visit_assign_expr(name, *value),
            Expr::Binary(left, _, right) => self.visit_binary_expr(*left, *right),
            Expr::Logical(left, _, right) => self.visit_logical_expr(*left, *right),
            Expr::Ternary(left, middle, right) => self.visit_ternary_expr(*left, *middle, *right),
            Expr::Grouping(group) => self.visit_grouping_expr(*group),
            Expr::Unary(_, right) => self.visit_unary_expr(*right),
            Expr::Call(callee, _, arguments) => self.visit_call_expr(*callee, arguments),
            Expr::Lambda(args, body) => self.visit_lambda_expr(args, body),
            Expr::Get(object, _) => self.visit_get_expr(*object),
            Expr::Set(object, _, value) => self.visit_set_expr(*object, *value),
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

    fn visit_assign_expr(&mut self, name: &Token, value: ExprRef) -> ResolverError {
        self.visit_expr(value)?;
        self.resolve_local(name);
        Ok(())
    }

    fn visit_binary_expr(&mut self, left: ExprRef, right: ExprRef) -> ResolverError {
        self.visit_expr(left)?;
        self.visit_expr(right)
    }

    fn visit_logical_expr(&mut self, left: ExprRef, right: ExprRef) -> ResolverError {
        self.visit_expr(left)?;
        self.visit_expr(right)
    }

    fn visit_ternary_expr(&mut self, left: ExprRef, middle: ExprRef, right: ExprRef) -> ResolverError {
        self.visit_expr(left)?;
        self.visit_expr(middle)?;
        self.visit_expr(right)
    }

    fn visit_grouping_expr(&mut self, group: ExprRef) -> ResolverError {
        self.visit_expr(group)
    }

    fn visit_unary_expr(&mut self, expr: ExprRef) -> ResolverError {
        self.visit_expr(expr)
    }

    fn visit_call_expr(&mut self, callee: ExprRef, arguments: &Vec<ExprRef>) -> ResolverError {
        self.visit_expr(callee)?;
        for argument in arguments {
            self.visit_expr(*argument)?;
        }
        Ok(())
    }

    fn visit_lambda_expr(&mut self, params: &Vec<Token>, body: &Vec<StmtRef>) -> ResolverError {
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve(body)?;
        self.end_scope();
        Ok(())
    }

    fn visit_get_expr(&mut self, object: ExprRef) -> ResolverError {
        self.visit_expr(object)
    }

    fn visit_set_expr(&mut self, object: ExprRef, value: ExprRef) -> ResolverError {
        self.visit_expr(value)?;
        self.visit_expr(object)?;
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
    ) -> ResolverError {
        let enclosing_function = self.current_function;
        self.current_function = typ;
        self.begin_scope();
        for param in params {
            self.declare(param)?;
            self.define(param);
        }
        self.resolve(body)?;
        self.end_scope();
        self.current_function = enclosing_function;
        Ok(())
    }

    fn scopes_depth(&self) -> usize {
        self.scopes.len() - 1
    }
}
