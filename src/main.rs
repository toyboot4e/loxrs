use std::env; // args
use std::vec::Vec;

fn main() {
    ::env_logger::init();

    let cx = loxrs::RunContext { is_debug: true };
    let args: Vec<String> = env::args().collect();
    match args.len() {
        0 | 1 => {
            loxrs::run_repl();
        }
        n if n >= 2 => {
            // loxrs::run_file(&args[1]);
            loxrs::run_file(&args[1], &cx);
        }
        _ => {
            eprintln!("Given more than one argument");
        }
    }
}
