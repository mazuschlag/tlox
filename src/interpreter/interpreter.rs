use crate::arena::pool::Pool;
use crate::error::report::{runtime_report, RuntimeError};
use crate::interpreter::class::Class;
use crate::interpreter::environment::Environment;
use crate::interpreter::function::Function;
use crate::lexer::literal::{Instance, Literal};
use crate::lexer::token::{Token, TokenType};
use crate::parser::expression::{ExprRef, Expr};
use crate::parser::statement::{StmtRef, Stmt};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

pub struct Interpreter {
    pub globals: Rc<RefCell<Environment>>,
    pub environment: Rc<RefCell<Environment>>,
    pub in_initializer: bool,
    locals: HashMap<Token, usize>,
    stmt_pool: Pool<Stmt>,
    expr_pool: Pool<Expr>,
    return_value: Literal,
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

impl Interpreter {
    pub fn new(locals: HashMap<Token, usize>, stmt_pool: Pool<Stmt>, expr_pool: Pool<Expr>) -> Interpreter {
        let globals = Rc::new(RefCell::new(Environment::new(None)));
        let environment = Rc::clone(&globals);
        let return_value = Literal::Nothing;
        let in_initializer = false;
        Interpreter {
            globals,
            environment,
            in_initializer,
            locals,
            stmt_pool,
            expr_pool,
            return_value,
        }
    }

    pub fn run(mut self, program: Vec<StmtRef>) {
        for stmt in program {
            let result = self
                .visit_stmt(stmt)
                .map_err(|err| runtime_report(err))
                .err();
            if let Some(e) = result {
                println!("{}", e);
            }
        }
    }

