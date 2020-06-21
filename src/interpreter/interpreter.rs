use crate::parser::expression::{Expression, Expr};
use crate::parser::statement::{Stmt, Declarations};
use crate::error::report::{runtime_report, RuntimeError};
use crate::lexer::token::{Token, TokenType};
use crate::interpreter::environment::Environment;
use crate::interpreter::function::Function;
use crate::lexer::literal::Literal;
use std::collections::HashMap;

pub struct Interpreter {
    pub globals: Environment,
    pub environments: Vec<Environment>,
    locals: HashMap<Token, usize>,
    return_value: Literal,
    current_depth: usize
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

impl Interpreter {
    pub fn new() -> Interpreter {
        let globals =  Environment::new(0);
        let environments = Vec::new();
        let locals = HashMap::new();
        let return_value = Literal::Nothing;
        let current_depth = 0;
        Interpreter {
            globals,
            environments,
            locals,
            return_value,
            current_depth
        }
    }

    pub fn resolve(&mut self, name: &Token, depth: usize) {
        self.locals.insert(name.clone(), depth);
    }

    pub fn interpret(&mut self, program: &Declarations) {
        dbg!(&self.locals);
        println!("");
        for stmt in program {
            let result = self.visit_stmt(stmt).map_err(|err| runtime_report(err)).err();
            if let Some(e) = result {
                println!("{}", e);
            }
        }
    }

