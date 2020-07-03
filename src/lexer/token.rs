use std::fmt;

#[derive(Debug, PartialEq, Copy, Clone, Eq, Hash)]
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
    Colon,
    QuestionMark,
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

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Token {
    pub typ: TokenType,
    pub lexeme: String,
    pub line: u32,
    pub number: u32
}

impl Token {
    pub fn new(typ: TokenType, lexeme: String, line: u32, number: u32) -> Token {
        Token {
            typ,
            lexeme,
            line,
            number
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       write!(f, "{} {} {}", self.typ, self.lexeme, self.number)
    }
}
