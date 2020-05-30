pub mod chunk;
pub mod compiler;
pub mod vm;

use crate::vm::{Vm, VmError};
use std::{
    fs,
    io::{self, BufRead, BufWriter, Write},
    path::Path,
};

type Result<T> = ::std::result::Result<T, VmError>;

pub fn interpret(s: &str) -> Result<()> {
    Ok(())
}

pub fn run_file(file: &Path) -> Result<()> {
    let s = fs::read_to_string(file).or_else(|_| Err(VmError::RuntimeError))?;
    self::interpret(&s)
}

pub fn run_repl() {
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
                // if let Some(Err(why)) = self::run_string(line, cx, &mut interpreter) {
                //     println!("{:?}", why);
                // }
            }
        }
    }
}
