const HEADER: &str = concat!(
    "readchat ",
    env!("CARGO_PKG_VERSION"),
    " (",
    env!("GIT_REVISION"),
    ")"
);

const HELP_MESSAGE: &str = "
description:
    simply read-only client for a single twitch channel's chat

usage:
    readchat <channel>

flags:
    -h, --help             prints this message
    -v, --version          prints the version
    -d, --debug            use a debug stream source instead of actually connecting

optional flags:
    -n, --nick-max <int>   the max width before truncation of nicknames
    -b, --buffer-max <int> the number of messages to keep in the redraw queue

arguments:
    <string>               the twitch channel to join
";

pub struct Args {
    pub channel: String,
    pub nick_max: usize,
    pub buffer_max: usize,
    pub debug: bool,
}

impl Args {
    pub fn parse() -> anyhow::Result<Self> {
        let mut args = pico_args::Arguments::from_env();
        if args.contains(["-h", "--help"]) {
            println!("{}", HEADER);
            println!("{}", HELP_MESSAGE);
            std::process::exit(0);
        }

        if args.contains(["-v", "--version"]) {
            println!("{}", HEADER);
            std::process::exit(0);
        }

        let nick_max: usize = args.opt_value_from_str(["-n", "--nick-max"])?.unwrap_or(11);
        let buffer_max: usize = args
            .opt_value_from_str(["-b", "--buffer-max"])?
            .unwrap_or(100);

        let debug = args.contains(["-d", "--debug"]);

        let mut channels = args.free()?;

        let channel = if !debug {
            match channels.len() {
                0 => {
                    eprintln!("ERROR: a channel must be provded");
                    std::process::exit(1);
                }
                1 => channels.remove(0),
                _ => {
                    eprintln!("ERROR: only a single channel can be provded");
                    std::process::exit(1);
                }
            }
        } else {
            "#testing".to_string()
        };

        Ok(Self {
            nick_max,
            buffer_max,
            channel,
            debug,
        })
    }
}
