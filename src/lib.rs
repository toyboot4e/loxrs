//! loxrs is an implementation of Lox in Rust.

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
use std::io::{self, BufRead, BufWriter, Write};

pub struct RunContext {
    pub is_debug: bool,
}

// TODO: buffering for reading source files
pub fn run_file(path: &str, cx: &RunContext) {
    let source = match fs::read_to_string(path) {
        Err(why) => {
            println!("{} (given path: `{}`)", why, path);
            ::std::process::exit(1);
        }
        Ok(s) => s,
    };

    let (tokens, scan_errors) = Scanner::new(&source).scan();
    if cx.is_debug {
        self::print_all_debug(&scan_errors, "====== scan errors =====");
        self::print_all_debug(&tokens, "====== tokens =====");
    }

    let (mut stmts, parse_errors) = Parser::new(&tokens).parse();
    if cx.is_debug {
        self::print_all_debug(&parse_errors, "===== parse errors =====");
        self::print_all_display(
            stmts
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{} {}", i, s.pretty_print())),
            "===== AST =====",
        );
    }
    if parse_errors.len() > 0 {
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
    self::interpret(&mut interpreter, &mut stmts, cx);
}

fn print_all_debug(items: impl IntoIterator<Item = impl ::std::fmt::Debug>, description: &str) {
    let out = io::stdout();
    let mut out = BufWriter::new(out.lock());
    writeln!(out, "{}", description).unwrap();

    for i in items {
        writeln!(out, "{:?}", i).unwrap();
    }
    writeln!(out).unwrap();
}

fn print_all_display(items: impl IntoIterator<Item = impl ::std::fmt::Display>, description: &str) {
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

pub fn interpret(interpreter: &mut Interpreter, stmts: &mut [Stmt], cx: &RunContext) {
    if cx.is_debug {
        println!("====== interpretations =====");
    }
    for (i, stmt) in stmts.iter().enumerate() {
        if let Err(why) = interpreter.interpret(stmt) {
            println!("\n====== runtime errors =====");
            println!("at {}, {:?}", i, why);
            return;
        }
    }
}
