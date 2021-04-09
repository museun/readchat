fn setup_panic_logger() {
    std::panic::set_hook(Box::new(|info| {
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>",
            },
        };

        use std::io::Write as _;

        let mut fi = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open("readchat-panics.log")
            .unwrap();

        match info.location() {
            Some(loc) => writeln!(&mut fi, "{}: {}:{}", msg, loc.file(), loc.line()).unwrap(),
            None => writeln!(&mut fi, "{}", msg).unwrap(),
        }
    }));
}

fn main() -> anyhow::Result<()> {
    let args = readchat::Args::parse()?;

    let logger = if args.debug {
        readchat::Logger::default()
    } else {
        readchat::Logger::from_xdg(&args.channel)?
    };

    setup_panic_logger();

    let _screen = readchat::AltScreen::enter();
    readchat::App::run(args, logger)
}
