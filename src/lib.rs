//! loxrs is a Lox implementation in Rust.

#![allow(dead_code)]
#![allow(unused_variables)]
#![warn(rust_2018_idioms)]

mod analizer;
mod ast;
mod lexer;
mod runtime;

use crate::analizer::resolver::Resolver;
use crate::ast::{stmt::Stmt, PrettyPrint};
use crate::lexer::{parser::Parser, scanner::Scanner};
use crate::runtime::Interpreter;

use std::fs;
use std::io::{self, BufRead, BufWriter, Write}; // flush()

pub fn run_file(path: &str) {
    let source = match fs::read_to_string(path) {
        Err(why) => {
            println!("{} (given path: `{}`)", why, path);
            ::std::process::exit(1);
        }
        Ok(s) => s,
    };
    let (tokens, scan_errors) = Scanner::new(&source).scan();
    let (mut stmts, parse_errors) = Parser::new(&tokens).parse();
    if parse_errors.len() > 0 {
        println!("parse error");
        return;
    }
    let mut interpreter = Interpreter::new();
    {
        let mut resolver = Resolver::new(&mut interpreter.caches);
        if let Err(why) = resolver.resolve_stmts(&mut stmts) {
            println!("====== resolving error ======");
            println!("{:?}", why);
            return;
        }
    }
    self::interpret(&mut stmts, &mut interpreter);
}

/// Runs a file with debug output (including lexer output)
pub fn run_file_debug(path: &str) {
    let source = match fs::read_to_string(path) {
        Err(why) => {
            println!("{}", why);
            ::std::process::exit(1);
        }
        Ok(s) => s,
    };

    let (tokens, scan_errors) = Scanner::new(&source).scan();
    self::print_all_debug("====== scan errors =====", scan_errors);
    self::print_all_debug("====== tokens =====", &tokens);

    let (mut stmts, parse_errors) = Parser::new(&tokens).parse();
    self::print_all_debug("===== parse errors =====", &parse_errors);
    self::print_all_display("===== AST =====", stmts.iter().map(|s| s.pretty_print()));

    let mut interpreter = Interpreter::new();
    {
        let mut resolver = Resolver::new(&mut interpreter.caches);
        if let Err(why) = resolver.resolve_stmts(&mut stmts) {
            println!("====== resolving error ======");
            println!("{:?}", why);
            return;
        }
    }
    self::interpret(&mut stmts, &mut interpreter);
}

fn print_all_debug(description: &str, items: impl IntoIterator<Item = impl ::std::fmt::Debug>) {
    let out = io::stdout();
    let mut out = BufWriter::new(out.lock());
    writeln!(out, "{}", description).unwrap();
    for i in items {
        writeln!(out, "{:?}", i).unwrap();
    }
    writeln!(out).unwrap();
}

fn print_all_display(description: &str, items: impl IntoIterator<Item = impl ::std::fmt::Display>) {
    let out = io::stdout();
    let mut out = BufWriter::new(out.lock());
    writeln!(out, "{}", description).unwrap();
    for i in items {
        writeln!(out, "{}", i).unwrap();
    }
    writeln!(out).unwrap();
}

pub fn run_repl() {
    println!("Entered Lox REPL");
    let prompt_str = "> ";

    let mut line = String::new();
    // We can use [LineWriter](https://doc.rust-lang.org/std/io/struct.LineWriter.html) instead
    // to automate flushing
    let out = io::stdout();
    let mut out = BufWriter::new(out.lock());
    let handle = io::stdin();
    let mut handle = handle.lock();
    loop {
        print!("{}", prompt_str);
        out.flush().expect("error when flushing stdout");
        line.clear();
        handle
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

pub fn interpret(stmts: &mut [Stmt], interpreter: &mut Interpreter) {
    println!("====== interpretations =====");
    match stmts
        .iter()
        .enumerate()
        .map(|(i, stmt)| (i, interpreter.interpret(stmt)))
        .find(|(i, result)| result.is_err())
    {
        Some((i, err)) => {
            println!("\n====== runtime errors =====");
            println!("at {}, {:?}", i, err);
        }
        None => {
            //
        }
    }
}
