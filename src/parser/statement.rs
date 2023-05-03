use crate::lexer::token::Token;
use super::expression::ExprRef;
use super::pool::PoolRef;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StmtRef(u32);

impl PoolRef for StmtRef {
    fn new(val: u32) -> Self {
        StmtRef(val)
    }

    fn val(&self) -> u32 {
        self.0
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Expression(ExprRef),
    Print(ExprRef),
    Var(Token, ExprRef),
    Block(Declarations),
    If(ExprRef, StmtRef, Option<StmtRef>),
    While(ExprRef, StmtRef),
    Function(Token, Vec<Token>, Declarations),
    Getter(Token, Declarations),
    Return(Token, ExprRef),
    Class(Token, Vec<Stmt>, Option<ExprRef>),
}

pub type Declarations = Vec<Stmt>;
