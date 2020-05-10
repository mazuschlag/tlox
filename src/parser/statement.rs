use crate::parser::expression::Expression;
use crate::lexer::token::Token;

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expression(Expression),
    Print(Expression),
    Var(Token, Expression),
    Block(Declarations),
    If(Expression, Box<Stmt>, Box<Option<Stmt>>),
    While(Expression, Box<Stmt>),
    Function(Token, Vec<Token>, Declarations)
}

pub type Declarations = Vec<Stmt>;