    fn visit_stmt(&mut self, stmt: StmtRef) -> RuntimeResult<()> {
        match &self.stmt_pool.get(stmt) {
            Stmt::Expression(expr) => self.visit_expression_stmt(*expr),
            Stmt::Print(expr) => self.visit_print_stmt(*expr),
            Stmt::Var(name, expr) => self.visit_var_stmt(name, *expr),
            Stmt::Block(statements) => self.visit_block_stmt(statements, None),
            Stmt::If(condition, then_branch, else_branch) => {
                self.visit_if_stmt(*condition, *then_branch, *else_branch)
            }
            Stmt::While(condition, body) => self.visit_while_stmt(*condition, *body),
            Stmt::Function(name, params, body) => self.visit_function_stmt(name, params, body),
            Stmt::Getter(name, _) => {
                return Err(RuntimeError::new(
                    name.clone(),
                    &format!("{} getter require a class.", name.lexeme),
                ))
            }
            Stmt::Return(keyword, value) => self.visit_return_stmt(keyword, *value),
            Stmt::Class(name, methods, super_class) => {
                self.visit_class_stmt(name, methods, *super_class)
            }
        }?;
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expr: ExprRef) -> RuntimeResult<()> {
        if self.return_value != Literal::Nothing {
            return Ok(());
        }
        self.visit_expr(expr)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: ExprRef) -> RuntimeResult<()> {
        if self.return_value != Literal::Nothing {
            return Ok(());
        }
        let value = self.visit_expr(expr)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: ExprRef) -> RuntimeResult<()> {
        if self.return_value != Literal::Nothing {
            return Ok(());
        }
        let value = match self.expr_pool.get(initializer) {
            Expr::Literal(Literal::Nothing) => Literal::Nothing,
            _ => self.visit_expr(initializer)?,
        };
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), value);
        return Ok(());
    }

    pub fn visit_block_stmt(
        &mut self,
        statements: &Vec<StmtRef>,
        environment: Option<Environment>,
    ) -> RuntimeResult<()> {
        //dbg!(statements);
        if self.return_value != Literal::Nothing {
            return Ok(());
        }
        let previous = Rc::clone(&self.environment);
        self.environment = match environment {
            Some(env) => Rc::new(RefCell::new(env)),
            None => Rc::new(RefCell::new(Environment::new(Some(Rc::clone(
                &self.environment,
            ))))),
        };
        for statement in statements {
            if let Some(err) = self.visit_stmt(*statement).err() {
                self.environment = previous;
                return Err(err);
            }
            if self.return_value != Literal::Nothing {
                self.environment = previous;
                return Ok(());
            }
        }
        self.environment = previous;
        Ok(())
    }

    fn visit_if_stmt(
        &mut self,
        condition: ExprRef,
        then_branch: StmtRef,
        else_branch: Option<StmtRef>,
    ) -> RuntimeResult<()> {
        if self.return_value != Literal::Nothing {
            return Ok(());
        }
        let result = self.visit_expr(condition)?;
        if self.is_truthy(&result) {
            return self.visit_stmt(then_branch);
        }
        if let Some(else_branch) = else_branch {
            return self.visit_stmt(else_branch);
        }
        return Ok(());
    }

    fn visit_while_stmt(&mut self, condition: ExprRef, body: StmtRef) -> RuntimeResult<()> {
        let mut result = self.visit_expr(condition)?;
        while self.is_truthy(&result) {
            self.visit_stmt(body)?;
            if self.return_value != Literal::Nothing {
                return Ok(());
            }
            result = self.visit_expr(condition)?;
        }
        Ok(())
    }

    fn visit_function_stmt(
        &mut self,
        name: &Token,
        params: &Vec<Token>,
        body: &Vec<StmtRef>,
    ) -> RuntimeResult<()> {
        let function = Function::new(
            Some(name.clone()),
            params.clone(),
            body.clone(),
            &self.environment,
            false,
        );
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), Literal::Fun(function));
        Ok(())
    }

    #[allow(unused_variables)]
    fn visit_return_stmt(&mut self, keyword: &Token, value: ExprRef) -> RuntimeResult<()> {
        if self.in_initializer {
            self.return_value = match self.environment.borrow().get_at(&"this".to_string(), 0) {
                Some(value) => value,
                None => Literal::Nothing,
            };
            return Ok(());
        }
        self.return_value = match self.expr_pool.get(value) {
            Expr::Literal(Literal::Nothing) => Literal::Nothing,
            _ => self.visit_expr(value)?,
        };
        Ok(())
    }

    fn visit_class_stmt(
        &mut self,
        name: &Token,
        methods: &Vec<StmtRef>,
        super_class: Option<ExprRef>,
    ) -> RuntimeResult<()> {
        let parent_class = if let Some(super_class) = super_class {
            match self.visit_expr(super_class)? {
                Literal::Class(class) => Some(Rc::new(RefCell::new(class))),
                _ => {
                    return Err(RuntimeError::new(
                        name.clone(),
                        &format!("Superclass must be a class."),
                    ))
                }
            }
        } else {
            None
        };
        self.environment
            .borrow_mut()
            .define(name.lexeme.clone(), Literal::Nothing);
        if let Some(class) = super_class {
            self.environment = Rc::new(RefCell::new(Environment::new(Some(Rc::clone(&self.environment)))));
            let parent_class = self.visit_expr(class)?;
            self.environment.borrow_mut().define("super".to_string(), parent_class);
        }

        let mut class_methods = HashMap::new();
        for method in methods {
            let stmt = self.stmt_pool.get(*method);
            if let Stmt::Function(name, params, body) = stmt {
                let function = Literal::Fun(Function::new(
                    Some(name.clone()),
                    params.clone(),
                    body.clone(),
                    &self.environment,
                    name.lexeme == "init",
                ));
                class_methods.insert(name.lexeme.clone(), function);
            } else if let Stmt::Getter(name, body) = stmt {
                let function = Literal::Get(Function::new(
                    Some(name.clone()),
                    Vec::new(),
                    body.clone(),
                    &self.environment,
                    false,
                ));
                class_methods.insert(name.lexeme.clone(), function);
            }
        }
        let class = Literal::Class(Class::new(name.lexeme.clone(), class_methods, parent_class));
        if let Some(_) = super_class {
            let current_env = Rc::clone(&self.environment);
            self.environment = match &current_env.borrow().outer_scope {
                Some(enclosing) => Rc::clone(&enclosing),
                None => Rc::clone(&self.globals)
            };
        }

        self.environment.borrow_mut().assign(name, class)
    }

    fn visit_expr(&mut self, expr: ExprRef) -> RuntimeResult<Literal> {
        match &self.expr_pool.get(expr) {
            Expr::Assign(name, value) => self.visit_assign_expr(name, *value),
            Expr::Variable(var) => self.visit_var_expr(var),
            Expr::Binary(left, operator, right) => self.visit_binary_expr(*left, operator, *right),
            Expr::Logical(left, operator, right) => self.visit_logical_expr(*left, operator, *right),
            Expr::Ternary(left, middle, right) => self.visit_ternary_expr(*left, *middle, *right),
            Expr::Grouping(group) => self.visit_grouping_expr(*group),
            Expr::Unary(operator, right) => self.visit_unary_expr(operator, *right),
            Expr::Call(callee, right_paren, arguments) => {
                self.visit_call_expr(*callee, right_paren, arguments)
            }
            Expr::Lambda(args, body) => self.visit_lambda_expr(args, body),
            Expr::Get(instance, name) => self.visit_get_expr(*instance, name),
            Expr::Set(object, name, value) => self.visit_set_expr(*object, name, *value),
            Expr::This(name) => self.visit_this_expr(name),
            Expr::Super(keyword, method) => self.visit_super_expr(keyword, method),
            Expr::Literal(value) => self.visit_literal(value.clone()),
        }
    }

    fn visit_assign_expr(&mut self, name: &Token, initializer: ExprRef) -> RuntimeResult<Literal> {
        let value = self.visit_expr(initializer)?;
        let distance = self.locals.get(name);
        match distance {
            Some(d) => self
                .environment
                .borrow_mut()
                .assign_at(name, value.clone(), *d)?,
            None => self.globals.borrow_mut().assign(name, value.clone())?,
        }
        Ok(value)
    }

    fn visit_var_expr(&self, name: &Token) -> RuntimeResult<Literal> {
        let distance = self.locals.get(name);
        let value = match distance {
            Some(d) => self.environment.borrow().get_at(&name.lexeme, *d),
            None => self.globals.borrow().get(&name.lexeme),
        };
        match value {
            Some(v) => Ok(v.clone()),
            None => Err(RuntimeError::new(
                name.clone(),
                &format!("Undefined variable '{}'.", name.lexeme),
            )),
        }
    }

    fn visit_binary_expr(
        &mut self,
        left: ExprRef,
        operator: &Token,
        right: ExprRef,
    ) -> RuntimeResult<Literal> {
        let l = self.visit_expr(left)?;
        let r = self.visit_expr(right)?;

        match operator.typ {
            TokenType::Minus | TokenType::Slash | TokenType::Star => {
                self.calculate_number(&l, operator, &r)
            }
            TokenType::Plus => self.calculate_addition(&l, operator, &r),
            TokenType::Greater
            | TokenType::GreaterEqual
            | TokenType::Less
            | TokenType::LessEqual => self.calculate_bool(&l, operator, &r),
            TokenType::BangEqual => Ok(Literal::Bool(!self.is_equal(l, r))),
            TokenType::EqualEqual => Ok(Literal::Bool(self.is_equal(l, r))),
            TokenType::Comma => Ok(r),
            _ => Err(RuntimeError::new(
                operator.clone(),
                "Unknown binary operator.",
            )),
        }
    }

    fn visit_logical_expr(
        &mut self,
        left: ExprRef,
        operator: &Token,
        right: ExprRef,
    ) -> RuntimeResult<Literal> {
        let left = self.visit_expr(left)?;
        if operator.typ == TokenType::Or {
            if self.is_truthy(&left) {
                return Ok(left);
            }
        } else {
            if !self.is_truthy(&left) {
                return Ok(left);
            }
        }
        self.visit_expr(right)
    }

    fn visit_ternary_expr(
        &mut self,
        left: ExprRef,
        middle: ExprRef,
        right: ExprRef,
    ) -> RuntimeResult<Literal> {
        let l = self.visit_expr(left)?;
        match self.is_truthy(&l) {
            true => return self.visit_expr(middle),
            false => return self.visit_expr(right),
        }
    }

    fn visit_grouping_expr(&mut self, group: ExprRef) -> RuntimeResult<Literal> {
        self.visit_expr(group)
    }

    fn visit_unary_expr(&mut self, operator: &Token, expr: ExprRef) -> RuntimeResult<Literal> {
        let right = self.visit_expr(expr)?;
        match operator.typ {
            TokenType::Minus => {
                if let Literal::Number(num) = right {
                    return Ok(Literal::Number(-num));
                } else {
                    return Err(RuntimeError::new(
                        operator.clone(),
                        "Cannot make non-number negative.",
                    ));
                }
            }
            TokenType::Bang => return Ok(Literal::Bool(!self.is_truthy(&right))),
            _ => {
                return Err(RuntimeError::new(
                    operator.clone(),
                    "Uknown unary operator.",
                ))
            }
        }
    }

    fn visit_call_expr(
        &mut self,
        callee: ExprRef,
        right_paren: &Token,
        arguments: &Vec<ExprRef>,
    ) -> RuntimeResult<Literal> {
        let callee = self.visit_expr(callee)?;
        let mut evaluated_args = Vec::new();
        for arg in arguments {
            evaluated_args.push(self.visit_expr(*arg)?);
        }
        match callee {
            Literal::Fun(function) => {
                if arguments.len() != function.arity {
                    return Err(RuntimeError::new(
                        right_paren.clone(),
                        "Wrong number of arguments.",
                    ));
                }
                function.call(self, &evaluated_args)?;
                let value = self.return_value.clone();
                self.return_value = Literal::Nothing;
                return Ok(value);
            }
            Literal::Class(class) => class.call(self, &evaluated_args),
            _ => Err(RuntimeError::new(
                right_paren.clone(),
                "Can only call functions and classes.",
            )),
        }
    }

    fn visit_lambda_expr(
        &self,
        params: &Vec<Token>,
        body: &Vec<StmtRef>,
    ) -> RuntimeResult<Literal> {
        let function = Function::new(None, params.clone(), body.clone(), &self.environment, false);
        Ok(Literal::Fun(function))
    }

    fn visit_get_expr(&mut self, expr: ExprRef, name: &Token) -> RuntimeResult<Literal> {
        let instance = self.visit_expr(expr)?;
        if let Literal::Instance(Instance::Dynamic(object)) = instance {
            // This is grabbing the wrong function
            let result = object.borrow().get(name)?;
            if let Literal::Get(getter) = result {
                getter.call(self, &Vec::new())?;
                let value = self.return_value.clone();
                self.return_value = Literal::Nothing;
                return Ok(value);
            }
            if let Literal::Fun(fun) = result {
                dbg!(&fun.name);
                return Ok(Literal::Fun(fun));
            }
            return Ok(result)
        }
        if let Literal::Class(class) = instance {
            return class.get(name);
        }
        Err(RuntimeError::new(
            name.clone(),
            "Only instances have properties.",
        ))
    }

    fn visit_set_expr(
        &mut self,
        left: ExprRef,
        name: &Token,
        right: ExprRef,
    ) -> RuntimeResult<Literal> {
        let instance = self.visit_expr(left)?;
        if let Literal::Instance(Instance::Dynamic(object)) = instance {
            let value = self.visit_expr(right)?;
            let result = object.borrow_mut().set(name, value)?;
            return Ok(result);
        }
        Err(RuntimeError::new(
            name.clone(),
            "Only instances have properties.",
        ))
    }

    fn visit_this_expr(&mut self, name: &Token) -> RuntimeResult<Literal> {
        let distance = self.locals.get(name);
        let value = match distance {
            Some(d) => self.environment.borrow().get_at(&name.lexeme, *d),
            None => self.globals.borrow().get(&name.lexeme),
        };
        match value {
            Some(v) => Ok(v.clone()),
            None => Err(RuntimeError::new(
                name.clone(),
                &format!("Undefined variable '{}'.", name.lexeme),
            )),
        }
    }

    fn visit_super_expr(&mut self, keyword: &Token, method: &Token) -> RuntimeResult<Literal> {
        let distance = self.locals.get(keyword);
        if let Some(d) = distance {
            let super_class = self.environment.borrow().get_at(&"super".to_string(), *d);
            let object = self.environment.borrow().get_at(&"this".to_string(), d - 1);
            if let Some(Literal::Class(class)) = super_class {
                if let Some(Literal::Instance(instance)) = object {
                    let method = class.find_method(&method.lexeme);
                    if let Some(Literal::Fun(function)) = method {
                        return Ok(function.bind(instance, false));
                    }
                }
            }
        }
        Err(RuntimeError::new(method.clone(), &format!("Undefined property '{}'.", method.lexeme)))
    }

    fn visit_literal(&self, value: Literal) -> RuntimeResult<Literal> {
        Ok(value)
    }

    fn calculate_addition(
        &self,
        left: &Literal,
        operator: &Token,
        right: &Literal,
    ) -> RuntimeResult<Literal> {
        match left {
            Literal::Number(l) => match right {
                Literal::Number(r) => return Ok(Literal::Number(r + l)),
                Literal::Str(r) => return Ok(Literal::Str(format!("{}{}", l, r))),
                _ => return Err(RuntimeError::new(operator.clone(), "Cannot add operands.")),
            },
            Literal::Str(l) => match right {
                Literal::Number(r) => return Ok(Literal::Str(format!("{}{}", l, r))),
                Literal::Str(r) => return Ok(Literal::Str(format!("{}{}", l, r))),
                _ => return Err(RuntimeError::new(operator.clone(), "Cannot add operands.")),
            },
            _ => Err(RuntimeError::new(operator.clone(), "Cannot add operands.")),
        }
    }

    fn calculate_number(
        &self,
        left: &Literal,
        operator: &Token,
        right: &Literal,
    ) -> RuntimeResult<Literal> {
        if let Literal::Number(l) = left {
            if let Literal::Number(r) = right {
                match operator.typ {
                    TokenType::Minus => return Ok(Literal::Number(l - r)),
                    TokenType::Star => return Ok(Literal::Number(l * r)),
                    TokenType::Slash => {
                        if *r == 0.0 {
                            return Err(RuntimeError::new(
                                operator.clone(),
                                "Cannot divide by zero.",
                            ));
                        }
                        return Ok(Literal::Number(l / r));
                    }
                    _ => {
                        return Err(RuntimeError::new(
                            operator.clone(),
                            "Uknown operator for numbers.",
                        ))
                    }
                }
            }
        }
        Err(RuntimeError::new(
            operator.clone(),
            "Operand must be a number",
        ))
    }

    fn calculate_bool(
        &self,
        left: &Literal,
        operator: &Token,
        right: &Literal,
    ) -> RuntimeResult<Literal> {
        if let Literal::Number(l) = left {
            if let Literal::Number(r) = right {
                match operator.typ {
                    TokenType::Greater => return Ok(Literal::Bool(l > r)),
                    TokenType::GreaterEqual => return Ok(Literal::Bool(l >= r)),
                    TokenType::Less => return Ok(Literal::Bool(l < r)),
                    TokenType::LessEqual => return Ok(Literal::Bool(l <= r)),
                    _ => {
                        return Err(RuntimeError::new(
                            operator.clone(),
                            "Uknown operator for numbers.",
                        ))
                    }
                }
            }
        }
        Err(RuntimeError::new(
            operator.clone(),
            "Cannot compare non-booleans.",
        ))
    }

    fn is_truthy(&self, value: &Literal) -> bool {
        match value {
            Literal::Nothing => false,
            Literal::Bool(b) => *b,
            _ => true,
        }
    }

    fn is_equal(&self, left: Literal, right: Literal) -> bool {
        left == right
    }
}
