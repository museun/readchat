use twitchchat::commands::Channel;

const HEADER: &str = concat!("readchat ", env!("CARGO_PKG_VERSION"));

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
    -m, --min-width <int>  if window size is below this, use a compact view
    -s, --timestamp        render timestamps of messages, in the compact view

arguments:
    <string>               the twitch channel to join
";

pub struct Args {
    pub channel: String,
    pub nick_max: usize,
    pub buffer_max: usize,
    pub debug: bool,
    pub transcribe: bool,
    pub timestamps: bool,
    pub min_width: Option<usize>,
}

impl Args {
    pub fn parse() -> anyhow::Result<Self> {
        let mut args = pico_args::Arguments::from_env();
        if args.contains(["-h", "--help"]) {
            exit_normally(&[&HEADER, &HELP_MESSAGE]);
        }

        if args.contains(["-v", "--version"]) {
            exit_normally(&[&HEADER]);
        }

        if args.contains(["-l", "--log-dir"]) {
            exit_normally(&[&crate::Logger::get_dir()?.display()]);
        }

        let nick_max: usize = args.opt_value_from_str(["-n", "--nick-max"])?.unwrap_or(11);
        let buffer_max: usize = args
            .opt_value_from_str(["-b", "--buffer-max"])?
            .unwrap_or(100);

        let min_width = args.opt_value_from_str(["-m", "--min-width"])?;

        let debug = args.contains(["-d", "--debug"]);
        let transcribe = args.contains(["-t", "--transcribe"]);
        let timestamps = args.contains(["-s", "--timestamp"]);

        let mut channels = args.finish();
        let channel = match channels.len() {
            _ if debug => "#testing".to_string(),
            1 => channels.remove(0).into_string().map_err(|s| {
                // TODO we shouldn't really care if its utf-8 or not. probably
                anyhow::anyhow!("string contains invalid utf-8, '{}'", s.to_string_lossy())
            })?,
            0 => exit_with_error("ERROR: a channel must be provded"),
            _ => exit_with_error("ERROR: only a single channel can be provded"),
        };

        // this'll format/correct the channel for us
        let channel = Channel::new(&channel).to_string();

        Ok(Self {
            nick_max,
            buffer_max,
            min_width,
            channel,
            debug,
            transcribe,
            timestamps,
        })
    }
}

fn exit_normally(msgs: &[&dyn ToString]) -> ! {
    for msg in msgs {
        println!("{}", msg.to_string());
    }
    std::process::exit(0)
}

fn exit_with_error(msg: &str) -> ! {
    eprintln!("{}", msg);
    std::process::exit(1);
}
