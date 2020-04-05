use crate::lexer::token::{Token, TokenType, Literal};
use crate::error::report::error;
use super::expression::Expr;

pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,
            current: 0
        }
    }

    pub fn parse(&mut self) -> Box<Expr> {
        self.expression()
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
            match self.consume(TokenType::RightParen, "Expect ')' after expression.") {
                Ok(_) => return Box::new(Expr::Grouping(expr)),
                Err(message) => panic!(message)
            }
        }
        panic!(self.parse_error("Expect expression."))
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, String> {
        if self.check(&token_type) {
            return Ok(self.advance().clone());
        }
        Err(self.parse_error(message))
    }

    fn _synchronize(&mut self) {
        self.advance();
        while !self.is_at_end() {
            if self.previous().typ == TokenType::SemiColon {
                return
            }
            match self.peek().typ {
                TokenType::Class | TokenType::Fun | TokenType::Var | TokenType::For | 
                TokenType::If | TokenType::While | TokenType::Print | TokenType::Return => return,
                _ => ()
            };
            self.advance();
        }
    }

    fn parse_error(&self, message: &str) -> String {
        let token = self.peek();
        error(token, message)
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