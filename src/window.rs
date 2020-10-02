use super::{
    args::Args,
    keys::{self, Message},
    queue::Queue,
    twitch::TwitchChat,
    util,
};

use std::io::Write;

use twitchchat::messages::Privmsg;
use twitchchat::twitch::color::RGB;

use flume as channel;

use crossterm::{
    cursor::*,
    event::*,
    style::*,
    terminal::{self, *},
};

fn keep_running(ch: &channel::Receiver<()>) -> bool {
    matches!(ch.try_recv(), Err(channel::TryRecvError::Empty))
}

pub fn main_loop(args: Args) -> anyhow::Result<()> {
    let mut window = Window::new(args.nick_max, args.buffer_max);

    let conn = if args.debug {
        let addr = crate::testing::make_interesting_chat(crate::testing::TestingOpts::load())?;
        std::net::TcpStream::connect(addr)?
    } else {
        std::net::TcpStream::connect(twitchchat::TWITCH_IRC_ADDRESS)?
    };

    let (messages_tx, messages) = channel::bounded(64);
    let (done_tx, done) = channel::bounded(1);

    let _ = std::thread::spawn(move || {
        let _ = TwitchChat::run_to_completion(args.channel, messages_tx, conn);
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

enum UpdateMode {
    Redraw,
    Append,
    MarkAll,
}

struct Window {
    queue: Queue<Privmsg<'static>>,
    left: usize,
    pad: String,
}

impl Window {
    fn new(left: usize, limit: usize) -> Self {
        Self {
            left,
            pad: " ".repeat(left),
            queue: Queue::with_size(limit),
        }
    }

    fn push(&mut self, message: Privmsg<'static>) {
        self.queue.push(message);
    }

    fn update(&mut self, update: UpdateMode) -> anyhow::Result<()> {
        let (width, height) = terminal::size()?;
        let mut stdout = std::io::stdout();

        match update {
            UpdateMode::Redraw if self.queue.is_empty() => return Ok(()),
            UpdateMode::Redraw => {
                crossterm::execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
                for msg in self.queue.iter().rev().take(height as _).rev() {
                    let state = self.state(width);
                    print_message(&mut stdout, msg, state)?;
                }
            }
            UpdateMode::Append => {
                if let Some(msg) = self.queue.last() {
                    if self.queue.len() == 1 {
                        crossterm::execute!(stdout, MoveTo(0, 0))?;
                    }
                    let state = self.state(width);
                    print_message(&mut stdout, msg, state)?;
                }
            }
            UpdateMode::MarkAll => {
                crossterm::queue!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;

                let iter = self.queue.iter().rev().take((height) as _).rev();
                let mut ch = ALPHA.iter().take(iter.len()).rev();

                for msg in iter {
                    let mut state = self.state(width);
                    state.prefix.replace(*ch.next().unwrap());
                    print_message(&mut stdout, msg, state)?;
                }
            }
        }

        stdout.flush()?;
        Ok(())
    }

    fn delete(&mut self, ch: char) -> anyhow::Result<()> {
        if let Some(p) = ALPHA.iter().position(|&c| c == ch) {
            self.queue.remove_rev(p)
        }
        self.update(UpdateMode::Redraw)
    }

    fn state(&self, width: u16) -> State<'_> {
        State {
            prefix: None,
            left: self.left,
            width: width as _,
            pad: &self.pad,
        }
    }
}

const ALPHA: &[char] = &[
    'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S',
    'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b',
    'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u',
    'v', 'w', 'x', 'y', 'z',
];

struct State<'a> {
    prefix: Option<char>,
    left: usize,
    width: usize,
    pad: &'a str,
}

fn print_message(
    stdout: &mut std::io::Stdout,
    msg: &Privmsg<'_>,
    state: State<'_>,
) -> anyhow::Result<()> {
    let RGB(r, g, b) = msg.color().unwrap_or_default().rgb;
    let color = Color::Rgb { r, g, b };

    let p = state.prefix.map(|_| 4).unwrap_or(0);

    let name = util::truncate_or_pad(&msg.name(), state.left - p);
    let name = style(name).with(color);

    let partition = util::partition(&msg.data(), state.width - p - state.left - 1);

    for (i, part) in partition.into_iter().enumerate() {
        let first = i == 0;

        crossterm::queue!(stdout, Print("\n"), MoveToColumn(0))?;

        if let Some(prefix) = state.prefix {
            if first {
                crossterm::queue!(
                    stdout,
                    Print("["),
                    Print(style(prefix).with(Color::Yellow)),
                    Print("] ")
                )?;
            } else {
                crossterm::queue!(stdout, Print("    "),)?;
            }
        }

        if first {
            crossterm::queue!(stdout, Print(&name))?;
        } else {
            crossterm::queue!(stdout, Print(&state.pad[..state.pad.len() - p]))?;
        }
        crossterm::queue!(stdout, Print(" "), Print(part))?;
    }

    Ok(())
}
