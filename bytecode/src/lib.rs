pub mod chunk;
pub mod vm;

use crate::vm::{Vm, VmInterpretError};
use std::fs;
use std::io::{self, BufRead, BufWriter, Write};

type Result<T> = ::std::result::Result<T, VmInterpretError>;

pub fn interpret(s: &str) -> Result<()> {
    Ok(())
}

pub fn run_repl() {
    println!("Entered loxrs (bytecode) REPL (press q<Enter> or Ctrl-c to quit)");
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
