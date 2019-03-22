use std::env; // args
use std::io; // Result
use std::vec::Vec;
extern crate lox;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 | 1 => {
            lox::run_repl();
        }
        x if x >= 2 => {
            lox::run_file(&args[1]);
        }
        _ => {
            panic!("");
        }
    }

    return Ok(());
}
