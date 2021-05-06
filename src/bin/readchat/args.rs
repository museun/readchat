use std::fmt::Display;

use twitchchat::commands::Channel;

const PACKAGE_NAME: &str = concat!(env!("CARGO_PKG_NAME"), " ", env!("CARGO_PKG_VERSION"));

const READCHAT_HELP: &str = r#"
description:
    a simple read-only client for a single twitch channel's chat

usage:
readchat <channel> [flags|arguments]

arguments:
    -b, --buffer-max <int>    the number of messages to keep in history

flags:
    -d, --debug               enables the test-debug mode
    -h, --help                prints the help message
    -l, --data-dir            prints the data directory and exit
    -t, --transcribe          logs the chat to disk
    -v, --version             prints the current version

arguments:
    <channel>                 a twitch channel to join
                              NOTE: the leading '#' is optional
"#;

#[derive(Debug)]
pub struct Args {
    pub channel: String,
    pub debug_mode: bool,
    pub transcribe: bool,
    pub buffer_max: usize,
}

impl Args {
    pub fn parse() -> anyhow::Result<Self> {
        let mut args = pico_args::Arguments::from_env();
        if args.contains(["-h", "--help"]) {
            exit_normally([&PACKAGE_NAME, &READCHAT_HELP]);
        }

        if args.contains(["-v", "--version"]) {
            exit_normally([&PACKAGE_NAME]);
        }

        if args.contains(["-l", "--data-dir"]) {
            exit_normally([&readchat::Logger::get_dir()?.display()]);
        }

        let debug_mode = args.contains(["-d", "--debug"]);
        let transcribe = args.contains(["-t", "--transcribe"]);
        let buffer_max = args
            .opt_value_from_str(["-b", "--buffer-max"])?
            .unwrap_or(50);

        let mut channels = args.finish();
        let channel = match channels.len() {
            _ if debug_mode => "#testing".to_string(),

            1 => channels
                .remove(0)
                .into_string()
                .map(|s| Channel::new(&s).to_string())
                .unwrap_or_else(|s| {
                    exit_abnormally([&format!(
                        "string contains invalid utf-8, {}",
                        s.to_string_lossy()
                    )])
                }),

            0 => exit_abnormally([&"ERROR: a channel must be provided"]),

            _ => exit_abnormally([&"ERROR: only a single channel can be provided"]),
        };

        Ok(Self {
            channel,
            debug_mode,
            transcribe,
            buffer_max,
        })
    }
}

fn exit_with<const N: usize>(msg: [&dyn Display; N], status: i32) -> ! {
    for msg in readchat::array_iter(msg) {
        eprintln!("{}", msg);
    }
    std::process::exit(status);
}

fn exit_abnormally<const N: usize>(msg: [&dyn Display; N]) -> ! {
    exit_with(msg, 1)
}

fn exit_normally<const N: usize>(msg: [&dyn Display; N]) -> ! {
    exit_with(msg, 0)
}
