use crate::lexer::token::Token;
use crate::parser::expression::Expression;

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expression(Expression),
    Print(Expression),
    Var(Token, Expression),
    Block(Declarations),
    If(Expression, Box<Stmt>, Box<Option<Stmt>>),
    While(Expression, Box<Stmt>),
    Function(Token, Vec<Token>, Declarations),
    Return(Token, Expression),
    Class(Token, Vec<Stmt>),
}

pub type Declarations = Vec<Stmt>;
