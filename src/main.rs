mod arena;
mod error;
mod interpreter;
mod lexer;
mod parser;

use interpreter::interpreter::Interpreter;
use lexer::scanner::Scanner;

use parser::parser::{Parser, ParseOutput};
use parser::resolver::Resolver;

use std::env;
use std::fs::File;
use std::io::{self, BufRead, Read, Write};

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => run_file(&args[1]),
        1 => run_prompt(),
        _ => println!("Usage: tlox [script]"),
    }
}

fn run_file(file_name: &str) {
    let mut source_file = File::open(file_name).unwrap();
    let mut input = String::new();
    source_file.read_to_string(&mut input).unwrap();
    run(&input, false);
}

fn run_prompt() {
    new_line();
    for line in io::stdin().lock().lines() {
        match line {
            Ok(input) => {
                run(&input, true);
                new_line();
            }
            Err(error) => println!("Error reading line: {}", error),
        }
    }
}

fn run(source: &str, is_repl: bool) {
    let scanner = Scanner::new(source);
    let tokens = scanner.scan_tokens();
    
    let parsed = match Parser::new(tokens, is_repl).run() {
        Ok(ParseOutput(program, pools)) => {
            Resolver::new().run(program, pools)
        }
        Err(errors) => Err(errors.into_iter().map(|err| format!("{}\n", err)).collect::<String>()),
    };

    match parsed {
        Ok((program, pools, locals)) => {
            Interpreter::new(locals).run(program, pools)
        },
        Err(e) => println!("{}", e),
    }
}

fn new_line() {
    print!("> ");
    io::stdout().flush().unwrap();
}
