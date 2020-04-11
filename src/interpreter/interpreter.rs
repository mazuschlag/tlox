use crate::parser::expression::Expr;
use crate::error::report::{runtime_report, RuntimeError};
use crate::lexer::token::{Token, Literal, TokenType};

pub struct Interpreter;

type RuntimeResult<T> = Result<T, RuntimeError>;

impl Interpreter {
    pub fn print(&self, expr: &Expr) {
        let result = self.visit(expr)
            .map_err(|err| runtime_report(err))
            .and_then(|result| Ok(match result {
                Literal::Number(n) => format!("{}", n),
                Literal::Bool(b) => format!("{}", b),
                Literal::Str(s) => format!("{}", s),
                _ => unimplemented!()
            }));
        match result {
            Ok(r) => println!("{}", r),
            Err(e) => println!("{}", e)
        };
    }

    fn visit(&self, expr: &Expr) -> RuntimeResult<Literal> {
        match expr {
            Expr::Grouping(group) => self.visit_grouping_expr(group),
            Expr::Unary(operator, right) => self.visit_unary_expr(operator, right),
            Expr::Literal(value) => self.visit_literal(value.clone()),
            Expr::Binary(left, operator, right) => self.visit_binary_expr(left, operator, right),
            _ => unimplemented!()
        }
    }

    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> RuntimeResult<Literal> {
        let l = self.visit(left)?;
        let r = self.visit(right)?;

        match operator.typ {
            TokenType::Minus | TokenType::Slash | TokenType::Star => Ok(Literal::Number(self.calculate_number(&l, operator, &r)?)),
            TokenType::Plus => {
                if let Literal::Number(_) = l {
                    return Ok(Literal::Number(self.calculate_number(&l, operator, &r)?))
                }
                if let Literal::Str(l) = l {
                    if let Literal::Str(r) = r {
                        return Ok(Literal::Str(format!("{}{}", l, r)))
                    }
                }
                Err(RuntimeError::new(operator.clone(), "Operands must be two numbers or two strings."))
            },
            TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual => Ok(Literal::Bool(self.calculate_bool(&l, operator, &r)?)),
            TokenType::BangEqual => Ok(Literal::Bool(!self.is_equal(l, r))),
            TokenType::EqualEqual => Ok(Literal::Bool(self.is_equal(l, r))),
            _ => unimplemented!()
        }
    }

    fn visit_literal(&self, value: Literal) -> RuntimeResult<Literal> {
        Ok(value)
    }

    fn visit_unary_expr(&self, operator: &Token, expr: &Expr) -> RuntimeResult<Literal> {
        let right = self.visit(expr)?;
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
            _ => unreachable!()
        }
    }

    fn visit_grouping_expr(&self, group: &Expr) -> RuntimeResult<Literal> {
        self.visit(&group)
    }

    fn calculate_number(&self, left: &Literal, operator: &Token, right: &Literal) -> RuntimeResult<f64> {
        if let Literal::Number(l) = left {
            if let Literal::Number(r) = right {
                match operator.typ {
                    TokenType::Minus => return Ok(l - r),
                    TokenType::Slash => return Ok(l / r),
                    TokenType::Star => return Ok(l * r),
                    TokenType::Plus => return Ok(l + r),
                    _ => unreachable!()
                }   
            }
        }
        Err(RuntimeError::new(operator.clone(), "Operand must be a number"))
    }

    fn calculate_bool(&self, left: &Literal, operator: &Token, right: &Literal) -> RuntimeResult<bool> {
        if let Literal::Number(l) = left {
            if let Literal::Number(r) = right {
                match operator.typ {
                    TokenType::Greater => return Ok(l > r),
                    TokenType::GreaterEqual => return Ok(l >= r),
                    TokenType::Less => return Ok(l < r),
                    TokenType::LessEqual => return Ok(l <= r),
                    _ => unreachable!()
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