//! loxrs ommand line interface

use {
    clap::Clap,
    loxrs_vm::parse::lexer::Lexer,
    std::io::{self, prelude::*},
    termcolor::{BufferWriter, Color, ColorChoice, ColorSpec, WriteColor},
};

use loxrs_vm::parse::token::Token;

pub type Result<T> = anyhow::Result<T>;

pub fn run() -> Result<()> {
    let mut x = Cli::parse();
    x.run()
}

// `adbook`
#[derive(Clap, Debug)]
#[clap(
    name = "loxrs is an interpreter",
    setting = clap::AppSettings::ColoredHelp
)]
pub struct Cli {
    #[clap(subcommand)]
    pub cmd: SubCommand,
}

impl Cli {
    pub fn run(&mut self) -> Result<()> {
        self.cmd.run()
    }
}

#[derive(Clap, Debug)]
pub enum SubCommand {
    #[clap(name = "lex")]
    /// Paints stdin as a loxrs file
    Lex(Lex),
}

impl SubCommand {
    pub fn run(&mut self) -> Result<()> {
        match self {
            SubCommand::Lex(cmd) => cmd.run(),
        }
    }
}

/// `loxrs lex`
#[derive(Clap, Debug)]
pub struct Lex {}

fn color_for_token(tk: Token) -> Color {
    match tk {
        Token::LineComment => Color::Rgb(128, 128, 128),
        Token::Ident => Color::Cyan,
        _ => Color::White,
    }
}

impl Lex {
    pub fn run(&mut self) -> Result<()> {
        let src = {
            let mut buf = String::new();
            io::stdin().read_to_string(&mut buf)?;
            buf
        };

        // spanned tokens in the buffer
        let stks = {
            let mut stks = Vec::with_capacity(100);
            let mut lex = Lexer::new(&src);
            loop {
                let stk = lex.next_stk()?;

                if stk.tk == Token::Eof {
                    break;
                }

                stks.push(stk);
            }
            stks
        };

        let stdout = BufferWriter::stdout(ColorChoice::Always);

        let mut outbuf = stdout.buffer();
        let mut color = Color::White;
        outbuf.set_color(ColorSpec::new().set_fg(Some(color)))?;

        for stk in &stks {
            let new_color = self::color_for_token(stk.tk);
            if new_color != color {
                outbuf.set_color(ColorSpec::new().set_fg(Some(new_color)))?;
                color = new_color;
            }

            let s = stk.slice(&src);
            write!(outbuf, "{}", s)?;
        }

        stdout.print(&outbuf)?;

        Ok(())
    }
}
