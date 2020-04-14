use crate::parser::expression::Expression;
use crate::lexer::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Expression(Expression),
    Print(Expression),
    Var(Token, Expression)
}

pub type Program = Vec<Stmt>;