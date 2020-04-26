use crate::parser::expression::Expr;
use crate::parser::statement::{Stmt, Declarations};
use crate::error::report::{runtime_report, RuntimeError};
use crate::lexer::token::{Token, Literal, TokenType};
use crate::interpreter::environment::Environment;

pub struct Interpreter {
    environment: Environment
}

pub type RuntimeResult<T> = Result<T, RuntimeError>;

impl Interpreter {
    pub fn new() -> Interpreter {
        Interpreter {
            environment: Environment::new(None)
        }
    }

    pub fn interpret(&mut self, program: &Declarations) {
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
            Stmt::Block(statements) => self.visit_block_stmt(statements)
        }?;
        Ok(())
    }

    fn visit_block_stmt(&mut self, statements: &Declarations) -> RuntimeResult<()> {
        let previous = self.environment.clone();
        self.environment = Environment::new(Some(self.environment.clone()));
        for statement in statements {
            if let Some(err) = self.visit_stmt(statement).err() {
                self.environment = previous;
                return Err(err)
            }
        }
        self.environment = previous;
        Ok(())
    }

    fn visit_expression_stmt(&mut self, expr: &Expr) -> RuntimeResult<()> {
        self.visit_expr(expr)?;
        Ok(())
    }

    fn visit_print_stmt(&mut self, expr: &Expr) -> RuntimeResult<()> {
        let value = self.visit_expr(expr)?;
        println!("{}", value);
        Ok(())
    }

    fn visit_var_stmt(&mut self, name: &Token, initializer: &Expr) -> RuntimeResult<()> {
        let value = match *initializer {
            Expr::Literal(Literal::Nothing) => Literal::Nothing,
            _ => self.visit_expr(initializer)?
        };
        self.environment.define(name.lexeme.clone(), value);
        return Ok(())
    }

    fn visit_expr(&mut self, expr: &Expr) -> RuntimeResult<Literal> {
        match expr {
            Expr::Grouping(group) => self.visit_grouping_expr(group),
            Expr::Unary(operator, right) => self.visit_unary_expr(operator, right),
            Expr::Literal(value) => self.visit_literal(value.clone()),
            Expr::Binary(left, operator, right) => self.visit_binary_expr(left, operator, right),
            Expr::Ternary(left, _, middle, _, right) => self.visit_ternary_expr(left, middle, right),
            Expr::Variable(var) => self.visit_var_expr(var),
            Expr::Assign(name, value) => self.visit_assign_expr(name, value),
            _ => unimplemented!()
        }
    }

    fn visit_assign_expr(&mut self, name: &Token, initializer: &Expr) -> RuntimeResult<Literal> {
        let value = self.visit_expr(initializer)?;
        self.environment.assign(name, value.clone())?;
        Ok(value)
    }

    fn visit_var_expr(&self, name: &Token) -> RuntimeResult<Literal> {
        self.environment.get(name)
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
            _ => unimplemented!()
        }
    }

    fn visit_ternary_expr(&mut self, left: &Expr, middle: &Expr, right: &Expr) -> RuntimeResult<Literal> {
        let l = self.visit_expr(left)?;
        match self.is_truthy(l) {
            true => return self.visit_expr(middle),
            false => return self.visit_expr(right)
        }
    }

    fn visit_literal(&self, value: Literal) -> RuntimeResult<Literal> {
        Ok(value)
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
                return Ok(Literal::Bool(!self.is_truthy(right)))
            }
            _ => return Err(RuntimeError::new(operator.clone(), "Uknown unary operator."))
        }
    }

    fn visit_grouping_expr(&mut self, group: &Expr) -> RuntimeResult<Literal> {
        self.visit_expr(&group)
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

    fn is_truthy(&self, value: Literal) -> bool {
        match value {
            Literal::Nothing => false,
            Literal::Bool(b) => b,
            _ => true
        }
    }

    fn is_equal(&self, left: Literal, right: Literal) -> bool {
        left == right
    }
}