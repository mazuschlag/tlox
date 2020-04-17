mod lexer;
mod parser;
mod error;
mod interpreter;

use std::env;
use std::fs::File;
use std::io::{self, Read, BufRead, Write};
use lexer::scanner::Scanner;
use parser::parser::Parser;
use interpreter::interpreter::Interpreter;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => run_file(&args[1]),
        1 => run_prompt(),
        _ => println!("Usage: tlox [script]")
    }
}

fn run_file(file_name: &str) {
    let mut source_file = File::open(file_name).unwrap();
    let mut input = String::new();
    let mut interpreter = Interpreter::new();
    source_file.read_to_string(&mut input).unwrap();
    run(&mut interpreter, &input);
}

fn run_prompt() {
    new_line();
    let mut interpreter = Interpreter::new();
    for line in io::stdin().lock().lines() {
        match line {
            Ok(input) => { 
                run(&mut interpreter, &input);
                new_line();
            },
            Err(error) => println!("Error reading line: {}", error)
        }
    }
}

fn run(interpreter: &mut Interpreter, source: &str) {
    let mut scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    let mut parser = Parser::new(tokens);
    parser.parse();
    if parser.errors.len() > 0 {
        parser.errors.iter().for_each(|err| { println!("{}", err) });
        return
    }
    let program = parser.statements;
    interpreter.interpret(&program);
}

fn new_line() {
    print!("> ");
    io::stdout().flush().unwrap();
}