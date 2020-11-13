//! Command line interface of `loxrs` tree-walk interpreter

pub use anyhow::Result;
use anyhow::{anyhow, Context, Error};
use std::{
    env, fs,
    io::{self, BufRead, BufWriter, Write},
};

use crate::{
    analizer::resolver::Resolver,
    ast::{stmt::Stmt, PrettyPrint},
    lexer::{parser::Parser, scanner::Scanner},
    runtime::{obj::LoxObj, Interpreter /*Result*/},
};

// --------------------------------------------------------------------------------
// API

pub fn parse() -> Result<Cli> {
    let mut cli = Cli::default();
    cli.parse_args()?;
    Ok(cli)
}

#[derive(Default)]
pub struct RunContext {
    /// If true, prints tokens and AST
    pub is_debug: bool,
    /// Is it read, evaluate and print loop?
    pub is_repl: bool,
}

/// The command line interface
#[derive(Default)]
pub struct Cli {
    pub cx: RunContext,
    pub run_file: Option<String>,
}

impl Cli {
    fn parse_args(&mut self) -> Result<()> {
        let args: Vec<String> = env::args().collect();
        for arg in args.iter().skip(1) {
            self.parse_arg(arg.as_str())?;
        }
        self.cx.is_repl = self.run_file.is_none();
        Ok(())
    }

    fn parse_arg(&mut self, arg: &str) -> Result<()> {
        match arg {
            "-d" | "--debug" => self.cx.is_debug = true,
            arg => {
                if self.run_file.is_some() {
                    return Err(anyhow!("Given more than one argument"));
                }
                self.run_file = Some(arg.to_string());
            }
        };
        Ok(())
    }

    pub fn run(&self) -> Result<()> {
        if let Some(file) = self.run_file.as_ref() {
            self::run_file(file, &self.cx)?;
        } else {
            self::run_repl(&self.cx)?;
        }
        Ok(())
    }
}

// --------------------------------------------------------------------------------
// Running

// TODO: buffering for reading source files
pub fn run_file(path: &str, cx: &RunContext) -> Result<LoxObj> {
    let src = fs::read_to_string(path).map_err(Error::msg)?;
    let mut interpreter = Interpreter::new();
    self::run_string(&src, cx, &mut interpreter)
}

/// Returns a `Result` of the interpretation if parse & Resolving succeeded
pub fn run_string(source: &str, cx: &RunContext, interpreter: &mut Interpreter) -> Result<LoxObj> {
    // scan
    let (tks, scan_errors) = Scanner::new(&source).scan();

    if cx.is_debug {
        self::print_all_debug("====== tokens =====", &tks);
    }
    if scan_errors.len() > 0 {
        self::print_all_debug("====== scan errors =====", &scan_errors);
        return Err(anyhow!("=> failed to scan"));
    }

    // parse
    let (mut stmts, parse_errors) = Parser::new(&tks).parse();

    if cx.is_debug {
        self::print_all_display(
            "===== AST =====",
            stmts
                .iter()
                .enumerate()
                .map(|(i, s)| format!("{} {}", i, s.pretty_print())),
        );
    }
    if parse_errors.len() > 0 {
        self::print_all_debug("===== parse errors =====", &parse_errors);
        return Err(anyhow!("=> failed to parse"));
    }

    // analizing
    let mut resolver = Resolver::new(&mut interpreter.caches);
    resolver.resolve_stmts(&mut stmts).map_err(Error::msg)?;

    self::interpret(interpreter, &mut stmts, cx)
}

pub fn interpret(
    interpreter: &mut Interpreter,
    stmts: &mut [Stmt],
    cx: &RunContext,
) -> Result<LoxObj> {
    if !cx.is_repl && cx.is_debug {
        println!("====== interpretations =====");
    }
    let mut res = Ok(None);
    for (i, stmt) in stmts.iter().enumerate() {
        res = interpreter.interpret(stmt);
        if let Err(why) = res {
            if !cx.is_repl && cx.is_debug {
                eprintln!("\n====== runtime errors =====");
            }
            eprintln!("at {}, {:?}", i, why);
            return Err(why).map_err(Error::msg);
        }
    }
    Ok(res.unwrap().unwrap_or(LoxObj::nil()))
}

// --------------------------------------------------------------------------------
// REPL

pub fn run_repl(cx: &RunContext) -> Result<()> {
    println!("Entered loxrs REPL (press q<Enter> or Ctrl-c to quit)");
    let prompt = "> ";

    let mut line = String::new();

    let out = io::stdout();
    let mut out = BufWriter::new(out.lock());
    let input = io::stdin();
    let mut input = input.lock();

    let mut interpreter = Interpreter::new();
    loop {
        print!("{}", prompt);
        out.flush().context("error when flushing stdout")?;
        line.clear();
        input
            .read_line(&mut line)
            .expect("error when reading stdin");

        match line.trim_end() {
            "q" | "quit" => {
                break;
            }
            line => match self::run_string(line, cx, &mut interpreter) {
                Ok(obj) => println!("{:?}", obj),
                Err(why) => println!("{:?}", why),
            },
        }
    }

    Ok(())
}

// --------------------------------------------------------------------------------
// utilities

fn print_all_debug<T, U>(header: &str, items: U)
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

fn print_all_display<T, U>(header: &str, items: U)
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
