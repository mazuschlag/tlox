use crate::arena::pool::{Pool, Pools};
use crate::error::report::error;
use crate::lexer::literal::Literal;
use crate::lexer::token::{Token, TokenType};

use super::expression::{Expr, ExprRef, FunctionType};
use super::statement::{Stmt, StmtRef};

#[derive(Debug)]
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    is_repl: bool,
    errors: Vec<String>,
    pub statements: Vec<StmtRef>,
    pub expr_pool: Pool<Expr>,
    pub stmt_pool: Pool<Stmt>,
}

type ParseResult<T> = Result<T, String>;

pub struct ParseOutput(pub Vec<StmtRef>, pub Pools<Stmt, Expr>);

impl Parser {
    pub fn new(tokens: Vec<Token>, is_repl: bool) -> Parser {
        Parser {
            tokens: tokens,
            current: 0,
            is_repl: is_repl,
            errors: Vec::new(),
            statements: Vec::new(),
            expr_pool: Pool::default(),
            stmt_pool: Pool::default(),
        }
    }

    pub fn run(mut self) -> Result<ParseOutput, Vec<String>> {
        while !self.is_at_end() {
            match self.declaration() {
                Ok(statement) => self.statements.push(statement),
                Err(err) => self.synchronize(err),
            }
        }

        if self.errors.len() > 0 {
            return Err(self.errors);
        }

        Ok(ParseOutput(
            self.statements,
            Pools(self.stmt_pool, self.expr_pool),
        ))
    }

    fn declaration(&mut self) -> ParseResult<StmtRef> {
        if self.matches(&[TokenType::Var]) {
            return self.var_declaration();
        }
        if self.matches(&[TokenType::Class]) {
            return self.class_declaration();
        }
        if self.matches(&[TokenType::Fun]) {
            return self.function(FunctionType::Function);
        }
        self.statement()
    }

    fn var_declaration(&mut self) -> ParseResult<StmtRef> {
        let name = self.consume(TokenType::Identifier, "Expect variable name.")?;
        let initializer = match self.matches(&[TokenType::Equal]) {
            true => self.expression()?,
            false => self.expr_pool.add(Expr::Literal(Literal::Nothing)),
        };

        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        let stmt = self.stmt_pool.add(Stmt::Var(name, initializer));
        Ok(stmt)
    }

    fn class_declaration(&mut self) -> ParseResult<StmtRef> {
        let name = self.consume(TokenType::Identifier, "Expect class name.")?;
        let mut super_class = None;
        if self.matches(&[TokenType::Less]) {
            self.consume(TokenType::Identifier, "Expect superclass name")?;
            super_class = Some(Expr::Variable(self.previous()));
        }
        self.consume(TokenType::LeftBrace, "Expect '{' before class body")?;
        let mut methods = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            methods.push(self.class_function()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after class body")?;
        let expr = super_class.map(|class| self.expr_pool.add(class));
        let stmt = self.stmt_pool.add(Stmt::Class(name, methods, expr));
        Ok(stmt)
    }

    fn class_function(&mut self) -> ParseResult<StmtRef> {
        let kind = if self.matches(&[TokenType::Class]) {
            FunctionType::Static
        } else {
            FunctionType::Method
        };
        self.function(kind)
    }

    fn function(&mut self, mut kind: FunctionType) -> ParseResult<StmtRef> {
        let name = self.consume(TokenType::Identifier, &format!("Expect {} name.", kind))?;
        let mut params = Vec::new();
        match kind {
            FunctionType::Function | FunctionType::Static => {
                self.consume(
                    TokenType::LeftParen,
                    &format!("Expect '(' after {} name.", kind),
                )?;
                params = self.function_arguments(params)?;
                self.consume(TokenType::RightParen, "Expect ')' after parameters")?;
            }
            FunctionType::Method if self.matches(&[TokenType::LeftParen]) => {
                params = self.function_arguments(params)?;
                self.consume(TokenType::RightParen, "Expect ')' after parameters")?;
            }
            _ => kind = FunctionType::Getter,
        };
        self.consume(
            TokenType::LeftBrace,
            &format!("Expect '{{' before {} body.", kind),
        )?;
        let body = self.block()?;
        let stmt = self.stmt_pool.add(match kind {
            FunctionType::Getter => Stmt::Getter(name, body),
            _ => Stmt::Function(name, params, body),
        });
        Ok(stmt)
    }

    fn function_arguments(&mut self, mut params: Vec<Token>) -> ParseResult<Vec<Token>> {
        if !self.check(TokenType::RightParen) {
            params.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);
            while self.matches(&[TokenType::Comma]) {
                if params.len() > 254 {
                    return Err(self.parse_error("Cannot have more than 255 arguments."));
                }
                params.push(self.consume(TokenType::Identifier, "Expect parameter name.")?);
            }
        }
        Ok(params)
    }

