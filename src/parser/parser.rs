use super::expression::{Expr, Expression};
use super::statement::{Declarations, Stmt};
use crate::error::report::error;
use crate::lexer::literal::Literal;
use crate::lexer::token::{Token, TokenType};

#[derive(Debug)]
pub struct Parser<'a> {
    tokens: &'a Vec<Token>,
    current: usize,
    is_repl: bool,
    pub errors: Vec<String>,
    pub statements: Declarations,
}

type ParseResult<T> = Result<T, String>;

impl<'a> Parser<'a> {
    pub fn new(tokens: &Vec<Token>, is_repl: bool) -> Parser {
        Parser {
            tokens: tokens,
            current: 0,
            is_repl: is_repl,
            errors: Vec::new(),
            statements: Vec::new(),
        }
    }

    pub fn parse(&mut self) {
        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => self.statements.push(statement),
                Err(err) => self.synchronize(err),
            }
        }
    }

    fn declaration(&mut self) -> ParseResult<Stmt> {
        if self.matches(&[TokenType::Var]) {
            return self.var_declaration();
        }
        if self.matches(&[TokenType::Class]) {
            return self.class_declaration();
        }
        if self.matches(&[TokenType::Fun]) {
            return self.function("function");
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
        let initializer = match self.matches(&[TokenType::Equal]) {
            true => self.expression()?,
            false => Box::new(Expr::Literal(Literal::Nothing)),
        };
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Var(name, initializer))
    }

    fn class_declaration(&mut self) -> ParseResult<Stmt> {
        let name = self.consume(TokenType::Identifier, "Expect class name.")?;
        self.consume(TokenType::LeftBrace, "Expect '{' before class body")?;
        let mut methods = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.function("method")?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body")?;
        Ok(Stmt::Class(name, methods))
    }

    fn function(&mut self, kind: &str) -> ParseResult<Stmt> {
        let name = if self.matches(&[TokenType::Class]) {
            self.consume(TokenType::Identifier, &format!("Expect {} name.", "class"))?
        } else {
            self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?
        };

        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after {} name.", kind),
        )?;
        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            params.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);
            while self.matches(&[TokenType::Comma]) {
                if params.len() > 254 {
                    return Err(self.parse_error("Cannot have more than 255 arguments."));
                }
                params.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters")?;
        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;
        let body = self.block()?;
        Ok(Stmt::Function(name, params, body))
    }

    fn statement(&mut self) -> ParseResult<Stmt> {
        if self.matches(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.matches(&[TokenType::LeftBrace]) {
            return Ok(Stmt::Block(self.block()?));
        }
        if self.matches(&[TokenType::If]) {
            return self.if_statement();
        }
        if self.matches(&[TokenType::While]) {
            return self.while_statement();
        }
        if self.matches(&[TokenType::For]) {
            return self.for_statement();
        }
        if self.matches(&[TokenType::Return]) {
            return self.return_statement();
        }
        self.expression_statement()
    }

    fn block(&mut self) -> ParseResult<Declarations> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> ParseResult<Stmt> {
        let value = self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Print(value))
    }

    fn if_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;
        let then_branch = self.statement()?;
        let else_branch = if self.matches(&[TokenType::Else]) {
            Some(self.statement()?)
        } else {
            None
        };
        Ok(Stmt::If(
            condition,
            Box::new(then_branch),
            Box::new(else_branch),
        ))
    }

    fn while_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after while condition.")?;
        let body = self.statement()?;
        Ok(Stmt::While(condition, Box::new(body)))
    }

    fn for_statement(&mut self) -> ParseResult<Stmt> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'for'.")?;
        let initializer = if self.matches(&[TokenType::SemiColon]) {
            None
        } else if self.matches(&[TokenType::Var]) {
            match self.var_declaration() {
                Ok(stmt) => Some(stmt),
                Err(err) => return Err(err),
            }
        } else {
            match self.expression_statement() {
                Ok(stmt) => Some(stmt),
                Err(err) => return Err(err),
            }
        };
        let condition = if !self.check(TokenType::SemiColon) {
            match self.expression() {
                Ok(expr) => Some(expr),
                Err(err) => return Err(err),
            }
        } else {
            None
        };
        self.consume(TokenType::SemiColon, "Expect ';' after loop condition.")?;
        let increment = if !self.check(TokenType::RightParen) {
            match self.expression() {
                Ok(expr) => Some(expr),
                Err(err) => return Err(err),
            }
        } else {
            None
        };
        self.consume(TokenType::RightParen, "Expect ')' after for clauses.")?;
        let mut body = if self.matches(&[TokenType::LeftBrace]) {
            let mut block = self.block()?;
            if let Some(expr) = increment {
                block.push(Stmt::Expression(expr))
            }
            Stmt::Block(block)
        } else {
            self.statement()?
        };
        body = match condition {
            Some(expr) => Stmt::While(expr, Box::new(body)),
            None => Stmt::While(Box::new(Expr::Literal(Literal::Bool(true))), Box::new(body)),
        };
        if let Some(stmt) = initializer {
            body = Stmt::Block(vec![stmt, body]);
        }
        Ok(body)
    }

    fn return_statement(&mut self) -> ParseResult<Stmt> {
        let keyword = self.previous();
        let value = if !self.check(TokenType::SemiColon) {
            self.expression()?
        } else {
            Box::new(Expr::Literal(Literal::Nothing))
        };
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Return(keyword, value))
    }

    fn expression_statement(&mut self) -> ParseResult<Stmt> {
        let value = self.expression()?;
        if self.is_repl {
            return Ok(Stmt::Print(value));
        }
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        Ok(Stmt::Expression(value))
    }

    fn expression(&mut self) -> ParseResult<Expression> {
        self.comma()
    }

    fn comma(&mut self) -> ParseResult<Expression> {
        let mut expr = self.assignment()?;
        while self.matches(&[TokenType::Comma]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn assignment(&mut self) -> ParseResult<Expression> {
        let expr = self.or()?;
        if self.matches(&[TokenType::Equal]) {
            let value = self.assignment()?;
            match *expr {
                Expr::Variable(name) => return Ok(Box::new(Expr::Assign(name.clone(), value))),
                Expr::Get(object, name) => {
                    return Ok(Box::new(Expr::Set(object.clone(), name.clone(), value)))
                }
                _ => return Err(self.parse_error("Invalid assignment target.")),
            }
        }
        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<Expression> {
        let mut expr = self.and()?;
        while self.matches(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = Box::new(Expr::Logical(expr, operator, right));
        }
        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<Expression> {
        let mut expr = self.ternary()?;
        while self.matches(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.ternary()?;
            expr = Box::new(Expr::Logical(expr, operator, right));
        }
        Ok(expr)
    }

    fn ternary(&mut self) -> ParseResult<Expression> {
        let expr = self.equality()?;
        if self.matches(&[TokenType::QuestionMark]) {
            let middle = self.expression()?;
            self.consume(TokenType::Colon, "Expect ':' in ternary expression.")?;
            let right = self.expression()?;
            return Ok(Box::new(Expr::Ternary(expr, middle, right)));
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<Expression> {
        let mut expr = self.comparison()?;
        while self.matches(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<Expression> {
        let mut expr = self.addition()?;
        while self.matches(&[
            TokenType::LessEqual,
            TokenType::Less,
            TokenType::GreaterEqual,
            TokenType::Greater,
        ]) {
            let operator = self.previous();
            let right = self.addition()?;
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn addition(&mut self) -> ParseResult<Expression> {
        let mut expr = self.multiplication()?;
        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.multiplication()?;
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn multiplication(&mut self) -> ParseResult<Expression> {
        let mut expr = self.unary()?;
        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Box::new(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<Expression> {
        if self.matches(&[TokenType::Minus, TokenType::Bang]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Box::new(Expr::Unary(operator, right)));
        }
        self.call()
    }

    fn call(&mut self) -> ParseResult<Expression> {
        let mut expr = self.primary()?;
        loop {
            if self.matches(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.matches(&[TokenType::Dot]) {
                let name =
                    self.consume(TokenType::Identifier, "Expect property name after '.''.")?;
                expr = Box::new(Expr::Get(expr, name));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, expr: Box<Expr>) -> ParseResult<Expression> {
        let mut arguments = Vec::new();
        if !self.check(TokenType::RightParen) {
            arguments.push(self.call_argument()?);
            while self.matches(&[TokenType::Comma]) {
                if arguments.len() >= 255 {
                    return Err(self.parse_error("Cannot have more than 255 arguments."));
                }
                arguments.push(self.call_argument()?);
            }
        }
        let paren = self.consume(TokenType::RightParen, "Expect ')' after arguments")?;
        Ok(Box::new(Expr::Call(expr, paren, arguments)))
    }

    fn call_argument(&mut self) -> ParseResult<Expression> {
        if self.matches(&[TokenType::Fun]) {
            return self.lambda();
        }
        self.expression()
    }

    fn lambda(&mut self) -> ParseResult<Expression> {
        self.consume(
            TokenType::LeftParen,
            &format!("Expect '(' after lambda declaration."),
        )?;
        let mut params = Vec::new();
        if !self.check(TokenType::RightParen) {
            params.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);
            while self.matches(&[TokenType::Comma]) {
                if params.len() > 254 {
                    return Err(self.parse_error("Cannot have more than 255 arguments."));
                }
                params.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);
            }
        }
        self.consume(TokenType::RightParen, "Expect ')' after parameters")?;
        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before lambda body."),
        )?;
        let body = self.block()?;
        Ok(Box::new(Expr::Lambda(params, body)))
    }

    fn primary(&mut self) -> ParseResult<Expression> {
        if self.matches(&[TokenType::False]) {
            return Ok(Box::new(Expr::Literal(Literal::Bool(false))));
        }

        if self.matches(&[TokenType::True]) {
            return Ok(Box::new(Expr::Literal(Literal::Bool(true))));
        }

        if self.matches(&[TokenType::Nil]) {
            return Ok(Box::new(Expr::Literal(Literal::Nothing)));
        }

        if self.matches(&[TokenType::Number]) {
            let num = self.previous().lexeme.parse().unwrap();
            return Ok(Box::new(Expr::Literal(Literal::Number(num))));
        }

        if self.matches(&[TokenType::Str]) {
            return Ok(Box::new(Expr::Literal(Literal::Str(
                self.previous().lexeme.clone(),
            ))));
        }

        if self.matches(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            match self.consume(TokenType::RightParen, "Expect ')' after expression.") {
                Ok(_) => return Ok(Box::new(Expr::Grouping(expr))),
                Err(message) => return Err(message),
            }
        }

        if self.matches(&[TokenType::This]) {
            return Ok(Box::new(Expr::This(self.previous())));
        }

        if self.matches(&[TokenType::Identifier]) {
            return Ok(Box::new(Expr::Variable(self.previous())));
        }

        Err(self.parse_error("Expect expression."))
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> ParseResult<Token> {
        if self.check(token_type) {
            return Ok(self.advance().clone());
        }
        Err(self.parse_error(message))
    }

    fn synchronize(&mut self, err: String) {
        self.errors.push(err);
        self.advance();
        while !self.is_at_end() {
            if self.previous().typ == TokenType::SemiColon {
                return;
            }
            match self.peek().typ {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Var
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Print
                | TokenType::Return => return,
                _ => (),
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
            if self.check(*token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: TokenType) -> bool {
        if self.is_at_end() {
            return false;
        }
        self.peek().typ == token_type
    }

    fn advance(&mut self) -> Token {
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

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }
}
