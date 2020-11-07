//! Loxrs bytecode interpreter

pub mod chunk;
pub mod compiler;
pub mod parser;
pub mod vm;

#[macro_use]
extern crate anyhow;

#[macro_use]
extern crate log;

use {
    anyhow::{Context, Error, Result},
    std::{
        fs,
        io::{self, prelude::*, BufWriter},
        path::Path,
    },
};

use crate::vm::{Vm, VmError};

pub fn interpret(vm: &mut Vm, src: &str) -> Result<()> {
    // let x = compiler::compile(src);
    Ok(())
}

pub fn run_file(file: &Path) -> Result<()> {
    let s = fs::read_to_string(file)
        .with_context(|| format!("when opening file {}", file.display()))?;
    let mut vm = Vm::new();
    self::interpret(&mut vm, &s)
}

pub fn run_repl() -> Result<()> {
    println!("loxrs REPL (bytecode) [press q<Enter> or Ctrl-c to quit]");
    let prompt_str = "> ";

    // setting up I/O
    let out = io::stdout();
    let mut out = BufWriter::new(out.lock());

    let input = io::stdin();
    let mut input = input.lock();

    let mut line = String::new();
    let mut vm = Vm::new();

    loop {
        print!("{}", prompt_str);
        out.flush().context("error when flushing stdout")?;

        line.clear();
        input.read_line(&mut line).context("when reading stdin")?;

        match line.trim_end() {
            "q" | "quit" => {
                break;
            }
            line => match self::interpret(&mut vm, line) {
                Err(why) => eprintln!("Error: {}", why),
                Ok(()) => {
                    println!("run without error");
                }
            },
        }

        vm.clear_stack();
    }

    Ok(())
}
