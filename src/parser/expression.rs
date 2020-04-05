use crate::lexer::token::{Token, Literal};

#[derive(Debug)]
pub enum Expr {
    Binary(Box<Expr>, Token, Box<Expr>),
    Grouping(Box<Expr>),
    Literal(Literal),
    #[allow(dead_code)]
    Logical(Box<Expr>, Token, Box<Expr>),
    Unary(Token, Box<Expr>)
}

/* 
    Wrapper for printing AST in Lisp-like format
    Example use:
    use lexer::token::{Token, TokenType, Literal as TokenLiteral};
    use parser::expression::{Expr, AstPrinter};

    let expr = Expr::Binary(
        Box::new(Expr::Unary(
            Token::new(TokenType::Minus, "-".to_string(), TokenLiteral::Nothing, 1),
            Box::new(Expr::Literal(Literal::Number(123.0)))
        )),
        Token::new(TokenType::Star, "*".to_string(), TokenLiteral::Nothing, 1),
        Box::new(Expr::Grouping(
            Box::new(Expr::Literal(Literal::Number(456.0)))
        ))
    );
    let ast_printer = AstPrinter;
    ast_printer.print(&expr);
*/

pub struct AstPrinter;

impl AstPrinter {
    #[allow(dead_code)]
    pub fn print(&self, expr: &Expr) {
        println!("{}", self.visit(expr));
    }
    
    fn visit(&self, expr: &Expr) -> String {
        match expr {
            Expr::Binary(left, operator, right) => self.visit_binary_expr(&left, &operator, &right),
            Expr::Grouping(group) => self.visit_grouping_expr(&group),
            Expr::Literal(literal) => self.visit_literal_expr(literal),
            Expr::Logical(left, operator, right) => self.visit_logical_expr(&left, &operator, &right),
            Expr::Unary(operator, right) => self.visit_unary_expr(&operator, &right)
        }
    }

    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> String {
        let mut tree = format!("({} ", operator.lexme);
        tree.push_str(&self.visit(&left));
        tree.push_str(" ");
        tree.push_str(&self.visit(&right));
        tree.push_str(")");
        tree
    }

    fn visit_grouping_expr(&self, group: &Expr) -> String {
        format!("({})", self.visit(&group))
    }

    fn visit_literal_expr(&self, literal: &Literal) -> String {
        match literal {
            Literal::Str(string) => format!("\"{}\"", string),
            Literal::Number(num) => format!("{}", num),
            Literal::Bool(boolean) => format!("{}", boolean),
            _ => "null".to_owned()
        }
    }

    fn visit_logical_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> String {
        let mut tree = format!("({} ", operator.lexme);
        tree.push_str(&self.visit(&left));
        tree.push_str(" ");
        tree.push_str(&self.visit(&right));
        tree.push_str(")");
        tree
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> String {
        let mut tree = format!("({} ", operator.lexme);
        tree.push_str(&self.visit(&right));
        tree.push_str(")");
        tree
    }
}

/*
    Wrapper for printing AST in Reverse Polish Notation (RPN)
    Example use:
    use lexer::token::{Token, TokenType, Literal as TokenLiteral};
    use parser::expression::{Expr, RpnPrinter};

    let expr = Expr::Binary(
        Box::new(Expr::Grouping(
            Box::new(Expr::Binary(
                Box::new(Expr::Literal(Literal::Number(1.0))),
                Token::new(TokenType::Plus, "+".to_string(), TokenLiteral::Nothing, 1),
                Box::new(Expr::Literal(Literal::Number(2.0)))
            ),
        ))),
        Token::new(TokenType::Star, "*".to_string(), TokenLiteral::Nothing, 1),
        Box::new(Expr::Grouping(
            Box::new(Expr::Binary(
                Box::new(Expr::Literal(Literal::Number(4.0))),
                Token::new(TokenType::Minus, "-".to_string(), TokenLiteral::Nothing, 1),
                Box::new(Expr::Literal(Literal::Number(3.0)))
            ),
        ),
    )));
    let rpn_printer = RpnPrinter;
    rpn_printer.print(&expr);
*/


pub struct RpnPrinter;

impl RpnPrinter {
    #[allow(dead_code)]
    pub fn print(&self, expr: &Expr) {
        println!("{}", self.visit(expr));
    }

    fn visit(&self, expr: &Expr) -> String {
        match expr {
            Expr::Binary(left, operator, right) => self.visit_binary_expr(&left, &operator, &right),
            Expr::Grouping(group) => self.visit_grouping_expr(&group),
            Expr::Literal(literal) => self.visit_literal_expr(literal),
            Expr::Logical(left, operator, right) => self.visit_logical_expr(&left, &operator, &right),
            Expr::Unary(operator, right) => self.visit_unary_expr(&operator, &right)
        }
    }

    fn visit_binary_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> String {
        let mut tree = String::new();
        tree.push_str(&self.visit(&left));
        tree.push_str(" ");
        tree.push_str(&self.visit(&right));
        tree.push_str(" ");
        tree.push_str(&operator.lexme);
        tree
    }

    fn visit_grouping_expr(&self, group: &Expr) -> String {
        self.visit(&group)
    }

    fn visit_literal_expr(&self, literal: &Literal) -> String {
        match literal {
            Literal::Str(string) => string.to_owned(),
            Literal::Number(num) => format!("{}", num),
            Literal::Bool(boolean) => format!("{}", boolean),
            _ => "null".to_owned()
        }
    }

    fn visit_logical_expr(&self, left: &Expr, operator: &Token, right: &Expr) -> String {
        let mut tree = String::new();
        tree.push_str(&self.visit(&left));
        tree.push_str(" ");
        tree.push_str(&self.visit(&right));
        tree.push_str(" ");
        tree.push_str(&operator.lexme);
        tree
    }

    fn visit_unary_expr(&self, operator: &Token, right: &Expr) -> String {
        let mut tree = String::new();
        tree.push_str(&self.visit(&right));
        tree.push_str(" ");
        tree.push_str(&operator.lexme);
        tree
    }
}