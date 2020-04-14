use crate::lexer::token::{Token, TokenType};

pub struct RuntimeError {
    token: Token,
    message: String
}

impl RuntimeError {
    pub fn new(token: Token, message: &str) -> RuntimeError {
        RuntimeError {
            token: token,
            message: message.to_owned()
        }
    }
}

pub fn runtime_report(err: RuntimeError) -> String {
    error(&err.token, &err.message)
}

pub fn error(token: &Token, message: &str) -> String {
    match token.typ {
        TokenType::Eof => report(token.line, "at end", message),
        _ => report(token.line, &format!("at '{}'", token.lexeme), message)
    }
}

fn report(line: u32, offender: &str, message: &str) -> String {
    format!("[line {}] Error {}: {}", line, offender, message)
}