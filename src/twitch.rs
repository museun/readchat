use std::io::Write;
use std::net::TcpStream;

use twitchchat::messages::{Commands, Privmsg};
use twitchchat::FromIrcMessage;

use channel::Sender;
use flume as channel;

use crossterm::{cursor::*, style::*, terminal::*};

pub(super) fn run_to_completion(
    channel: String,
    messages: Sender<Privmsg<'static>>,
    conn: TcpStream,
) -> anyhow::Result<()> {
    let conn = &conn;

    crossterm::execute!(
        std::io::stdout(),
        MoveTo(0, 0),
        Print(style("connecting..").with(Color::Cyan)),
        MoveToNextLine(1),
    )?;

    let user_config = twitchchat::UserConfig::builder()
        .anonymous()
        .enable_all_capabilities()
        .build()?;

    let mut decoder = twitchchat::Decoder::new(conn);
    let mut encoder = twitchchat::Encoder::new(conn);
    encoder.encode(twitchchat::commands::register(&user_config))?;

    let mut out = std::io::stdout();

    // ensure its converted properly.
    let channel = twitchchat::commands::Channel::new(&channel).to_string();

    replace_line(
        &mut out,
        format!(
            "{}{}",
            style("joining ").with(Color::Cyan),
            style(&channel).with(Color::Green),
        ),
    )?;

    // TODO timeout logic here

    // wait for ready
    while let Some(msg) = decoder.next() {
        let msg = twitchchat::messages::Commands::from_irc(msg?)?;
        if let Commands::IrcReady(_) = msg {
            break;
        }
    }

    // join the channel
    encoder.encode(twitchchat::commands::join(&channel))?;

    // wait for join
    while let Some(msg) = decoder.next() {
        let msg = twitchchat::messages::Commands::from_irc(msg?)?;
        if let Commands::Join(msg) = msg {
            if msg.channel() == &*channel && msg.name() == "justinfan1234" {
                replace_line(
                    &mut out,
                    format!(
                        "{}{}",
                        style("joined ").with(Color::Cyan),
                        style(&channel).with(Color::Green),
                    ),
                )?;
                break;
            }
        }
    }

    // and then run the main loop
    while let Some(Ok(msg)) = decoder.next() {
        if let Commands::Privmsg(msg) = twitchchat::messages::Commands::from_irc(msg)? {
            if messages.send(msg).is_err() {
                break;
            }
        }

        // Commands::ClearChat(_) => {}
        // Commands::ClearMsg(_) => {}
        // Commands::HostTarget(_) => {}
        // Commands::Notice(_) => {}
        // Commands::UserNotice(_) => {}
    }

    Ok(())
}

fn replace_line(w: &mut impl Write, line: impl ToString) -> anyhow::Result<()> {
    crossterm::execute!(
        w,
        MoveToPreviousLine(1),
        Clear(ClearType::CurrentLine),
        Print(line.to_string()),
        MoveToNextLine(1),
    )?;
    Ok(())
}
