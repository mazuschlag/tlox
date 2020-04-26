use crate::parser::expression::Expression;
use crate::lexer::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Expression(Expression),
    Print(Expression),
    Var(Token, Expression),
    Block(Declarations),
    If(Expression, Box<Stmt>, Box<Option<Stmt>>)
}

pub type Declarations = Vec<Stmt>;