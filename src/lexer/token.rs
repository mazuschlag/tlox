use std::fmt;

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum TokenType {
    // Single character tokens
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    SemiColon,
    Slash,
    Star,
    // One or two character tokens
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater, 
    GreaterEqual,
    Less, 
    LessEqual,
    // Literals
    Identifier,
    Str,
    Number,
    // Keywords
    And,
    Class,
    Else,
    False,
    Fun,
    For,
    If,
    Nil,
    Or,
    Print,
    Return,
    Super,
    This,
    True,
    Var,
    While,
    Eof
}


impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Debug)]
pub struct Token {
    typ: TokenType,
    lexme: String,
    literal: Literal,
    line: u32
}

impl Token {
    pub fn new(typ: TokenType, lexme: String, literal: Literal, line: u32) -> Token {
        Token {
            typ,
            lexme,
            literal,
            line
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       write!(f, "{} {} {}", self.typ, self.lexme, self.literal)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum Literal {
    Str(String),
    Number(f32),
    Comment,
    Identifier,
    Nothing
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
