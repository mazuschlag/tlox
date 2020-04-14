use crate::lexer::token::{Token, TokenType, Literal};
use crate::error::report::error;
use super::expression::{Expr, Expression};
use super::statement::{Program, Stmt};

#[derive(Debug)]
pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize
}

type ParseResult<T> = Result<T, String>;

impl<'a> Parser<'a> {
    pub fn new(tokens: &Vec<Token>) -> Parser {
        Parser {
            tokens: tokens,
            current: 0
        }
    }

    pub fn parse(&mut self) -> ParseResult<Program> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => statements.push(statement),
                Err(_) => self.synchronize()
            }
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> ParseResult<Stmt> {
        if self.matches(&[TokenType::Var]) {
            return self.var_declaration()
        }
        let result = self.statement();
        result
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
        let initializer = match self.matches(&[TokenType::Equal]) {
            true => self.expression()?,
            false => Box::new(Expr::Literal(Literal::Nothing))
        };
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Var(name, initializer))
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.matches(&[TokenType::Print]) {
            return self.print_statement()
        }
        self.expression_statement()
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(value))
    }

    fn expression(&mut self) -> ParseResult<Expression> {
        self.comma()
    }

    fn comma(&mut self) -> ParseResult<Expression> {
        let mut expr = self.ternary()?;
        while self.matches(&[TokenType::Comma]) {
            let operator = self.previous().clone();
            let right = self.equality()?;
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn ternary(&mut self) -> ParseResult<Expression> {
        let expr = self.equality()?;
        if self.matches(&[TokenType::QuestionMark]) {
            let question_mark = self.previous().clone();
            let middle = self.expression()?;
            match self.consume(TokenType::Colon, "Expect ':' in ternary expression.") {
                Ok(colon) => {
                    let right = self.expression()?;
                    return Ok(Box::new(Expr::Ternary(expr, question_mark, middle, colon, right)))
                },
                Err(message) => return Err(message)
            }
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Expression> {
        let mut expr = self.comparison()?;
        while self.matches(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expression> {
        let mut expr = self.addition()?;
        while self.matches(&[TokenType::LessEqual, TokenType::Less, TokenType::GreaterEqual, TokenType::Greater]) {
            let operator = self.previous().clone();
            let right = self.addition()?;
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn addition(&mut self) -> ParseResult<Expression> {
        let mut expr = self.multiplication()?;
        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = self.multiplication()?;
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn multiplication(&mut self) -> ParseResult<Expression> {
        let mut expr = self.unary()?;
        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expression> {
        if self.matches(&[TokenType::Minus, TokenType::Bang]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            return Ok(Box::new(Expr::Unary(operator, right)))
        }
        self.primary()
    }

    fn primary(&mut self) -> ParseResult<Expression> {
        if self.matches(&[TokenType::False]) {
            return Ok(Box::new(Expr::Literal(Literal::Bool(false))))
        }

        if self.matches(&[TokenType::True]) {
            return Ok(Box::new(Expr::Literal(Literal::Bool(true))))
        }

        if self.matches(&[TokenType::Nil]) {
            return Ok(Box::new(Expr::Literal(Literal::Nothing)))
        }

        if self.matches(&[TokenType::Number, TokenType::Str]) {
            return Ok(Box::new(Expr::Literal(self.previous().literal.clone())))
        }

        if self.matches(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            match self.consume(TokenType::RightParen, "Expect ')' after expression.") {
                Ok(_) => return Ok(Box::new(Expr::Grouping(expr))),
                Err(message) => return Err(message)
            }
        }

        if self.matches(&[TokenType::Identifier]) {
            return Ok(Box::new(Expr::Variable(self.previous().clone())))
        }

        Err(self.parse_error("Expect expression."))
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> ParseResult<Token> {
        if self.check(&token_type) {
            return Ok(self.advance().clone());
        }
        Err(self.parse_error(message))
    }

    fn synchronize(&mut self) {
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