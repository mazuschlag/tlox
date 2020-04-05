use crate::lexer::token::{Token, TokenType};

pub fn error(token: &Token, message: &str) -> String {
    match token.typ {
        TokenType::Eof => report(token.line, "at end", message),
        _ => report(token.line, &format!("at '{}'", token.lexme), message)
    }
}

fn report(line: u32, offender: &str, message: &str) -> String {
    format!("[line {}] Error {}: {}", line, offender, message)
}