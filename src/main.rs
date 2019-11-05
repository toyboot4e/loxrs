fn main() {
    ::env_logger::init();

    let cli = loxrs::parse_args();
    cli.run();
}
