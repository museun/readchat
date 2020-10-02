fn main() -> anyhow::Result<()> {
    let args = readchat::Args::parse()?;
    let _screen = readchat::AltScreen::enter();
    readchat::main_loop(args)
}