    fn statement(&mut self) -> ParseResult<StmtRef> {
        if self.matches(&[TokenType::Print]) {
            return self.print_statement();
        }
        if self.matches(&[TokenType::LeftBrace]) {
            let block = self.block()?;
            let stmt = self.stmt_pool.add(Stmt::Block(block));
            return Ok(stmt);
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

    fn block(&mut self) -> ParseResult<Vec<StmtRef>> {
        let mut statements = Vec::new();
        while !self.check(TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(TokenType::RightBrace, "Expect '}' after block.")?;
        Ok(statements)
    }

    fn print_statement(&mut self) -> ParseResult<StmtRef> {
        let value = self.expression()?;
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        let stmt = self.stmt_pool.add(Stmt::Print(value));
        Ok(stmt)
    }

    fn if_statement(&mut self) -> ParseResult<StmtRef> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'if'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after if condition.")?;
        let then_branch = self.statement()?;
        let else_branch = if self.matches(&[TokenType::Else]) {
            Some(self.statement()?)
        } else {
            None
        };

        let stmt = self
            .stmt_pool
            .add(Stmt::If(condition, then_branch, else_branch));
        Ok(stmt)
    }

    fn while_statement(&mut self) -> ParseResult<StmtRef> {
        self.consume(TokenType::LeftParen, "Expect '(' after 'while'.")?;
        let condition = self.expression()?;
        self.consume(TokenType::RightParen, "Expect ')' after while condition.")?;
        let body = self.statement()?;
        let stmt = self.stmt_pool.add(Stmt::While(condition, body));
        Ok(stmt)
    }

    fn for_statement(&mut self) -> ParseResult<StmtRef> {
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
                let stmt = self.stmt_pool.add(Stmt::Expression(expr));
                block.push(stmt)
            }
            self.stmt_pool.add(Stmt::Block(block))
        } else {
            self.statement()?
        };

        body = match condition {
            Some(expr) => self.stmt_pool.add(Stmt::While(expr, body)),
            None => {
                let expr = self.expr_pool.add(Expr::Literal(Literal::Bool(true)));
                self.stmt_pool.add(Stmt::While(expr, body))
            }
        };

        if let Some(stmt) = initializer {
            body = self.stmt_pool.add(Stmt::Block(vec![stmt, body]));
        }
        Ok(body)
    }

    fn return_statement(&mut self) -> ParseResult<StmtRef> {
        let keyword = self.previous();
        let value = if !self.check(TokenType::SemiColon) {
            self.expression()?
        } else {
            self.expr_pool.add(Expr::Literal(Literal::Nothing))
        };
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        let stmt = self.stmt_pool.add(Stmt::Return(keyword, value));
        Ok(stmt)
    }

    fn expression_statement(&mut self) -> ParseResult<StmtRef> {
        let value = self.expression()?;
        if self.is_repl {
            let stmt = self.stmt_pool.add(Stmt::Print(value));
            return Ok(stmt);
        }
        self.consume(TokenType::SemiColon, "Expect ';' after value.")?;
        let stmt = self.stmt_pool.add(Stmt::Expression(value));
        Ok(stmt)
    }

    fn expression(&mut self) -> ParseResult<ExprRef> {
        self.comma()
    }

