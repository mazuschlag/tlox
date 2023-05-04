use crate::lexer::token::Token;
use crate::arena::pool::PoolRef;
use super::expression::ExprRef;


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
    Block(Vec<StmtRef>),
    If(ExprRef, StmtRef, Option<StmtRef>),
    While(ExprRef, StmtRef),
    Function(Token, Vec<Token>, Vec<StmtRef>),
    Getter(Token, Vec<StmtRef>),
    Return(Token, ExprRef),
    Class(Token, Vec<StmtRef>, Option<ExprRef>),
}
