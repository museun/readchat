pub fn setup() {
    std::panic::set_hook(Box::new(move |info| {
        use std::io::Write as _;

        let msg = info
            .payload()
            .downcast_ref::<&'static str>()
            .copied()
            .or_else(|| info.payload().downcast_ref::<String>().map(|s| s.as_str()))
            .unwrap_or("Box<Any>");

        let path = readchat::Logger::get_dir()
            .expect("get log dir")
            .join("panics")
            .with_extension("log");

        let mut fi = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .write(true)
            .open(path)
            .expect("open panics.log for writing");

        match info.location() {
            Some(loc) => {
                writeln!(&mut fi, "{}: {}:{}", msg, loc.file(), loc.line())
            }
            None => {
                writeln!(&mut fi, "{}", msg)
            }
        }
        .unwrap()
    }));
}
