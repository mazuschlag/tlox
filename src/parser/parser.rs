use crate::lexer::token::Token;

/* 
    Wrapper for printing AST in Lisp-like format
    Example use:
    let expr = Binary::new(
        Box::new(
            Unary::new(
                Token::new(TokenType::Minus, "-".to_string(), TokenLiteral::Nothing, 1),
                Box::new(Literal::new(123 as f64))
            )), 
        Token::new(TokenType::Star, "*".to_string(), TokenLiteral::Nothing, 1),
        Box::new(
            Grouping::new(
                Box::new(Literal::new(45.67))
            ))
        );
    let mut ast_printer = AstPrinter;
    ast_printer.print(&expr);
*/

pub struct AstPrinter;

impl AstPrinter {
    #[allow(dead_code)]
    pub fn print<E: Expr>(&mut self, expr: &E) {
        println!("{}", expr.accept(self));
    }
}

/*
    Wrapper for printing AST in Reverse Polish Notation (RPN)
    Example use:
    let expr = Binary::new(
        Box::new(Grouping::new(
            Box::new(Binary::new(
                Box::new(Literal::new(1 as f64)),
                Token::new(TokenType::Plus, "+".to_string(), TokenLiteral::Nothing, 1),
                Box::new(Literal::new(2 as f64))
            ),
        ))),
        Token::new(TokenType::Star, "*".to_string(), TokenLiteral::Nothing, 1),
        Box::new(Grouping::new(
            Box::new(Binary::new(
                Box::new(Literal::new(4 as f64)),
                Token::new(TokenType::Minus, "-".to_string(), TokenLiteral::Nothing, 1),
                Box::new(Literal::new(3 as f64))
            ),
        ),
    )));
    let mut rpn_printer = RpnPrinter;
    rpn_printer.print(&expr);
*/


pub struct RpnPrinter;

impl RpnPrinter {
    #[allow(dead_code)]
    pub fn print<E: Expr>(&mut self, expr: &E) {
        println!("{}", expr.accept(self));
    }
}

#[derive(Debug)]
pub struct Binary<L: Expr, R: Expr> {
    left: Box<L>,
    operator: Token,
    right: Box<R>
}

impl<L: Expr, R: Expr> Binary<L, R> {
    pub fn new(left: Box<L>, operator: Token, right: Box<R>) -> Binary<L, R> {
        Binary {
            left,
            operator,
            right
        }
    }
}

#[derive(Debug)]
pub struct Grouping<E: Expr> {
    expression: Box<E>
}

impl<E: Expr> Grouping<E> {
    pub fn new(expression: Box<E>) -> Grouping<E> {
        Grouping {
            expression
        }
    }
}

#[derive(Debug)]
pub struct Literal {
    value: Option<f64>
}

impl Literal {
    pub fn new(value: f64) -> Literal {
        Literal {
            value: Some(value)
        }
    }
}

#[derive(Debug)]
pub struct Logical<L: Expr, R: Expr> {
    left: Box<L>,
    operator: Token,
    right: Box<R>
}

impl<L: Expr, R: Expr> Logical<L, R> {
    pub fn new(left: Box<L>, operator: Token, right: Box<R>) -> Logical<L, R> {
        Logical {
            left,
            operator,
            right
        }
    }
}

#[derive(Debug)]
pub struct Unary<R: Expr> {
    operator: Token,
    right: Box<R>
}

impl<R: Expr> Unary<R> {
    pub fn new(operator: Token, right: Box<R>) -> Unary<R> {
        Unary {
            operator,
            right
        }
    }
}

pub trait Expr {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result;
}

impl<L: Expr, R: Expr> Expr for Binary<L, R> {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_binary_expr(self)
    }
}

impl<E: Expr> Expr for Grouping<E> {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_grouping_expr(self)
    }
}

impl Expr for Literal {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_literal_expr(self)
    }
}

impl<L: Expr, R: Expr> Expr for Logical<L, R> {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_logical_expr(self)
    }
}

