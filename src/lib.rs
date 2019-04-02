//! loxrs is a Lox implementation written in Rust.
//! It's just a hobby project and not so sopihsticated.

// TODO: not using extern crate?
extern crate itertools;

mod abs;
mod parser;
mod scanner;
mod token;

use abs::stmt::Stmt;
use parser::Parser;
use scanner::Scanner;

use std::fs::{self, File};
use std::io::{self, Write}; // flush()

pub fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Err(why) => {
            println!("{:?}", why);
            ::std::process::exit(1);
        }
        Ok(s) => s,
    };
    let (tokens, scan_errors) = Scanner::new(&source).scan();
    self::print_all("scan errors:", &scan_errors);
    self::print_all("tokens:", &tokens);
    let (mut stmts, parse_errors) = Parser::new(&tokens).parse();
    self::print_all("parse errors:", &parse_errors);
    self::interpret(&mut stmts);
}

// TODO: more generic code
fn print_all<T>(description: &str, items: &[T])
where
    T: std::fmt::Debug,
{
    println!("{}", description);
    for i in items {
        println!("  {:?}", i);
    }
    println!("");
}

pub fn run_repl() {
    println!("Entered Lox REPL");
    let prompt_str = "> ";
    let mut line = String::new();
    loop {
        print!("{}", prompt_str);
        io::stdout().flush().expect("error when flushing stdout");
        line.clear();
        io::stdin()
            .read_line(&mut line)
            .expect("error when reading stdin");
        match line.trim_end() {
            "q" | "quit" => {
                break;
            }
            _ => {}
        }
    }
}

pub fn interpret(stmts: &mut [Stmt]) {
    println!("interuption:");
    for stmt in stmts {
        // println!("  stmt: {:?}", stmt);
        stmt.execute();
    }
}
