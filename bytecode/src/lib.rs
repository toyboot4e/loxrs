pub mod chunk;
pub mod compiler;
pub mod vm;

use crate::vm::{Vm, VmError};
use anyhow::{anyhow, Context, Error, Result};
use std::{
    fs,
    io::{self, BufRead, BufWriter, Write},
    path::Path,
};

pub fn interpret(src: &str) -> Result<()> {
    // let x = compiler::compile(src);
    Ok(())
}

pub fn run_file(file: &Path) -> Result<()> {
    let s = fs::read_to_string(file)
        .with_context(|| format!("when opening file {}", file.display()))?;
    self::interpret(&s)
}

pub fn run_repl() -> Result<()> {
    println!("loxrs REPL (bytecode) [press q<Enter> or Ctrl-c to quit]");
    let prompt_str = "> ";

    // setting up I/O
    let mut line = String::new();

    let out = io::stdout();
    let mut out = BufWriter::new(out.lock());
    let input = io::stdin();
    let mut handle = input.lock();

    let mut vm = Vm::new();

    loop {
        print!("{}", prompt_str);
        out.flush().context("error when flushing stdout")?;

        line.clear();
        handle.read_line(&mut line).context("when reading stdin")?;

        match line.trim_end() {
            "q" | "quit" => {
                break;
            }
            line => {
                // if let Err(why) = self::interpret(line) {
                //     //
                // }
                //     println!("{:?}", why);
                // }
            }
        }
    }

    Ok(())
}
