use crate::parser::expression::Expr;
use crate::lexer::token::{Token, Literal, TokenType};

struct Interpreter;

impl Interpreter {
    fn visit(&self, expr: &Expr) -> Literal {
        match expr {
            Expr::Grouping(group) => self.visit_grouping_expr(group),
            Expr::Unary(operator, right) => self.visit_unary_expr(operator, right),
            Expr::Literal(value) => self.visit_literal(value.clone()),
            _ => unimplemented!()
        }
    }

    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> Literal {
        let l = self.visit(left);
        let r = self.visit(right);

        match operator.typ {
            TokenType::Minus | TokenType::Slash | TokenType::Star => Literal::Number(self.calculate_number(&l, operator.typ, &r)),
            TokenType::Plus => {
                if let Literal::Number(_) = l {
                    return Literal::Number(self.calculate_number(&l, TokenType::Slash, &r))
                }
                if let Literal::Str(l) = l {
                    if let Literal::Str(r) = r {
                        return Literal::Str(format!("{}{}", l, r))
                    }
                }
                panic!("Cannot add non-numbers and non-strings.")
            },
            TokenType::Greater | TokenType::GreaterEqual | TokenType::Less | TokenType::LessEqual => Literal::Bool(self.calculate_bool(&l, operator.typ, &r)),
            TokenType::BangEqual => Literal::Bool(!self.is_equal(l, r)),
            TokenType::EqualEqual => Literal::Bool(self.is_equal(l, r)),
            _ => unimplemented!()
        }
    }

    fn visit_literal(&self, value: Literal) -> Literal {
        value
    }

    fn visit_unary_expr(&self, operator: &Token, expr: &Expr) -> Literal {
        let right = self.visit(expr);
        match operator.typ {
            TokenType::Minus => {
                if let Literal::Number(num) = right {
                    return Literal::Number(-num)
                } else {
                    panic!("Cannot make non-number negative.");
                }
            },
            TokenType::Bang => {
                return Literal::Bool(!self.is_truthy(right))
            }
            _ => unreachable!()
        }
    }

    fn visit_grouping_expr(&self, group: &Expr) -> Literal {
        self.visit(&group)
    }

    fn calculate_number(&self, left: &Literal, operator: TokenType, right: &Literal) -> f64 {
        if let Literal::Number(l) = left {
            if let Literal::Number(r) = right {
                match operator {
                    TokenType::Minus => return l - r,
                    TokenType::Slash => return l / r,
                    TokenType::Star => return l * r,
                    TokenType::Plus => return l + r,
                    _ => unreachable!()
                }   
            }
        }
        panic!("Cannot subtract non-numbers.")
    }

    fn calculate_bool(&self, left: &Literal, operator: TokenType, right: &Literal) -> bool {
        if let Literal::Number(l) = left {
            if let Literal::Number(r) = right {
                match operator {
                    TokenType::Greater => return l > r,
                    TokenType::GreaterEqual => return l >= r,
                    TokenType::Less => return l < r,
                    TokenType::LessEqual => return l <= r,
                    _ => unreachable!()
                }   
            }
        }
        panic!("Cannot subtract non-numbers.")
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