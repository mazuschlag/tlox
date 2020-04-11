use crate::parser::expression::Expr;

#[derive(Debug)]
pub enum Stmt {
    Expression(Box<Expr>),
    Print(Box<Expr>)
}

pub type Program = Vec<Stmt>;