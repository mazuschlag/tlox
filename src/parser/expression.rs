use crate::arena::pool::PoolRef;
use crate::lexer::literal::Literal;
use crate::lexer::token::Token;

use std::fmt;

use super::statement::StmtRef;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExprRef(u32);

impl PoolRef for ExprRef {
    fn new(val: u32) -> Self {
        ExprRef(val)
    }

    fn val(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Binary(ExprRef, Token, ExprRef),
    Ternary(ExprRef, ExprRef, ExprRef),
    Grouping(ExprRef),
    Literal(Literal),
    Logical(ExprRef, Token, ExprRef),
    Unary(Token, ExprRef),
    Variable(Token),
    Assign(Token, ExprRef),
    Call(ExprRef, Token, Vec<ExprRef>),
    Lambda(Vec<Token>, Vec<StmtRef>),
    Get(ExprRef, Token),
    Set(ExprRef, Token, ExprRef),
    This(Token),
    Super(Token, Token),
}

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
