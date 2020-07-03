use super::token::{Token, TokenType};
#[derive(Debug, Clone, Copy, PartialEq)]
enum Kind {
    Str,
    Number,
    Comment,
    MultiComment,
    Identifier,
    Nothing
}
#[derive(Debug)]
pub struct Scanner<'a> {
    source: &'a str,
    tokens: Vec<Token>,
    current_token: String,
    current_kind: Kind,
    current_token_number: u32,
    line: u32,
    errors: Vec<String>
}

impl<'a> Scanner<'a> {
    pub fn new(source: &str) -> Scanner {
        Scanner {
            source: source,
            tokens: Vec::new(),
            current_token: String::new(),
            current_kind: Kind::Nothing,
            current_token_number: 0,
            line: 1,
            errors: Vec::new()
        }
    }

    pub fn scan_tokens(&mut self) -> &Vec<Token> {
        for c in self.source.chars() {
            self.scan_token(c);
        }
        self.add_saved_token('\0');
        self.tokens.push(Token::new(TokenType::Eof, String::new(), self.line, self.current_token_number));
        &self.tokens
    }

    fn scan_token(&mut self, c: char) {
        match &self.current_kind {
            Kind::Comment if c != '\n' => return,
            Kind::MultiComment if !self.multi_comment_end(c) => return,
            Kind::Str => {
                self.add_to_string(c);
                return
            },
            Kind::Number if valid_digit(c) => { 
                self.add_to_number(c);
                return
            },
            Kind::Identifier if valid_identifier(c) => {
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
            '?' => self.add_single_token(TokenType::QuestionMark, c),
            ':' => self.add_single_token(TokenType::Colon, c),
            '*' => self.add_multi_token(c),
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
        if self.current_kind == Kind::Nothing {
            self.current_kind = Kind::Str;
            return
        }

        if c == '"' {
            self.add_token(TokenType::Str);
            return
        }

        self.advance(c);
    }

    fn add_to_number(&mut self, c: char) {
        if self.current_kind == Kind::Nothing {
            self.current_kind = Kind::Number;
        }
        self.advance(c);
    }

    fn add_to_identifier(&mut self, c: char) {
        if self.current_kind == Kind::Nothing {
            self.current_kind = Kind::Identifier;
        }
        self.advance(c);
    }

    fn add_saved_token(&mut self, c: char) {
        match &self.current_kind {
            Kind::Str => self.add_error(self.line, "Unterminated string"),
            Kind::Number => self.add_token(TokenType::Number),
            Kind::Identifier => self.add_token(self.identifier_type()),
            Kind::Comment => return,
            Kind::MultiComment if self.current_token.as_str() != "*/" => return,
            _ => ()
        };

        match self.current_token.as_str() {
            "/" if c != '/' && c != '*' => self.add_token(TokenType::Slash),
            "*" if c != '/' && self.current_kind != Kind::MultiComment => self.add_token(TokenType::Star),
            "*" if c != '/' && self.current_kind == Kind::MultiComment => self.current_token = String::new(),
            "=" if c != '=' => self.add_token(TokenType::Equal),
            "!" if c != '=' => self.add_token(TokenType::Bang),
            "<" if c != '=' => self.add_token(TokenType::Less),
            ">" if c != '=' => self.add_token(TokenType::Greater),
            "!=" => self.add_token(TokenType::BangEqual),
            "==" => self.add_token(TokenType::EqualEqual),
            "<=" => self.add_token(TokenType::LessEqual),
            ">=" => self.add_token(TokenType::GreaterEqual),
            "//" => self.add_comment(),
            "/*" => self.add_comment(),
            "*/" => self.end_comment(),
            "" => (),
            _ => ()
        };
    }

    fn add_token(&mut self, token_type: TokenType) {
        self.tokens.push(Token::new(token_type, self.current_token.to_owned(), self.line, self.current_token_number));
        self.current_token = String::new();
        self.current_kind = Kind::Nothing;
        self.current_token_number += 1;
    }

    fn advance(&mut self, c: char) {
        self.current_token.push(c);
    }

    fn identifier_type(&self) -> TokenType {
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

    fn white_space(&mut self, c: char) {
        if c == '\n' {
            self.new_line();
        }
        self.add_saved_token(c);
    }

    fn new_line(&mut self) {
        self.line += 1;
        if self.current_kind == Kind::Comment {
            self.current_kind = Kind::Nothing;
        }
    }

    fn add_comment(&mut self) {
        if self.current_token.as_str() == "//" {
            self.current_kind = Kind::Comment;
        } else {
            self.current_kind = Kind::MultiComment;
        }
        self.current_token = String::new();
    }

    fn end_comment(&mut self) {
        self.current_kind = Kind::Nothing;
        self.current_token = String::new();
    }

    fn add_error(&mut self, line: u32, message: &str) {
        self.errors.push(format!("{} {}", line, message));
    }

    fn multi_comment_end(&self, c: char) -> bool {
        if self.current_token.as_str() == "*/" {
            return true
        }

        if self.current_token.as_str() == "*" && c == '/' {
            return true
        }

        if c == '*' {
            return true
        }

        false
    }
}

fn valid_digit(c: char) -> bool {
    c.is_digit(10) || c == '.'
}

fn valid_identifier(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}