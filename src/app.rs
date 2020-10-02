use std::net::TcpStream;

use crate::{
    args::Args,
    keys::{self, Message},
    twitch,
    window::{UpdateMode, Window},
};

use crossterm::event::*;
use flume as channel;

pub fn main_loop(args: Args) -> anyhow::Result<()> {
    let mut window = Window::new(args.nick_max, args.buffer_max);

    // TODO timeout logic here
    let conn = connect(args.debug)?;

    let (messages_tx, messages) = channel::bounded(64);
    let (done_tx, done) = channel::bounded(1);

    let _ = std::thread::spawn(move || {
        let _ = twitch::run_to_completion(args.channel, messages_tx, conn);
        drop(done_tx)
    });

    let (events_tx, events_rx) = channel::bounded(32);

    let mut waiting_for_key = false;

    'outer: while keep_running(&done) {
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

                Message::Delete if !waiting_for_key => {
                    waiting_for_key = true;
                    window.update(UpdateMode::MarkAll)?;
                }

                Message::Delete if waiting_for_key => {
                    waiting_for_key = false;
                    window.update(UpdateMode::Redraw)?
                }

                Message::Char(ch) if waiting_for_key => {
                    window.delete(ch)?;
                    waiting_for_key = false;
                    continue 'outer;
                }

                _ => {}
            }
        }

        if waiting_for_key {
            continue 'outer;
        }

        for msg in messages.try_iter() {
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

fn keep_running(ch: &channel::Receiver<()>) -> bool {
    matches!(ch.try_recv(), Err(channel::TryRecvError::Empty))
}