impl<R: Expr> Expr for Unary<R> {
    fn accept<V: Visitor>(&self, visitor: &mut V) -> V::Result {
        visitor.visit_unary_expr(self)
    }
}

pub trait Visitor {
    type Result;
    fn visit_binary_expr<L: Expr, R: Expr>(&mut self, binary_expr: &Binary<L, R>) -> Self::Result;
    fn visit_grouping_expr<E: Expr>(&mut self, grouping_expr: &Grouping<E>) -> Self::Result;
    fn visit_literal_expr(&mut self, literal_expr: &Literal) -> Self::Result;
    fn visit_logical_expr<L: Expr, R: Expr>(&mut self, logical_expr: &Logical<L, R>) -> Self::Result;
    fn visit_unary_expr<R: Expr>(&mut self, unary_expr: &Unary<R>) -> Self::Result;
}

impl Visitor for AstPrinter {
    type Result = String;
    
    fn visit_binary_expr<L: Expr, R: Expr>(&mut self, binary_expr: &Binary<L, R>) -> Self::Result {
        let mut tree = format!("({} ", binary_expr.operator.lexme);
        tree.push_str(&binary_expr.left.accept(self));
        tree.push_str(&binary_expr.right.accept(self));
        tree.push_str(") ");
        tree
    }

    fn visit_grouping_expr<E: Expr>(&mut self, grouping_expr: &Grouping<E>) -> Self::Result {
        format!("({})", grouping_expr.expression.accept(self))
    }

    fn visit_literal_expr(&mut self, literal_expr: &Literal) -> Self::Result {
        match &literal_expr.value {
            Some(v) => v.to_string(),
            None => "nil".to_string()
        }
    }

    fn visit_logical_expr<L: Expr, R: Expr>(&mut self, logical_expr: &Logical<L, R>) -> Self::Result {
        let mut tree = format!("({} ", logical_expr.operator.lexme);
        tree.push_str(&logical_expr.left.accept(self));
        tree.push_str(&logical_expr.right.accept(self));
        tree.push_str(") ");
        tree
    }

    fn visit_unary_expr<R: Expr>(&mut self, unary_expr: &Unary<R>) -> Self::Result {
        let mut tree = format!("({} ", unary_expr.operator.lexme);
        tree.push_str(&unary_expr.right.accept(self));
        tree.push_str(") ");
        tree
    }
}

impl Visitor for RpnPrinter {
    type Result = String;
    
    fn visit_binary_expr<L: Expr, R: Expr>(&mut self, binary_expr: &Binary<L, R>) -> Self::Result {
        let mut tree = String::new();
        tree.push_str(&binary_expr.left.accept(self));
        tree.push_str(" ");
        tree.push_str(&binary_expr.right.accept(self));
        tree.push_str(" ");
        tree.push_str(&binary_expr.operator.lexme);
        tree
    }

    fn visit_grouping_expr<E: Expr>(&mut self, grouping_expr: &Grouping<E>) -> Self::Result {
        format!("{}", grouping_expr.expression.accept(self))
    }

    fn visit_literal_expr(&mut self, literal_expr: &Literal) -> Self::Result {
        match &literal_expr.value {
            Some(v) => v.to_string(),
            None => "nil".to_string()
        }
    }

    fn visit_logical_expr<L: Expr, R: Expr>(&mut self, logical_expr: &Logical<L, R>) -> Self::Result {
        let mut tree = String::new();
        tree.push_str(&logical_expr.left.accept(self));
        tree.push_str(" ");
        tree.push_str(&logical_expr.right.accept(self));
        tree.push_str(" ");
        tree.push_str(&logical_expr.operator.lexme);
        tree
    }

    fn visit_unary_expr<R: Expr>(&mut self, unary_expr: &Unary<R>) -> Self::Result {
        let mut tree = String::new();
        tree.push_str(&unary_expr.right.accept(self));
        tree.push_str(&unary_expr.operator.lexme);
        tree
    }
}