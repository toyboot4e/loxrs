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
use crate::runtime::{obj::LoxObj, Interpreter, Result};

use std::fs;
use std::io::{self, BufRead, BufWriter, Write};

// ***** cli / arg parse *****

#[derive(Default)]
pub struct RunContext {
    /// If true, tokens and AST are printed
    pub is_debug: bool,
    pub is_repl: bool,
}

#[derive(Default)]
pub struct Cli {
    pub cx: RunContext,
    pub run_file: Option<String>,
}

impl Cli {
    pub fn run(&self) {
        if let Some(file) = self.run_file.as_ref() {
            self::run_file(file, &self.cx);
        } else {
            self::run_repl(&self.cx);
        }
    }
}

pub fn parse_args() -> Cli {
    let mut cli = Cli::default();

    let args: Vec<String> = ::std::env::args().collect();
    for arg in args.iter().skip(1) {
        match arg.as_str() {
            "-d" | "--debug" => cli.cx.is_debug = true,
            arg => {
                if cli.run_file.is_none() {
                    cli.run_file = Some(arg.to_string());
                } else {
                    eprintln!("Given more than one argument");
                    ::std::process::exit(1);
                }
            }
        }
    }

    cli.cx.is_repl = cli.run_file.is_none();
    cli
}

// ***** run file *****

// TODO: buffering for reading source files
pub fn run_file(path: &str, cx: &RunContext) {
    let source = match fs::read_to_string(path) {
        Err(why) => {
            println!("{} (given path: `{}`)", why, path);
            ::std::process::exit(1);
        }
        Ok(s) => s,
    };
    let mut interpreter = Interpreter::new();
    self::run_string(&source, cx, &mut interpreter);
}

/// Returns a `Result` of the interpretation if parse & Resolving succeeded
pub fn run_string(
    source: &str,
    cx: &RunContext,
    interpreter: &mut Interpreter,
) -> Option<Result<LoxObj>> {
    // scanning
    let (tokens, scan_errors) = Scanner::new(&source).scan();
    if cx.is_debug {
        self::print_all_debug(&tokens, "====== tokens =====");
    }
    if scan_errors.len() > 0 {
        self::print_all_debug(&scan_errors, "====== scan errors =====");
        return None;
    }

    // parsing
    let (mut stmts, parse_errors) = Parser::new(&tokens).parse();
    if cx.is_debug {
        self::print_all_display(
            stmts
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{} {}", i, s.pretty_print())),
            "===== AST =====",
        );
    }
    if parse_errors.len() > 0 {
        self::print_all_debug(&parse_errors, "===== parse errors =====");
        return None;
    }

    {
        // analizing
        let mut resolver = Resolver::new(&mut interpreter.caches);
        if let Err(why) = resolver.resolve_stmts(&mut stmts) {
            println!("====== resolving error ======");
            println!("{:?}", why);
            return None;
        }
    }

    Some(self::interpret(interpreter, &mut stmts, cx))
}

fn print_all_debug<T, U>(items: U, header: &str)
where
    T: ::std::fmt::Debug,
    U: IntoIterator<Item = T>,
{
    let out = io::stdout();
    let mut out = BufWriter::new(out.lock());
    writeln!(out, "{}", header).unwrap();

    for i in items {
        writeln!(out, "{:?}", i).unwrap();
    }
    writeln!(out).unwrap();
}

fn print_all_display<T, U>(items: U, header: &str)
where
    T: ::std::fmt::Display,
    U: IntoIterator<Item = T>,
{
    let out = io::stdout();
    let mut out = BufWriter::new(out.lock());
    writeln!(out, "{}", header).unwrap();

    for i in items {
        writeln!(out, "{}", i).unwrap();
    }
    writeln!(out).unwrap();
}

pub fn interpret(
    interpreter: &mut Interpreter,
    stmts: &mut [Stmt],
    cx: &RunContext,
) -> Result<LoxObj> {
    if !cx.is_repl && cx.is_debug {
        println!("====== interpretations =====");
    }
    let mut result = Ok(None);
    for (i, stmt) in stmts.iter().enumerate() {
        result = interpreter.interpret(stmt);
        if let Err(why) = result.as_ref() {
            if !cx.is_repl && cx.is_debug {
                eprintln!("\n====== runtime errors =====");
            }
            eprintln!("at {}, {:?}", i, why);
            return result.map(|opt| opt.unwrap_or(LoxObj::nil()));
        }
    }
    return result.map(|opt| opt.unwrap_or(LoxObj::nil()));
}

// ********** REPL **********

pub fn run_repl(cx: &RunContext) {
    println!("Entered loxrs REPL (press q<Enter> or Ctrl-c to quit)");
    let prompt_str = "> ";

    let mut line = String::new();
    // We can use [LineWriter](https://doc.rust-lang.org/std/io/struct.LineWriter.html) instead
    // to automate flushing
    let out = io::stdout();
    let mut out = BufWriter::new(out.lock());
    let handle = io::stdin();
    let mut handle = handle.lock();

    let mut interpreter = Interpreter::new();
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
            line => {
                if let Some(Err(why)) = self::run_string(line, cx, &mut interpreter) {
                    println!("{:?}", why);
                }
            }
        }
    }
}
