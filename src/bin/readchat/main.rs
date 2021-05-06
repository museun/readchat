use std::sync::mpsc::SyncSender;

use readchat::*;

mod args;
mod panic_logger;

use twitchchat::messages::Commands::{self, *};
use twitchchat::{commands::*, FromIrcMessage};

fn twitch_main_loop(channel: &str, sender: SyncSender<Message>) -> anyhow::Result<()> {
    let send = |msg| anyhow::Context::context(sender.send(msg).ok(), "app is in an invalid state");
    let mut name = None;

    send(Message::Connecting)?;

    let stream = std::net::TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS)?;
    let mut encoder = twitchchat::Encoder::new(&stream);

    let config = twitchchat::UserConfig::builder()
        .anonymous()
        .enable_all_capabilities()
        .build()?;
    encoder.encode(register(&config))?;

    for message in twitchchat::Decoder::new(&stream)
        .into_iter()
        .flatten()
        .map(Commands::from_irc)
        .flatten()
    {
        match message {
            Ready(msg) => {
                name.get_or_insert(msg.username().to_string());
                send(Message::Connected)?;
                send(Message::Joining)?;
                encoder.encode(join(channel))?;
            }

            ClearChat(_) => {
                // clear the whole chat (should this do anything?)
            }

            ClearMsg(_) => {
                // mark a message as deleted
            }

            Join(msg) if name.as_deref().filter(|&name| name == msg.name()).is_some() =>
            // && msg.channel() == channel
            {
                send(Message::Joined)?;
            }

            Privmsg(msg) => {
                // if msg.channel() == channel
                send(Message::Append(msg.into()))?;
            }

            //GlobalUserState(_) => {}
            //Reconnect(_) => {}
            _ => {}
        }
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    simple_env_load::load_env_from(array_iter([".env", ".env.dev"]));

    let mut args = args::Args::parse()?;
    let logger = args
        .transcribe
        .then(|| Logger::from_xdg(&args.channel))
        .transpose()?
        .unwrap_or_default();

    if args.debug_mode {
        unimplemented!("not yet")
    }

    panic_logger::setup();

    let _screen = AltScreen::enter();

    let (app, tx) = App::create(
        Window::with_limit(args.buffer_max), //
        logger,
        &args.channel,
    );

    let channel = std::mem::take(&mut args.channel);
    let _handle = std::thread::spawn(move || {
        let channel = channel;
        twitch_main_loop(&channel, tx)
    });

    // let handle = std::thread::spawn({
    //     let sender = tx.clone();
    //     move || {
    //         use debug::*;
    //         let _: Result<(), _> = simulated_chat(DebugOpts::load())
    //             .map(Into::into)
    //             .map(Message::Append)
    //             .map(|msg| sender.send(msg))
    //             .try_for_each(|s| s);
    //     }
    // });

    app.run_on_buffered_stdout()?;

    // handle.join().unwrap().unwrap();
    Ok(())
}
