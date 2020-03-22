mod lexer;

use std::env;
use std::fs::File;
use std::io::{self, Read, BufRead, Write};
use lexer::scanner::Scanner;

fn main() {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        2 => run_file(&args[1]),
        1 => run_prompt(),
        _ => println!("Usage: jlox [script]")
    }
}

fn run_file(file_name: &str) {
    let mut source_file = File::open(file_name).unwrap();
    let mut input = String::new();
    source_file.read_to_string(&mut input).unwrap();
    run(&input);
}

fn run_prompt() {
    new_line();
    for line in io::stdin().lock().lines() {
        match line {
            Ok(input) => { 
                run(&input);
                new_line();
            },
            Err(error) => println!("Error reading line: {}", error)
        }
    }
}

fn run(source: &str) {
    let mut scanner = Scanner::new(source);
    println!("Tokens: {:?}", scanner.scan_tokens());
}

fn new_line() {
    print!("> ");
    io::stdout().flush().unwrap();
}

// Should actually use Result enum, but implement later
fn _error(line: u32, message: &str) {
    _report(line, "", message);
}

fn _report(line: u32, offender: &str, message: &str) {
    println!("[line {}] Error {}: {}", line, offender, message);
}