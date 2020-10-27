use crate::lexer::literal::Literal;
use crate::lexer::token::Token;
use crate::parser::statement::Declarations;
use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Binary(Expression, Token, Expression),
    Ternary(Expression, Expression, Expression),
    Grouping(Expression),
    Literal(Literal),
    Logical(Expression, Token, Expression),
    Unary(Token, Expression),
    Variable(Token),
    Assign(Token, Expression),
    Call(Expression, Token, Vec<Box<Expr>>),
    Lambda(Vec<Token>, Declarations),
    Get(Expression, Token),
    Set(Expression, Token, Expression),
    This(Token),
}

pub type Expression = Box<Expr>;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum FunctionType {
    Function,
    Method,
    Static,
    Getter,
}

impl fmt::Display for FunctionType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            FunctionType::Function => write!(f, "function"),
            FunctionType::Method => write!(f, "method"),
            FunctionType::Static => write!(f, "static function"),
            FunctionType::Getter => write!(f, "getter function"),
        }
    }
}
