fn setup_panic_logger() {
    std::panic::set_hook(Box::new(move |info| {
        let msg = match info.payload().downcast_ref::<&'static str>() {
            Some(s) => *s,
            None => match info.payload().downcast_ref::<String>() {
                Some(s) => &**s,
                None => "Box<Any>",
            },
        };

        use std::io::Write as _;

        let path = readchat::Logger::get_dir()
            .expect("get log dir")
            .join("panics.log");

        let mut fi = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(path)
            .expect("open panics.log file for writing");

        match info.location() {
            Some(loc) => writeln!(&mut fi, "{}: {}:{}", msg, loc.file(), loc.line()).unwrap(),
            None => writeln!(&mut fi, "{}", msg).unwrap(),
        }
    }));
}

fn main() -> anyhow::Result<()> {
    let args = readchat::Args::parse()?;

    let color_config = readchat::colors::ColorConfig::load()?;

    if args.color_test {
        readchat::colors::show_off_colors(&mut std::io::stdout(), &color_config)?;
        return Ok(());
    }

    let logger = if args.debug {
        readchat::Logger::default()
    } else {
        readchat::Logger::from_xdg(&args.channel)?
    };

    setup_panic_logger();

    let _screen = readchat::AltScreen::enter();
    readchat::App::run(args, logger)
}
