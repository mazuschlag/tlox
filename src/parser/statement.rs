use crate::parser::expression::Expression;
use crate::lexer::token::Token;

#[derive(Debug)]
pub enum Stmt {
    Expression(Expression),
    Print(Expression),
    Var(Token, Expression),
    Block(Declarations)
}

pub type Declarations = Vec<Stmt>;