    fn comma(&mut self) -> ParseResult<ExprRef> {
        let mut expr = self.assignment()?;
        while self.matches(&[TokenType::Comma]) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = self.expr_pool.add(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn assignment(&mut self) -> ParseResult<ExprRef> {
        let expr = self.or()?;
        if self.matches(&[TokenType::Equal]) {
            let value = self.assignment()?;
            match self.expr_pool.get(expr) {
                Expr::Variable(name) => {
                    let expr = self.expr_pool.add(Expr::Assign(name.clone(), value));
                    return Ok(expr);
                }
                Expr::Get(object, name) => {
                    let expr = self
                        .expr_pool
                        .add(Expr::Set(object.clone(), name.clone(), value));
                    return Ok(expr);
                }
                _ => return Err(self.parse_error("Invalid assignment target.")),
            }
        }
        Ok(expr)
    }

    fn or(&mut self) -> ParseResult<ExprRef> {
        let mut expr = self.and()?;
        while self.matches(&[TokenType::Or]) {
            let operator = self.previous();
            let right = self.and()?;
            expr = self.expr_pool.add(Expr::Logical(expr, operator, right));
        }
        Ok(expr)
    }

    fn and(&mut self) -> ParseResult<ExprRef> {
        let mut expr = self.ternary()?;
        while self.matches(&[TokenType::And]) {
            let operator = self.previous();
            let right = self.ternary()?;
            expr = self.expr_pool.add(Expr::Logical(expr, operator, right));
        }
        Ok(expr)
    }

    fn ternary(&mut self) -> ParseResult<ExprRef> {
        let expr = self.equality()?;
        if self.matches(&[TokenType::QuestionMark]) {
            let middle = self.expression()?;
            self.consume(TokenType::Colon, "Expect ':' in ternary expression.")?;
            let right = self.expression()?;
            let expr = self.expr_pool.add(Expr::Ternary(expr, middle, right));
            return Ok(expr);
        }
        Ok(expr)
    }

    fn equality(&mut self) -> ParseResult<ExprRef> {
        let mut expr = self.comparison()?;
        while self.matches(&[TokenType::EqualEqual, TokenType::BangEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = self.expr_pool.add(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn comparison(&mut self) -> ParseResult<ExprRef> {
        let mut expr = self.addition()?;
        while self.matches(&[
            TokenType::LessEqual,
            TokenType::Less,
            TokenType::GreaterEqual,
            TokenType::Greater,
        ]) {
            let operator = self.previous();
            let right = self.addition()?;
            expr = self.expr_pool.add(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn addition(&mut self) -> ParseResult<ExprRef> {
        let mut expr = self.multiplication()?;
        while self.matches(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous();
            let right = self.multiplication()?;
            expr = self.expr_pool.add(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn multiplication(&mut self) -> ParseResult<ExprRef> {
        let mut expr = self.unary()?;
        while self.matches(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = self.expr_pool.add(Expr::Binary(expr, operator, right));
        }
        Ok(expr)
    }

    fn unary(&mut self) -> ParseResult<ExprRef> {
        if self.matches(&[TokenType::Minus, TokenType::Bang]) {
            let operator = self.previous();
            let right = self.unary()?;
            let expr = self.expr_pool.add(Expr::Unary(operator, right));
            return Ok(expr);
        }
        self.call()
    }

    fn call(&mut self) -> ParseResult<ExprRef> {
        let mut expr = self.primary()?;
        loop {
            if self.matches(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.matches(&[TokenType::Dot]) {
                let name =
                    self.consume(TokenType::Identifier, "Expect property name after '.''.")?;
                expr = self.expr_pool.add(Expr::Get(expr, name));
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, expr: ExprRef) -> ParseResult<ExprRef> {
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
        let expr = self.expr_pool.add(Expr::Call(expr, paren, arguments));
        Ok(expr)
    }

    fn call_argument(&mut self) -> ParseResult<ExprRef> {
        if self.matches(&[TokenType::Fun]) {
            return self.lambda();
        }
        self.expression()
    }

    fn lambda(&mut self) -> ParseResult<ExprRef> {
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
        let expr = self.expr_pool.add(Expr::Lambda(params, body));
        Ok(expr)
    }

    fn primary(&mut self) -> ParseResult<ExprRef> {
        if self.matches(&[TokenType::False]) {
            let expr = self.expr_pool.add(Expr::Literal(Literal::Bool(false)));
            return Ok(expr);
        }

        if self.matches(&[TokenType::True]) {
            let expr = self.expr_pool.add(Expr::Literal(Literal::Bool(true)));
            return Ok(expr);
        }

        if self.matches(&[TokenType::Nil]) {
            let expr = self.expr_pool.add(Expr::Literal(Literal::Nothing));
            return Ok(expr);
        }

        if self.matches(&[TokenType::Number]) {
            let num = self.previous().lexeme.parse().unwrap();
            let expr = self.expr_pool.add(Expr::Literal(Literal::Number(num)));
            return Ok(expr);
        }

        if self.matches(&[TokenType::Str]) {
            let string = self.previous().lexeme.clone();
            let expr = self.expr_pool.add(Expr::Literal(Literal::Str(string)));
            return Ok(expr);
        }

        if self.matches(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            match self.consume(TokenType::RightParen, "Expect ')' after expression.") {
                Ok(_) => return Ok(self.expr_pool.add(Expr::Grouping(expr))),
                Err(message) => return Err(message),
            }
        }

        if self.matches(&[TokenType::This]) {
            let expr = self.expr_pool.add(Expr::This(self.previous()));
            return Ok(expr);
        }

        if self.matches(&[TokenType::Identifier]) {
            let expr = self.expr_pool.add(Expr::Variable(self.previous()));
            return Ok(expr);
        }

        if self.matches(&[TokenType::Super]) {
            let keyword = self.previous();
            self.consume(TokenType::Dot, "Expect '.' after 'super'.")?;
            let method = self.consume(TokenType::Identifier, "Expect superclass method name.")?;
            let expr = self.expr_pool.add(Expr::Super(keyword, method));
            return Ok(expr);
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