    fn visit_stmt(&mut self, stmt: &Stmt) -> RuntimeResult<()> {
        match stmt {
            Stmt::Expression(expr) => self.visit_expression_stmt(expr),
            Stmt::Print(expr) => self.visit_print_stmt(expr),
            Stmt::Var(name, expr) => self.visit_var_stmt(name, expr),
            Stmt::Block(statements) => self.visit_block_stmt(statements, None),
            Stmt::If(condition, then_branch, else_branch) => self.visit_if_stmt(condition, then_branch, else_branch),
            Stmt::While(condition, body) => self.visit_while_stmt(condition, body),
            Stmt::Function(name, params, body) => self.visit_function_stmt(name, params, body),
            Stmt::Return(keyword, value) => self.visit_return_stmt(keyword, value)
        }?;
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) -> RuntimeResult<()> {
        if self.return_value != Literal::Nothing {
            return Ok(())
        }
        self.visit_expr(expr)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> RuntimeResult<()> {
        if self.return_value != Literal::Nothing {
            return Ok(())
        }
        let value = self.visit_expr(expr)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Expr) -> RuntimeResult<()> {
        if self.return_value != Literal::Nothing {
            return Ok(())
        }
        let value = match *initializer {
            Expr::Literal(Literal::Nothing) => Literal::Nothing,
            _ => self.visit_expr(initializer)?
        };

        if self.current_depth > 0 {
            self.environments[self.current_depth-1].define(name.lexeme.clone(), value);
        } else {
            self.globals.define(name.lexeme.clone(), value)
        }
        Ok(())
    }

    pub fn visit_block_stmt(&mut self, statements: &Declarations, environment: Option<Environment>) -> RuntimeResult<()> {
        if self.return_value != Literal::Nothing {
            return Ok(())
        }

        match environment {
            Some(env) => self.environments.push(env),
            None => {
                self.environments.push(Environment::new(self.environments.len()));
            }
        }
        self.current_depth += 1;

        for statement in statements {
            if let Some(err) = self.visit_stmt(statement).err() {
                self.current_depth -= 1;
                return Err(err)
            }
            if self.return_value != Literal::Nothing {
                self.current_depth -= 1;
                return Ok(())
            }
        }
        self.current_depth -= 1;
        Ok(())
    }

    fn visit_if_stmt(&mut self, condition: &Expr, then_branch: &Stmt, else_branch: &Option<Stmt>) -> RuntimeResult<()> {
        if self.return_value != Literal::Nothing {
            return Ok(())
        }
        let result = self.visit_expr(condition)?;
        if self.is_truthy(&result) {
            return self.visit_stmt(then_branch)
        }
        if let Some(else_branch) = else_branch {
            return self.visit_stmt(else_branch)
        }
        Ok(())
    }

    fn visit_while_stmt(&mut self, condition: &Expr, body: &Stmt) -> RuntimeResult<()> {
        let mut result = self.visit_expr(condition)?;
        while self.is_truthy(&result) {
            self.visit_stmt(&body)?;
            if self.return_value != Literal::Nothing {
                return Ok(())
            }
            result = self.visit_expr(&condition)?;
        }
        Ok(())
    }

    fn visit_function_stmt(&mut self, name: &Token, params: &Vec<Token>, body: &Declarations) -> RuntimeResult<()> {
        let function = Function::new(Some(name.clone()), params.clone(), body.clone());
        if self.current_depth > 0 {
            self.environments[self.current_depth-1].define(name.lexeme.clone(), Literal::Fun(function));
        } else {
            self.globals.define(name.lexeme.clone(), Literal::Fun(function));
        }
        Ok(())
    }

    #[allow(unused_variables)]
    fn visit_return_stmt(&mut self, keyword: &Token, value: &Expr) -> RuntimeResult<()> {
        self.return_value = match value {
            Expr::Literal(Literal::Nothing) => Literal::Nothing,
            _ => self.visit_expr(value)?
        };
        Ok(())
    }

    fn visit_expr(&mut self, expr: &Expr) -> RuntimeResult<Literal> {
        match expr {
            Expr::Assign(name, value) => self.visit_assign_expr(name, value),
            Expr::Variable(var) => self.visit_var_expr(var),
            Expr::Binary(left, operator, right) => self.visit_binary_expr(left, operator, right),
            Expr::Logical(left, operator, right) => self.visit_logical_expr(left, operator, right),
            Expr::Ternary(left, middle, right) => self.visit_ternary_expr(left, middle, right),
            Expr::Grouping(group) => self.visit_grouping_expr(group),
            Expr::Unary(operator, right) => self.visit_unary_expr(operator, right),
            Expr::Call(callee, right_paren, arguments) => self.visit_call_expr(callee, right_paren, arguments),
            Expr::Lambda(args, body) => self.visit_lambda_expr(args, body),
            Expr::Literal(value) => self.visit_literal(value.clone())
        }
    }

    fn visit_assign_expr(&mut self, name: &Token, initializer: &Expr) -> RuntimeResult<Literal> {
        let value = self.visit_expr(initializer)?;
        let distance = self.locals.get(&name);
        match distance {
            Some(d) => {
                let depth = self.environments.len() - *d;
                self.environments[depth].assign(name, value.clone());
                Ok(value)
            },
            None => {
                if self.globals.assign(name, value.clone()) {
                    return Ok(value)
                }
                Err(RuntimeError::new(name.clone(), &format!("Undefined variable '{}'.", name.lexeme)))
            }
        }
    }

    fn visit_var_expr(&self, name: &Token) -> RuntimeResult<Literal> {
        self.look_up_variable(name)
    }

    fn look_up_variable(&self, name: &Token) -> RuntimeResult<Literal> {
        let distance = self.locals.get(name);
        dbg!(&name);
        dbg!(distance);
        dbg!(&self.environments);
        println!("");
        match distance {
            Some(d) => {
                let depth = self.environments.len() - *d;
                self.environments[depth].get(name, &self.environments)
            },
            None => self.globals.get(name, &self.environments)
        }
    }

    fn visit_binary_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> RuntimeResult<Literal> {
        let l = self.visit_expr(left)?;
        let r = self.visit_expr(right)?;

        match operator.typ {
            TokenType::Minus | TokenType::Slash | TokenType::Star => self.calculate_number(&l, operator, &r),
            TokenType::Plus => self.calculate_addition(&l, operator, &r),
            TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual => self.calculate_bool(&l, operator, &r),
            TokenType::BangEqual => Ok(Literal::Bool(!self.is_equal(l, r))),
            TokenType::EqualEqual => Ok(Literal::Bool(self.is_equal(l, r))),
            TokenType::Comma => Ok(r),
            _ => Err(RuntimeError::new(operator.clone(), "Unknown binary operator."))
        }
    }

    fn visit_logical_expr(&mut self, left: &Expr, operator: &Token, right: &Expr) -> RuntimeResult<Literal> {
        let left = self.visit_expr(left)?;
        if operator.typ == TokenType::Or {
            if self.is_truthy(&left) {
                return Ok(left)
            }
        } else {
            if !self.is_truthy(&left) {
                return Ok(left)
            }
        }
        self.visit_expr(right)
    }

    fn visit_ternary_expr(&mut self, left: &Expr, middle: &Expr, right: &Expr) -> RuntimeResult<Literal> {
        let l = self.visit_expr(left)?;
        match self.is_truthy(&l) {
            true => return self.visit_expr(middle),
            false => return self.visit_expr(right)
        }
    }

    fn visit_grouping_expr(&mut self, group: &Expr) -> RuntimeResult<Literal> {
        self.visit_expr(&group)
    }

    fn visit_unary_expr(&mut self, operator: &Token, expr: &Expr) -> RuntimeResult<Literal> {
        let right = self.visit_expr(expr)?;
        match operator.typ {
            TokenType::Minus => {
                if let Literal::Number(num) = right {
                    return Ok(Literal::Number(-num))
                } else {
                    return Err(RuntimeError::new(operator.clone(), "Cannot make non-number negative."))
                }
            },
            TokenType::Bang => {
                return Ok(Literal::Bool(!self.is_truthy(&right)))
            }
            _ => return Err(RuntimeError::new(operator.clone(), "Uknown unary operator."))
        }
    }

    fn visit_call_expr(&mut self, callee: &Expr, right_paren: &Token, arguments: &Vec<Expression>) -> RuntimeResult<Literal> {
        let callee = self.visit_expr(callee)?;
        match callee {
            Literal::Fun(function) =>  {
                if arguments.len() != function.arity {
                    return Err(RuntimeError::new(right_paren.clone(), "Wrong number of arguments."))
                }
                let mut evaluated_args = Vec::new();
                for arg in arguments {
                    evaluated_args.push(self.visit_expr(arg)?);
                };
                function.call(self, &evaluated_args)?;
                let value = self.return_value.clone();
                self.return_value = Literal::Nothing;
                return Ok(value)
            }
            _ => Err(RuntimeError::new(right_paren.clone(), "Can only call functions and classes")),
        }
    }

    fn visit_lambda_expr(&self, params: &Vec<Token>, body: &Declarations) -> RuntimeResult<Literal> {
        let function = Function::new(None, params.clone(), body.clone());
        Ok(Literal::Fun(function))
    }

    fn visit_literal(&self, value: Literal) -> RuntimeResult<Literal> {
        Ok(value)
    }

    fn calculate_addition(&self, left: &Literal, operator: &Token, right: &Literal) -> RuntimeResult<Literal> {
        match left {
            Literal::Number(l) => match right {
                Literal::Number(r) => return Ok(Literal::Number(r + l)),
                Literal::Str(r) => return Ok(Literal::Str(format!("{}{}", l, r))),
                _ => return Err(RuntimeError::new(operator.clone(), "Cannot add operands."))
            },
            Literal::Str(l) => match right {
                Literal::Number(r) => return Ok(Literal::Str(format!("{}{}", l, r))),
                Literal::Str(r) => return Ok(Literal::Str(format!("{}{}", l, r))),
                _ => return Err(RuntimeError::new(operator.clone(), "Cannot add operands."))
            },
            _ => Err(RuntimeError::new(operator.clone(), "Cannot add operands."))
        }
    }

    fn calculate_number(&self, left: &Literal, operator: &Token, right: &Literal) -> RuntimeResult<Literal> {
        if let Literal::Number(l) = left {
            if let Literal::Number(r) = right {
                match operator.typ {
                    TokenType::Minus => return Ok(Literal::Number(l - r)),
                    TokenType::Star => return Ok(Literal::Number(l * r)),
                    TokenType::Slash => {
                        if *r == 0.0 {
                            return Err(RuntimeError::new(operator.clone(), "Cannot divide by zero."))
                        }
                        return Ok(Literal::Number(l / r))
                    },
                    _ => return Err(RuntimeError::new(operator.clone(), "Uknown operator for numbers."))
                }   
            }
        }
        Err(RuntimeError::new(operator.clone(), "Operand must be a number"))
    }

    fn calculate_bool(&self, left: &Literal, operator: &Token, right: &Literal) -> RuntimeResult<Literal> {
        if let Literal::Number(l) = left {
            if let Literal::Number(r) = right {
                match operator.typ {
                    TokenType::Greater => return Ok(Literal::Bool(l > r)),
                    TokenType::GreaterEqual => return Ok(Literal::Bool(l >= r)),
                    TokenType::Less => return Ok(Literal::Bool(l < r)),
                    TokenType::LessEqual => return Ok(Literal::Bool(l <= r)),
                    _ => return Err(RuntimeError::new(operator.clone(), "Uknown operator for numbers."))
                }   
            }
        }
        Err(RuntimeError::new(operator.clone(), "Cannot compare non-booleans."))
    }

    fn is_truthy(&self, value: &Literal) -> bool {
        match value {
            Literal::Nothing => false,
            Literal::Bool(b) => *b,
            _ => true
        }
    }

    fn is_equal(&self, left: Literal, right: Literal) -> bool {
        left == right
    }
}