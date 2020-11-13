//! loxrs ommand line interface

use {anyhow::*, clap::Clap};

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
    #[clap(name = "paint", alias = "p")]
    /// Paints stdin as a loxrs file
    Paint(Paint),
}

impl SubCommand {
    pub fn run(&mut self) -> Result<()> {
        match self {
            SubCommand::Paint(cmd) => cmd.run(),
        }
    }
}

/// `loxrs paint`
#[derive(Clap, Debug)]
pub struct Paint {
    pub file: Option<String>,
}

impl Paint {
    pub fn run(&mut self) -> Result<()> {
        //

        Ok(())
    }
}
