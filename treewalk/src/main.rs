use loxrs::cli;

fn main() -> cli::Result<()> {
    env_logger::init();
    let cli = cli::parse()?;
    cli.run()
}
