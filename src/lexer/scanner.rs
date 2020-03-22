use super::token::{Token, TokenType, Literal};

pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    current_token: String,
    line: u32,
    errors: Vec<String>,
    literal: Literal 
}

impl<'a> Scanner<'a> {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source: source,
            tokens: Vec::new(),
            current_token: String::new(),
            line: 1,
            errors: Vec::new(),
            literal: Literal::Nothing,
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        for c in self.source.chars() {
            self.scan_token(c);
        }
        self.add_saved_token('\0');
        self.tokens.push(Token::new(TokenType::Eof, String::new(), Literal::Nothing, self.line));
        &self.tokens
    }

    fn scan_token(&mut self, c: char) {
        match &self.literal {
            Literal::Comment if c != '\n' => return,
            Literal::Str(_) => {
                self.add_to_string(c);
                return
            },
            Literal::Number(_) if valid_digit(c) => { 
                self.add_to_number(c);
                return
            },
            Literal::Identifier if valid_identifier(c) => {
                self.add_to_identifier(c);
                return
            },
            _ => ()
        };

        match c {
            '(' => self.add_single_token(TokenType::LeftParen, c),
            ')' => self.add_single_token(TokenType::RightParen, c),
            '{' => self.add_single_token(TokenType::LeftBrace, c),
            '}' => self.add_single_token(TokenType::RightBrace, c),
            ',' => self.add_single_token(TokenType::Comma, c),
            '.' => self.add_single_token(TokenType::Dot, c),
            '-' => self.add_single_token(TokenType::Minus, c),
            '+' => self.add_single_token(TokenType::Plus, c),
            ';' => self.add_single_token(TokenType::SemiColon, c),
            '*' => self.add_single_token(TokenType::Star, c),
            '!' => self.add_multi_token(c),
            '=' => self.add_multi_token(c),
            '<' => self.add_multi_token(c),
            '>' => self.add_multi_token(c),
            '/' => self.add_multi_token(c),
            '\n' => self.white_space(c),
            '\t' => self.white_space(c),
            '\r' => self.white_space(c),
            ' ' => self.white_space(c),
            '"' => self.add_to_string(c),
            c if valid_digit(c) => self.add_to_number(c),
            c if valid_identifier(c) => self.add_to_identifier(c),
            _ => self.add_error(self.line, "Unexpected character")
        };
    }

    fn add_single_token(&mut self, token_type: TokenType, c: char) {
        self.add_saved_token(c);
        self.advance(c);
        self.add_token(token_type);
    }

    fn add_multi_token(&mut self, c: char) {
        self.add_saved_token(c);
        self.advance(c);
    }

    fn add_to_string(&mut self, c: char) {
        if self.literal == Literal::Nothing {
            self.literal = Literal::Str("".to_owned());
            return
        }

        if c == '"' {
            self.add_token(TokenType::Str);
            return
        }

        self.advance(c);
    }

    fn add_to_number(&mut self, c: char) {
        if self.literal == Literal::Nothing {
            self.literal = Literal::Number(0.0);
        }
        self.advance(c);
    }

    fn add_to_identifier(&mut self, c: char) {
        if self.literal == Literal::Nothing {
            self.literal = Literal::Identifier;
        }
        self.advance(c);
    }

    fn add_saved_token(&mut self, c: char) {
        match &self.literal {
            Literal::Str(_) => self.add_error(self.line, "Unterminated string"),
            Literal::Number(_) => self.add_token(TokenType::Number),
            Literal::Identifier => self.add_token(self.identifier_type()),
            Literal::Comment => return,
            _ => ()
        };

        match self.current_token.as_str() {
            "/" if c != '/' => self.add_token(TokenType::Slash),
            "=" if c != '=' => self.add_token(TokenType::Equal),
            "!" if c != '=' => self.add_token(TokenType::Bang),
            "<" if c != '=' => self.add_token(TokenType::Less),
            ">" if c != '=' => self.add_token(TokenType::Greater),
            "!=" => self.add_token(TokenType::BangEqual),
            "==" => self.add_token(TokenType::EqualEqual),
            "<=" => self.add_token(TokenType::LessEqual),
            ">=" => self.add_token(TokenType::GreaterEqual),
            "//" => self.add_comment(),
            "" => (),
            _ => ()
        };
    }

    fn add_token(&mut self, token_type: TokenType) {
        match self.literal {
            Literal::Str(_) => self.literal = Literal::Str(self.current_token.clone()),
            Literal::Number(_) => self.literal = Literal::Number(self.current_token.parse().unwrap()),
            _ => ()
        };
        self.tokens.push(Token::new(token_type, self.current_token.to_owned(), self.literal.to_owned(), self.line));
        self.current_token = String::new();
        self.literal = Literal::Nothing;
    }

    fn identifier_type(&self) -> TokenType {
        dbg!(&self.current_token);
        match self.current_token.as_str() {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fun" => TokenType::Fun,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "print" => TokenType::Print,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "var" => TokenType::Var,
            "while" => TokenType::While,
            _ => TokenType::Identifier
        }
    }

    fn advance(&mut self, c: char) {
        self.current_token.push(c);
    }

    fn white_space(&mut self, c: char) {
        if c == '\n' {
            self.new_line();
        }
        self.add_saved_token(c);
    }

    fn new_line(&mut self) {
        self.line += 1;
        if self.literal == Literal::Comment {
            self.literal = Literal::Nothing;
        }
    }

    fn add_comment(&mut self) {
        self.literal = Literal::Comment;
        self.current_token = String::new();
    }

    fn add_error(&mut self, line: u32, message: &str) {
        self.errors.push(format!("{} {}", line, message));
    }
}

fn valid_digit(c: char) -> bool {
    c.is_digit(10) || c == '.'
}

fn valid_identifier(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}