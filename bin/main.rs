use std::env; // args
use std::vec::Vec;

fn main() {
    ::env_logger::init();

    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 | 1 => {
            loxrs::run_repl();
        }
        n if n >= 2 => {
            loxrs::run_file(&args[1]);
        }
        _ => {
            eprintln!("Given more than one arguments");
        }
    }
}
