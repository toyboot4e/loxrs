mod scanner;
mod token;
pub use scanner::Scanner;
pub use token::Token;

use std::format; // format!
use std::fs::File; // open
use std::io::{self, Read, Write}; // read_to_string(), flush()

pub fn run_file(path: &str) {
    let mut file = File::open(path).expect(&format!("not found file: {}", path));
    let mut source = String::new();
    file.read_to_string(&mut source)
        .expect("could not read file to string");
    let mut scanner = Scanner::new(&source);
    let (tokens, errors) = scanner.scan();
    for token in tokens {
        println!("{:?}", token);
    }
    for error in errors {
        println!("{:?}", error);
    }
}

pub fn run_repl() {
    println!("Entered Lox REPL");
    let prompt_str = "> ";
    let mut line = String::new();
    loop {
        print!("{}", prompt_str);
        io::stdout().flush().expect("error when flush stdout");
        line.clear();
        io::stdin()
            .read_line(&mut line)
            .expect("error when read_line");
        match line.trim_right() {
            "q" | "quit" => {
                break;
            }
            _ => {}
        }
    }
}

pub struct Lox {}
