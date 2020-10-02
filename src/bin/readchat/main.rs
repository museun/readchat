fn main() -> anyhow::Result<()> {
    let args = readchat::Args::parse()?;

    let logger = if args.debug {
        readchat::Logger::default()
    } else {
        readchat::Logger::from_xdg(&args.channel)?
    };

    let _screen = readchat::AltScreen::enter();
    readchat::main_loop(args, logger)
}
