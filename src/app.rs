use std::net::TcpStream;

use crate::{
    args::Args,
    keys::{self, Message},
    twitch,
    window::{UpdateMode, Window},
    Logger,
};

use crossterm::event::*;
use flume as channel;

pub fn main_loop(args: Args, mut logger: Logger) -> anyhow::Result<()> {
    logger.transcribe(&format!("*** session start: {}", crate::timestamp()))?;

    let mut window = Window::new(args.nick_max, args.buffer_max);
    let conn = connect(args.debug)?;
    let (sender, messages) = channel::bounded(64);

    let _ = std::thread::spawn(move || {
        let _ = twitch::run_to_completion(args.channel, sender, conn);
    });
    let (events_tx, events_rx) = channel::bounded(32);
    let mut waiting = false;

    'outer: while keep_running(&messages) {
        if crossterm::event::poll(std::time::Duration::from_millis(150))? {
            match crossterm::event::read()? {
                Event::Key(event) => keys::handle(event, &events_tx),
                Event::Resize(_, _) => window.update(UpdateMode::Redraw)?,
                _ => {}
            }
        }

        for event in events_rx.try_iter() {
            match event {
                Message::Quit => break 'outer,

                Message::Redraw => window.update(UpdateMode::Redraw)?,

                Message::Delete if !waiting => {
                    waiting = true;
                    window.update(UpdateMode::MarkAll)?;
                }

                Message::Delete if waiting => {
                    waiting = false;
                    window.update(UpdateMode::Redraw)?
                }

                Message::Char(ch) if waiting => {
                    window.delete(ch)?;
                    waiting = false;
                    continue 'outer;
                }

                Message::NameColumnGrow | Message::NameColumnShrink => {
                    use UpdateMode as U;
                    const COLUMN_ACTION: [fn(&mut Window) -> bool; 2] = [
                        Window::grow_nick_column, //
                        Window::shrink_nick_column,
                    ];

                    // pick the current 'mode'
                    let choice: usize = matches!(event, Message::NameColumnGrow) as u8 as _;
                    // should we update the window?
                    if COLUMN_ACTION[choice](&mut window) {
                        window.update(if waiting { U::MarkAll } else { U::Redraw })?;
                    }
                }

                _ => {}
            }
        }

        if waiting {
            continue 'outer;
        }

        for msg in messages.try_iter() {
            logger.transcribe(&format!(
                "{} {}: {}",
                crate::timestamp(),
                msg.name(),
                msg.data()
            ))?;
            window.push(msg);
            window.update(UpdateMode::Append)?;
        }
    }

    Ok(())
}

fn connect(debug: bool) -> anyhow::Result<TcpStream> {
    use crate::testing::*;
    let conn = if debug {
        let addr = make_interesting_chat(TestingOpts::load())?;
        TcpStream::connect(addr)?
    } else {
        TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS)?
    };

    Ok(conn)
}

fn keep_running<T>(ch: &channel::Receiver<T>) -> bool {
    !matches!(ch.try_recv(), Err(channel::TryRecvError::Disconnected))
}
