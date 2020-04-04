use crate::lexer::token::{Token, TokenType, Literal};
use super::expression::Expr;

struct Parser {
    tokens: Vec<Token>,
    current: usize
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,
            current: 0
        }
    }

    fn expression(&mut self) -> Box<Expr> {
        self.equality()
    }

    fn equality(&mut self) -> Box<Expr> {
        let mut expr = self.comparison();
        while self.matches(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison();
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        expr
    }

    fn comparison(&mut self) -> Box<Expr> {
        let mut expr = self.addition();
        while self.matches(&[TokenType::LessEqual, TokenType::Less, TokenType::GreaterEqual, TokenType::Greater]) {
            let operator = self.previous().clone();
            let right = self.addition();
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        expr
    }

    fn addition(&mut self) -> Box<Expr> {
        let mut expr = self.multiplication();
        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.multiplication();
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        expr
    }

    fn multiplication(&mut self) -> Box<Expr> {
        let mut expr = self.unary();
        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary();
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        expr
    }

    fn unary(&mut self) -> Box<Expr> {
        if self.matches(&[TokenType::Minus, TokenType::Bang]) {
            let operator = self.previous().clone();
            let right = self.unary();
            return Box::new(Expr::Unary(operator, right))
        }
        self.primary()
    }

    fn primary(&mut self) -> Box<Expr> {
        if self.matches(&[TokenType::False]) {
            return Box::new(Expr::Literal(Literal::Bool(false)))
        }

        if self.matches(&[TokenType::True]) {
            return Box::new(Expr::Literal(Literal::Bool(true)))
        }

        if self.matches(&[TokenType::Nil]) {
            return Box::new(Expr::Literal(Literal::Nothing))
        }

        if self.matches(&[TokenType::Number, TokenType::Str]) {
            return Box::new(Expr::Literal(self.previous().literal.clone()))
        }

        if self.matches(&[TokenType::LeftParen]) {
            let expr = self.expression();
            self.consume(TokenType::RightParen, "Expect ')' after expression.");
            return Box::new(Expr::Grouping(expr))
        }

        return Box::new(Expr::Literal(Literal::Nothing))
    }

    fn consume(&mut self, token_type: TokenType, error: &str) {

    }

    fn matches(&mut self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true
            }
        }
        false
    }

    fn check(&mut self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            return false
        }
        self.peek().typ == *token_type
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().typ == TokenType::Eof
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}