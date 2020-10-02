const HEADER: &str = concat!("readchat ", env!("CARGO_PKG_VERSION"),);

const HELP_MESSAGE: &str = "
description:
    simply read-only client for a single twitch channel's chat

usage:
    readchat <channel>

flags:
    -h, --help             prints this message
    -v, --version          prints the version
    -d, --debug            use simulated debug stream instead
    -t, --transcribe       log this channel to a file (when not in debug mode)
    -l, --log-dir          print the log directory and exit

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
    pub transcribe: bool,
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

        if args.contains(["-l", "--log-dir"]) {
            let dir = crate::Logger::get_dir()?;
            println!("{}", dir.display());
            std::process::exit(0)
        }

        let nick_max: usize = args.opt_value_from_str(["-n", "--nick-max"])?.unwrap_or(11);
        let buffer_max: usize = args
            .opt_value_from_str(["-b", "--buffer-max"])?
            .unwrap_or(100);

        let debug = args.contains(["-d", "--debug"]);
        let transcribe = args.contains(["-t", "--transcribe"]);

        let mut channels = args.free()?;
        let channel = match channels.len() {
            _ if debug => "#testing".to_string(),
            1 => channels.remove(0),
            0 => {
                eprintln!("ERROR: a channel must be provded");
                std::process::exit(1);
            }
            _ => {
                eprintln!("ERROR: only a single channel can be provded");
                std::process::exit(1);
            }
        };

        // this'll format/correct the channel for us
        let channel = twitchchat::commands::Channel::new(&channel).to_string();

        Ok(Self {
            nick_max,
            buffer_max,
            channel,
            debug,
            transcribe,
        })
    }
}
